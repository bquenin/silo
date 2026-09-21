//! Prepare a replay without modifying the user's content configuration.
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

use anyhow::{ensure, Context, Result};
use serde::Serialize;

use super::{config, content, download, selection, LaunchPlan, LaunchResult, ReplayTarget};

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub phase: &'static str,
    pub message: String,
    /// Bytes processed in the current download or extraction phase.
    pub completed: u64,
    pub total: Option<u64>,
}

#[derive(Default)]
pub struct Cancellation(AtomicU8);
impl Cancellation {
    /// Once launch is committed, cancellation cannot promise to prevent it.
    pub fn cancel(&self) -> bool {
        self.0
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            || self.0.load(Ordering::SeqCst) == 1
    }
}

pub struct Control<'a> {
    pub cancelled: &'a Cancellation,
    pub progress: &'a dyn Fn(Progress),
}

impl Control<'_> {
    pub fn check(&self) -> Result<()> {
        ensure!(
            self.cancelled.0.load(Ordering::SeqCst) != 1,
            "Replay preparation cancelled."
        );
        Ok(())
    }
    pub fn stage(&self, phase: &'static str, message: impl Into<String>) -> Result<()> {
        self.check()?;
        (self.progress)(Progress {
            phase,
            message: message.into(),
            completed: 0,
            total: None,
        });
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Readiness {
    pub can_play: bool,
    pub message: String,
    pub required_revision: Option<String>,
    pub game_path: Option<PathBuf>,
    pub details: Vec<String>,
}

pub fn cache_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            crate::ingest::default_catalogue_path()
                .parent()
                .unwrap()
                .to_path_buf()
        })
        .join("tacitus/playback")
}

/// Opening the playback dialog never downloads or creates a session.
pub fn inspect(target: &ReplayTarget, game_path: Option<&Path>, cache: &Path) -> Readiness {
    let mut report = Readiness {
        can_play: false,
        message: String::new(),
        required_revision: super::map_asset(&target.map_path).and_then(|(_, r)| r),
        game_path: game_path.map(Path::to_path_buf),
        details: Vec::new(),
    };
    let result = (|| -> Result<()> {
        let game_path =
            game_path.context("Choose your Kane's Wrath game folder to play replays.")?;
        replay_path(target)?;
        let game = config::base(game_path, target.version)?;
        let (asset, revision, crc) = requirement(target)?;
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        if let Some(found) = content::resolve(
            &game,
            cache,
            &asset,
            revision.as_deref(),
            crc,
            false,
            &control,
        )? {
            report.message = "Ready to play.".into();
            report.details = found
                .archives
                .iter()
                .chain(found.custom_map.iter())
                .map(|p| p.display().to_string())
                .collect();
        } else {
            ensure!(!asset.starts_with("data/maps/internal/"),
                "The exact custom map is missing from the game's Maps folder, or its file checksum differs from this replay.");
            ensure!(community_asset(&asset), "The required base-game map or its recorded compatibility value is missing. Repair the game installation.");
            let crc = crc
                .context("The required base-game map is missing. Repair the game installation.")?;
            ensure!(
                super::sources::supports_compatibility(revision.as_deref(), crc)
                    || revision.as_deref().is_some_and(download::supported),
                "Tacitus has no verified automatic download source for this map's compatibility value {crc:X}."
            );
            report.message =
                "The required map and scripts will be prepared when you press Play.".into();
        }
        report.can_play = true;
        Ok(())
    })();
    if let Err(error) = result {
        report.message = format!("{error:#}");
    }
    report
}

fn replay_path(target: &ReplayTarget) -> Result<PathBuf> {
    let path = target
        .file_path
        .as_ref()
        .map(Path::new)
        .filter(|p| p.is_file())
        .context("The replay file is missing. Reimport its folder to update its location.")?;
    Ok(dunce::canonicalize(path)?)
}

fn requirement(target: &ReplayTarget) -> Result<(String, Option<String>, Option<u32>)> {
    let (asset, revision) = super::map_asset(&target.map_path)
        .context("This custom map is not supported by automatic playback yet.")?;
    let crc = Some(
        u32::from_str_radix(&target.map_crc, 16)
            .context("The replay does not contain a valid map compatibility value")?,
    );
    Ok((asset, revision, crc))
}

pub(super) fn community_asset(asset: &str) -> bool {
    asset.contains("__") || asset.contains("1.02+") || asset.contains("1.03")
}

/// Cache a separately downloaded, verified package for an exact replay.
pub fn import_package(
    target: &ReplayTarget,
    cache: &Path,
    package: &Path,
    source: &str,
    expected_sha256: &str,
    control: &Control<'_>,
) -> Result<PathBuf> {
    let replay = replay_path(target)?;
    ensure!(
        content::hash_file(&replay, control)?.0 == target.file_hash,
        "The replay has changed since import. Import it again before preparing content."
    );
    let (asset, revision, crc) = requirement(target)?;
    let revision = revision.context("This replay does not identify a community map revision.")?;
    download::import_package(
        cache,
        package,
        &asset,
        &revision,
        crc.context("This replay does not identify a community map compatibility value")?,
        source,
        expected_sha256,
        control,
    )
}

/// Owns only generated files. Content archives remain in their original
/// location or the persistent cache, and the game directory is never written.
pub struct Prepared {
    pub plan: LaunchPlan,
    pub archives: Vec<PathBuf>,
    session: tempfile::TempDir,
}

impl Prepared {
    /// Explicit CLI preparation keeps the plan for inspection or a subsequent
    /// manual launch. The normal Play path owns the session until game exit.
    pub fn keep(self) -> LaunchPlan {
        let _ = self.session.keep();
        self.plan
    }
}

