//! Replay launch preflight: engine versions, map archives, and game arguments.
//! See docs/replay-launcher.md for compatibility checks and verification limits.

mod archive;
mod config;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct ReplayTarget {
    pub id: i64,
    pub file_path: Option<String>,
    pub file_hash: String,
    pub map_name: String,
    pub map_path: String,
    pub map_crc: String,
    pub version: [u32; 4],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub sku_path: Option<PathBuf>,
}

pub fn settings_path() -> PathBuf {
    crate::ingest::default_catalogue_path().with_file_name("launcher.json")
}

pub fn load_settings() -> Result<Settings> {
    let path = settings_path();
    if path.is_file() {
        return serde_json::from_slice(&fs::read(path)?).context("Read launcher settings");
    }
    // Other installs/languages can be selected explicitly in the UI.
    let sku_path = std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .map(|p| p.join("Steam/steamapps/common/Command and Conquer 3 - Kane's Wrath/CNC3EP1_english_1.2.SkuDef"))
        .filter(|p| p.is_file());
    Ok(Settings { sku_path })
}

pub fn save_settings(sku: &Path) -> Result<Settings> {
    let game = config::read(sku)?;
    let settings = Settings {
        sku_path: Some(game.sku),
    };
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&settings)?)?;
    Ok(settings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ready,
    NotConfigured,
    ReplayMissing,
    EngineMismatch,
    MapMissing,
    MapDisabled,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapProvider {
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub working_directory: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub replay_id: i64,
    pub status: Status,
    pub message: String,
    pub map_name: String,
    pub map_path: String,
    pub recorded_crc: String,
    pub required_revision: Option<String>,
    pub sku_path: Option<PathBuf>,
    pub providers: Vec<MapProvider>,
    pub warnings: Vec<String>,
    pub launch_plan: Option<LaunchPlan>,
}

impl Report {
    fn finish(mut self, status: Status, message: impl Into<String>) -> Self {
        self.status = status;
        self.message = message.into();
        self
    }
}

/// Replay M= paths have a three-hex-digit prefix (281, 283, 2c1, ...).
/// Preserve the full revision suffix: __24 and __24j are different map directories.
fn map_asset(raw: &str) -> Option<(String, Option<String>)> {
    let path = archive::normalize(raw);
    let path = if path
        .as_bytes()
        .get(..3)
        .is_some_and(|p| p.iter().all(u8::is_ascii_hexdigit))
    {
        path.get(3..)?
    } else {
        &path
    };
    if !path.starts_with("data/maps/official/")
        || path
            .split('/')
            .any(|s| s.is_empty() || s == ".." || s == ".")
    {
        return None;
    }
    let directory = path.trim_end_matches('/');
    let name = directory.rsplit('/').next()?;
    let revision = name
        .rsplit_once("__")
        .map(|(_, r)| r)
        .filter(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_alphanumeric()))
        .map(|r| format!("R{r}"));
    Some((format!("{directory}/{name}.map"), revision))
}

