//! Persistent, verified pack storage and exact-asset dependency selection.
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::{
    archive,
    automatic::Control,
    config::GameConfig,
    selection::{map_identity, PackKind},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedArchive {
    pub name: String,
    pub sha256: String,
    pub size: u64,
    pub entries: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pack {
    pub format: u32,
    pub revision: String,
    pub source: String,
    pub package_sha256: String,
    pub archives: Vec<CachedArchive>,
}

pub struct Resolved {
    pub archives: Vec<PathBuf>,
    pub custom_map: Option<PathBuf>,
}

pub fn hash_file(path: &Path, control: &Control<'_>) -> Result<(String, u64)> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 256 * 1024];
    let mut size = 0;
    loop {
        control.check()?;
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        size += n as u64;
    }
    Ok((hex::encode(hash.finalize()), size))
}

fn manifest(path: &Path) -> Result<Pack> {
    let file = File::open(path.join("manifest.json"))?;
    ensure!(
        file.metadata()?.len() <= 16 * 1024 * 1024,
        "Cached pack index is too large"
    );
    let pack: Pack = serde_json::from_reader(file)?;
    ensure!(
        pack.format == 1 && !pack.archives.is_empty() && pack.archives.len() <= 128,
        "Invalid cached pack index"
    );
    for archive in &pack.archives {
        ensure!(
            !archive.name.is_empty()
                && !archive.name.contains(['/', '\\', ':'])
                && archive.name.to_ascii_lowercase().ends_with(".big"),
            "Invalid cached archive name"
        );
    }
    Ok(pack)
}

fn script_name(name: &str, revision: &str) -> bool {
    name.eq_ignore_ascii_case("102Scripts.big")
        || name.eq_ignore_ascii_case(&format!("{revision}Scripts.big"))
}

fn has_scripts(pack: &Pack) -> bool {
    super::sources::uses_base_scripts(&pack.revision, &pack.source, &pack.package_sha256)
        || pack.archives.iter().any(|a| {
            script_name(&a.name, &pack.revision)
                && a.entries.iter().any(|e| e == "data/scripts/scripts.lua")
        })
}

#[cfg(test)]
pub fn has_asset(directory: &Path, asset: &str) -> Result<bool> {
    Ok(manifest(directory)?
        .archives
        .iter()
        .any(|a| a.entries.iter().any(|entry| entry == asset)))
}

pub fn has_compatible_map(directory: &Path, asset: &str, crc: u32) -> Result<bool> {
    let mut found = false;
    for archive in manifest(directory)?.archives {
        if archive.entries.iter().any(|entry| entry == asset) {
            // Every mounted copy must agree: an earlier BIG can shadow a
            // later archive that contains the requested compatibility value.
            if !super::metadata::matches(&directory.join(archive.name), asset, crc)? {
                return Ok(false);
            }
            found = true;
        }
    }
    Ok(found)
}

pub fn cached(
    cache: &Path,
    asset: &str,
    crc: u32,
    verify: bool,
    control: &Control<'_>,
) -> Result<Option<Resolved>> {
    let packages = cache.join("packages");
    if !packages.is_dir() {
        return Ok(None);
    }
    let mut paths: Vec<_> = fs::read_dir(&packages)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    paths.sort();
    for directory in paths {
        control.check()?;
        let Ok(pack) = manifest(&directory) else {
            continue;
        };
        if !has_scripts(&pack)
            || !pack
                .archives
                .iter()
                .any(|a| a.entries.iter().any(|e| e == asset))
        {
            continue;
        }
        // Several releases deliberately reuse the same directory suffix.
        // Read the compiled metadata instead of trusting the cache label.
        if !has_compatible_map(&directory, asset, crc).unwrap_or(false) {
            continue;
        }
        let mut valid = true;
        for a in &pack.archives {
            let path = directory.join(&a.name);
            let Ok(meta) = fs::symlink_metadata(&path) else {
                valid = false;
                break;
            };
            if !meta.file_type().is_file() || meta.len() != a.size {
                valid = false;
                break;
            }
            if verify {
                control.stage("verifying", "Verifying the cached map pack…")?;
                if hash_file(&path, control)?.0 != a.sha256 {
                    valid = false;
                    break;
                }
            }
        }
        if valid {
            let mut archives: Vec<_> = pack
                .archives
                .iter()
                .map(|a| directory.join(&a.name))
                .collect();
            order(&mut archives, &pack.revision);
            return Ok(Some(Resolved {
                archives,
                custom_map: None,
            }));
        }
    }
    Ok(None)
}

pub fn contains_other_map(cache: &Path, source: &str, revision: &str, asset: &str) -> Result<bool> {
    let packages = cache.join("packages");
    if !packages.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(packages)? {
        let entry = entry?;
        if let Ok(pack) = manifest(&entry.path()) {
            if pack.source == source
                && pack.revision.eq_ignore_ascii_case(revision)
                && super::sources::complete_map_archives(
                    source,
                    revision,
                    &pack
                        .archives
                        .iter()
                        .map(|a| a.name.as_str())
                        .collect::<Vec<_>>(),
                )
                && !pack
                    .archives
                    .iter()
                    .any(|a| a.entries.iter().any(|e| e == asset))
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Reuse map directories from every cached/installed revision to select a
/// download category. These are lookup hints only; resolve still requires
/// the replay's exact asset, matching scripts and verified content.
pub fn pack_hints(
    game: &GameConfig,
    cache: &Path,
    asset: &str,
    control: &Control<'_>,
) -> Result<Vec<PackKind>> {
    control.check()?;
    let Some(identity) = map_identity(asset) else {
        return Ok(Vec::new());
    };
    let mut matches = BTreeMap::new();
    let mut inspect = |kind, entries: &[String]| {
        for entry in entries {
            let rank = if entry == asset {
                0
            } else if map_identity(entry) == Some(identity) {
                1
            } else {
                continue;
            };
            matches
                .entry(kind)
                .and_modify(|old: &mut u8| *old = (*old).min(rank))
                .or_insert(rank);
        }
    };
    let packages = cache.join("packages");
    if packages.is_dir() {
        for entry in fs::read_dir(packages)? {
            control.check()?;
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Ok(pack) = manifest(&entry.path()) {
                for archive in &pack.archives {
                    if let Some(kind) = PackKind::from_archive(&archive.name) {
                        inspect(kind, &archive.entries);
                    }
                }
            }
        }
    }
    for entry in WalkDir::new(&game.root).max_depth(6).follow_links(false) {
        control.check()?;
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if let Some(kind) = PackKind::from_archive(&entry.file_name().to_string_lossy()) {
            if let Ok(entries) = archive::entries(entry.path()) {
                inspect(kind, &entries);
            }
        }
    }
    let mut ranked: Vec<_> = matches
        .into_iter()
        .map(|(kind, rank)| (rank, kind))
        .collect();
    ranked.sort_unstable();
    Ok(ranked.into_iter().map(|(_, kind)| kind).collect())
}

pub fn resolve(
    game: &GameConfig,
    cache: &Path,
    asset: &str,
    revision: Option<&str>,
    crc: Option<u32>,
    verify: bool,
    control: &Control<'_>,
) -> Result<Option<Resolved>> {
    if asset.starts_with("data/maps/internal/") {
        return Ok(super::custom::installed(
            asset,
            crc.context("Custom map checksum is missing")?,
            control,
        )?
        .map(|path| Resolved {
            archives: Vec::new(),
            custom_map: Some(path),
        }));
    }
    let crc = crc.context("The replay map compatibility value is missing")?;
    if super::automatic::community_asset(asset) {
        if let Some(found) = cached(cache, asset, crc, verify, control)? {
            return Ok(Some(found));
        }
    } else {
        if super::metadata::stock_crc(&game.archives, asset)? != Some(crc) {
            return Ok(None);
        }
        for path in &game.archives {
            control.check()?;
            if archive::entries(path)?.iter().any(|e| e == asset) {
                return Ok(Some(Resolved {
                    archives: Vec::new(),
                    custom_map: None,
                }));
            }
        }
        return Ok(None);
    }
    let Some(revision) = revision else {
        return Ok(None);
    };
    // Installed packs can be borrowed without enabling them globally. An
    // unversioned 102Scripts.big alone cannot prove which patch it came from.
    let mut candidates = Vec::new();
    for entry in WalkDir::new(&game.root).max_depth(6).follow_links(false) {
        let entry = entry?;
        control.check()?;
        if !entry.file_type().is_file()
            || !entry
                .path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("big"))
        {
            continue;
        }
        if game.archives.iter().any(|p| p == entry.path()) {
            continue;
        }
        if let Ok(entries) = archive::entries(entry.path()) {
            if entries.iter().any(|e| e == asset)
                && super::metadata::matches(entry.path(), asset, crc).unwrap_or(false)
            {
                candidates.push(entry.into_path());
            }
        }
    }
    candidates.sort();
    for map in candidates {
        let parent = map
            .parent()
            .context("Map pack has no parent directory")?
            .to_path_buf();
        let scripts = fs::read_dir(&parent)?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&format!("{revision}Scripts.big"))
            })
            .map(|e| e.path());
        if let Some(scripts) = scripts.filter(|p| {
            archive::entries(p).is_ok_and(|e| e.iter().any(|n| n == "data/scripts/scripts.lua"))
        }) {
            let mut archives = vec![scripts, map];
            // This stock community texture dependency is optional when absent.
            for e in fs::read_dir(&parent)?.filter_map(|e| e.ok()) {
                if e.file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("102TextureFix.big")
                {
                    archive::entries(&e.path())?;
                    archives.push(e.path());
                }
            }
            order(&mut archives, revision);
            return Ok(Some(Resolved {
                archives,
                custom_map: None,
            }));
        }
    }
    Ok(None)
}

fn order(archives: &mut [PathBuf], revision: &str) {
    archives.sort_by_key(|p| {
        let name = p
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_ascii_lowercase();
        let priority = if name == format!("{}scripts.big", revision.to_ascii_lowercase()) {
            0
        } else if name == "102scripts.big" {
            1
        } else {
            2
        };
        (priority, name)
    });
}

/// Publish only a complete index, after every extracted archive has been read
/// and hashed. Incomplete staging directories are never candidates for use.
pub fn publish(
    staging: &Path,
    cache: &Path,
    revision: &str,
    source: &str,
    package_sha256: &str,
    control: &Control<'_>,
) -> Result<PathBuf> {
    let mut archives = Vec::new();
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        if !entry
            .path()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("big"))
        {
            continue;
        }
        ensure!(
            entry.file_type()?.is_file(),
            "A downloaded archive is not a regular file"
        );
        control.stage("verifying", "Verifying downloaded map files…")?;
        let entries = archive::entries(&entry.path())?;
        let (sha256, size) = hash_file(&entry.path(), control)?;
        archives.push(CachedArchive {
            name: entry.file_name().to_string_lossy().into_owned(),
            sha256,
            size,
            entries,
        });
    }
    ensure!(
        !archives.is_empty() && archives.len() <= 128,
        "The downloaded package contains no usable map archives."
    );
    let pack = Pack {
        format: 1,
        revision: revision.into(),
        source: source.into(),
        package_sha256: package_sha256.into(),
        archives,
    };
    ensure!(
        has_scripts(&pack),
        "The package does not contain its matching replay scripts."
    );
    fs::write(staging.join("manifest.json"), serde_json::to_vec(&pack)?)?;
    control.check()?;
    let packages = cache.join("packages");
    fs::create_dir_all(&packages)?;
    let slot = tempfile::Builder::new()
        .prefix("pack-")
        .tempdir_in(&packages)?;
    let destination = slot.path().to_path_buf();
    slot.close()?; // Remove only the empty, private reservation.
    fs::rename(staging, &destination)?; // Same-volume atomic publication.
    Ok(destination)
}