pub fn prepare(
    target: &ReplayTarget,
    game_path: &Path,
    cache: &Path,
    allow_download: bool,
    control: &Control<'_>,
) -> Result<Prepared> {
    control.stage("checking", "Checking the replay and game files…")?;
    let replay = replay_path(target)?;
    let (hash, _) = content::hash_file(&replay, control)?;
    ensure!(
        hash == target.file_hash,
        "The replay has changed since import. Import it again before playing."
    );
    let game = config::base(game_path, target.version)?;
    let (asset, revision, crc) = requirement(target)?;
    control.stage("checking", "Looking for the required map and scripts…")?;
    let mut found = content::resolve(
        &game,
        cache,
        &asset,
        revision.as_deref(),
        crc,
        true,
        control,
    )?;
    if found.is_none() && allow_download {
        ensure!(!asset.starts_with("data/maps/internal/"),
            "The exact custom map is missing from the game's Maps folder, or its file checksum differs from this replay.");
        ensure!(community_asset(&asset), "The required base-game map or its recorded compatibility value is missing. Repair the game installation.");
        let crc =
            crc.context("The required base-game map is missing. Repair the game installation.")?;
        let mut known = content::pack_hints(&game, cache, &asset, control)?;
        if known.is_empty() {
            known = super::sources::compatible_kinds(revision.as_deref(), crc);
        }
        // Early packs mixed capacities before the modern category split.
        let players = if revision.is_none() {
            0
        } else {
            target.n_players
        };
        let sources = selection::pages(revision.as_deref().unwrap_or(""), &known, players);
        download::acquire(cache, &asset, revision.as_deref(), crc, &sources, control)?;
        found = content::resolve(
            &game,
            cache,
            &asset,
            revision.as_deref(),
            Some(crc),
            true,
            control,
        )?;
    }
    let found = found.context("The required map or matching scripts are not available locally.")?;
    control.stage("preparing", "Preparing replay playback…")?;
    let sessions = cache.join("sessions");
    fs::create_dir_all(&sessions)?;
    let session = tempfile::Builder::new()
        .prefix("replay-")
        .tempdir_in(dunce::canonicalize(&sessions)?)?;
    let sku = session.path().join("replay.SkuDef");
    let mut text = format!("set-exe {}\n", directive_path(&game.executable)?);
    // SAGE resolves earlier archives before later ones; replay dependencies
    // precede stock content, as community packs do in the normal config chain.
    for archive in found.archives.iter().chain(game.archives.iter()) {
        text.push_str(&format!("add-big {}\n", directive_path(archive)?));
    }
    for directory in &game.search_paths {
        text.push_str(&format!("add-search-path {}\n", directive_path(directory)?));
    }
    text.push_str("add-search-path big:\n");
    fs::write(&sku, text)?;
    fs::write(session.path().join("tacitus-session"), "1\n")?;
    control.check()?;
    Ok(Prepared {
        plan: LaunchPlan {
            executable: game.executable,
            working_directory: game.root,
            args: vec![
                "-replayGame".into(),
                replay.to_string_lossy().into_owned(),
                "-config".into(),
                sku.to_string_lossy().into_owned(),
            ],
        },
        archives: found.archives,
        session,
    })
}

fn directive_path(path: &Path) -> Result<String> {
    let path = dunce::canonicalize(path)?;
    let path = path
        .to_str()
        .context("A content path cannot be represented as text")?;
    ensure!(
        !path.contains(['\n', '\r', '"']),
        "A content path contains unsupported characters"
    );
    // SAGE consumes the remainder of the line as the path (including spaces).
    Ok(path.replace('/', "\\"))
}

pub fn launch(
    target: &ReplayTarget,
    game_path: &Path,
    cache: &Path,
    control: &Control<'_>,
) -> Result<LaunchResult> {
    ensure!(cfg!(windows), "Replay playback is available on Windows.");
    static LAUNCH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LAUNCH_LOCK
        .try_lock()
        .map_err(|_| anyhow::anyhow!("Another replay is being prepared."))?;
    ensure!(
        !super::game_running()?,
        "Kane's Wrath is already running. Close it before playing a replay."
    );
    cleanup_sessions(cache)?;
    let prepared = prepare(target, game_path, cache, true, control)?;
    ensure!(
        !super::game_running()?,
        "Kane's Wrath was started during preparation. Close it and press Play again."
    );
    control.stage("launching", "Starting Kane's Wrath…")?;
    control
        .cancelled
        .0
        .compare_exchange(0, 2, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| anyhow::anyhow!("Replay preparation cancelled."))?;
    let mut child = super::launch_command(&prepared.plan)
        .spawn()
        .context("Start Kane's Wrath")?;
    let pid = child.id();
    // A process exit leaves its generated files for the next startup. They
    // must survive Tacitus closing while the game still uses the session.
    std::thread::spawn(move || {
        let _ = child.wait();
        drop(prepared);
    });
    Ok(LaunchResult { pid })
}

fn cleanup_sessions(cache: &Path) -> Result<()> {
    let sessions = cache.join("sessions");
    if !sessions.is_dir() {
        return Ok(());
    }
    let root = dunce::canonicalize(&sessions)?;
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            || !entry.file_name().to_string_lossy().starts_with("replay-")
        {
            continue;
        }
        let path = dunce::canonicalize(entry.path())?;
        if path.parent() == Some(root.as_path())
            && fs::read(path.join("tacitus-session")).ok().as_deref() == Some(b"1\n")
        {
            // The caller established no KW process is running. Only our own
            // marked session directory, never an archive's directory, is removed.
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "automatic_tests.rs"]
mod tests;