// These are the script bundles shipped by the community map installers.
// The stock Core/Misc.big and unidentified loose scripts must not satisfy
// this requirement merely because they export the same asset name.
fn community_script_archive(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    if name == "102scripts.big" {
        return true;
    }
    name.strip_prefix('r')
        .and_then(|n| n.strip_suffix("scripts.big"))
        .is_some_and(|revision| {
            revision.starts_with(|c: char| c.is_ascii_digit())
                && revision.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

pub fn check(target: &ReplayTarget, sku: Option<&Path>) -> Result<Report> {
    let requirement = map_asset(&target.map_path);
    let mut report = Report {
        replay_id: target.id,
        status: Status::Unknown,
        message: String::new(),
        map_name: target.map_name.clone(),
        map_path: target.map_path.clone(),
        recorded_crc: target.map_crc.clone(),
        required_revision: requirement.as_ref().and_then(|(_, r)| r.clone()),
        sku_path: sku.map(Path::to_path_buf),
        providers: Vec::new(),
        warnings: Vec::new(),
        launch_plan: None,
    };
    let Some(replay) = target
        .file_path
        .as_ref()
        .map(Path::new)
        .filter(|p| p.is_file())
    else {
        return Ok(report.finish(
            Status::ReplayMissing,
            "The replay file is missing. Import its current folder to update its location.",
        ));
    };
    let Some(sku) = sku else {
        return Ok(report.finish(
            Status::NotConfigured,
            "Choose your Kane's Wrath game configuration to check this replay.",
        ));
    };
    let game = match config::read(sku) {
        Ok(game) => game,
        Err(error) => {
            return Ok(report.finish(
                Status::Unknown,
                format!("Cannot read the game configuration: {error:#}"),
            ))
        }
    };
    report.sku_path = Some(game.sku.clone());
    report.warnings = game.warnings.clone();
    let engine_version = game
        .executable
        .parent()
        .and_then(Path::file_name)
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let required_engine = format!("{}.{}", target.version[0], target.version[1]);
    if engine_version != required_engine || target.version[2..] != [0, 0] {
        return Ok(report.finish(
            Status::EngineMismatch,
            format!(
                "This replay requires engine {}.{}.{}.{}. Choose its matching game configuration.",
                target.version[0], target.version[1], target.version[2], target.version[3]
            ),
        ));
    }
    let Some((asset, revision)) = requirement else {
        return Ok(report.finish(Status::Unknown, "This map path is not a supported official/map-pack path. Custom map compatibility needs verification."));
    };
    let community = asset.contains("1.02+");
    if community && revision.is_none() {
        return Ok(report.finish(Status::Unknown, "This community map has no explicit revision suffix. Its recorded CRC alone cannot establish compatibility."));
    }

    let enabled: HashSet<PathBuf> = game.archives.iter().cloned().collect();
    let mut archives = enabled.clone();
    // Also inspect installed-but-disabled packs. Read directory entries only.
    for entry in WalkDir::new(&game.root).max_depth(12) {
        match entry {
            Ok(entry)
                if entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("big")) =>
            {
                match dunce::canonicalize(entry.path()) {
                    Ok(path) => {
                        archives.insert(path);
                    }
                    Err(e) => report
                        .warnings
                        .push(format!("Could not inspect {}: {e}", entry.path().display())),
                }
            }
            Err(e) => report
                .warnings
                .push(format!("Could not scan installed content: {e}")),
            _ => {}
        }
    }
    let mut scripts_enabled = false;
    let mut active_archive_error = false;
    let mut archives: Vec<_> = archives.into_iter().collect();
    archives.sort();
    for path in archives {
        let active = enabled.contains(&path);
        match archive::entries(&path) {
            Ok(entries) => {
                if active
                    && community_script_archive(&path)
                    && entries.iter().any(|n| n == "data/scripts/scripts.lua")
                {
                    scripts_enabled = true;
                    if community {
                        report
                            .warnings
                            .push(format!("Community script provider: {}", path.display()));
                    }
                }
                if entries.iter().any(|n| n == &asset) {
                    report.providers.push(MapProvider {
                        path,
                        enabled: active,
                    });
                }
            }
            Err(e) => {
                active_archive_error |= active;
                report
                    .warnings
                    .push(format!("Could not index {}: {e:#}", path.display()));
            }
        }
    }
    for directory in &game.search_paths {
        let path = directory.join(&asset);
        if path.is_file() {
            report.providers.push(MapProvider {
                path,
                enabled: true,
            });
        }
    }
    let active_count = report.providers.iter().filter(|p| p.enabled).count();
    if active_archive_error {
        return Ok(report.finish(
            Status::Unknown,
            "An active content archive could not be read. Map compatibility cannot be established.",
        ));
    }
    if active_count > 1 {
        return Ok(report.finish(Status::Unknown, "Multiple active sources provide this exact map. Resolve the conflicting content before launching."));
    }
    if active_count == 0 {
        let revision_label = revision.map(|r| format!(" ({r})")).unwrap_or_default();
        if report.providers.is_empty() {
            return Ok(report.finish(Status::MapMissing, format!("The exact map{revision_label} is missing. Install its matching map pack, then check again.")));
        }
        return Ok(report.finish(Status::MapDisabled, format!("The exact map{revision_label} is installed but is not enabled by this game configuration.")));
    }
    if community && !scripts_enabled {
        return Ok(report.finish(Status::Unknown, "The map revision is available, but no enabled community script bundle was identified. Enable 102Scripts.big or the applicable R<revision>Scripts.big; stock or unidentified loose scripts cannot establish compatibility."));
    }
    report.warnings.push("Map availability is checked by exact asset path. The recorded CRC, script revision, and deterministic playback are not verified.".into());
    let replay = dunce::canonicalize(replay)?;
    report.launch_plan = Some(LaunchPlan {
        executable: game.executable,
        working_directory: game.root,
        args: vec![
            "-replayGame".into(),
            replay.to_string_lossy().into_owned(),
            "-win".into(),
            "-config".into(),
            game.sku.to_string_lossy().into_owned(),
        ],
    });
    Ok(report.finish(
        Status::Ready,
        "The exact map is enabled and the engine version matches. Ready to launch.",
    ))
}

/// Start the prepared replay with the user's inherited environment.
pub fn launch_command(plan: &LaunchPlan) -> Command {
    let mut command = Command::new(&plan.executable);
    command
        .args(&plan.args)
        .current_dir(&plan.working_directory);
    command
}

#[derive(Debug, Serialize)]
pub struct LaunchResult {
    pub pid: u32,
}

pub fn launch(target: &ReplayTarget, sku: Option<&Path>) -> Result<LaunchResult> {
    ensure!(cfg!(windows), "Replay launching is supported on Windows");
    // Serialize check/spawn so rapid double-clicks cannot start two engines.
    static LAUNCH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LAUNCH_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let report = check(target, sku)?;
    ensure!(report.status == Status::Ready, "{}", report.message);
    let plan = report.launch_plan.context("No launch plan")?;
    let (hash, _) = crate::ingest::hash_file(Path::new(&plan.args[1]))?;
    ensure!(
        hash == target.file_hash,
        "The replay file has changed since import. Import it again before playing."
    );
    ensure!(
        !game_running()?,
        "Kane's Wrath is already running. Close it before launching another replay."
    );
    let mut child = launch_command(&plan)
        .spawn()
        .context("Start Kane's Wrath")?;
    let pid = child.id();
    // Reap the child without closing it when Tacitus is closed.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(LaunchResult { pid })
}

#[cfg(not(windows))]
fn game_running() -> Result<bool> {
    anyhow::bail!("Replay launching is supported on Windows")
}

#[cfg(windows)]
fn game_running() -> Result<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    // Inspect process names without attaching to or controlling the game.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        ensure!(
            snapshot != INVALID_HANDLE_VALUE,
            "Cannot inspect running games: {}",
            std::io::Error::last_os_error()
        );
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut more = Process32FirstW(snapshot, &mut entry);
        while more != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]).to_ascii_lowercase();
            if matches!(name.as_str(), "cnc3ep1.dat" | "cnc3ep1.exe" | "cnc3ep1") {
                found = true;
                break;
            }
            more = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        Ok(found)
    }
}
