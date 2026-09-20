//! Installed custom maps use SAGE's rotate/add file checksum (the replay MC).
//! The engine discovers these maps in its normal user Maps directory even
//! when a private stock-content SkuDef is selected.
use super::automatic::Control;
use anyhow::{ensure, Context, Result};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

fn checksum(path: &Path, control: &Control<'_>) -> Result<u32> {
    let mut file = File::open(path)?;
    ensure!(
        file.metadata()?.len() <= 512 * 1024 * 1024,
        "Custom map exceeds the supported size"
    );
    let mut buffer = [0; 128 * 1024];
    let mut crc = 0u32;
    loop {
        control.check()?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        for &byte in &buffer[..count] {
            crc = crc.rotate_left(1).wrapping_add(u32::from(byte));
        }
    }
    Ok(crc)
}

fn lookup(root: &Path, asset: &str, crc: u32, control: &Control<'_>) -> Result<Option<PathBuf>> {
    let relative = asset
        .strip_prefix("data/maps/internal/")
        .context("Invalid custom map namespace")?;
    let Some((directory, file)) = relative.split_once('/') else {
        return Ok(None);
    };
    ensure!(
        !directory.is_empty()
            && file == format!("{directory}.map")
            && !directory
                .chars()
                .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c))
            && directory != "."
            && directory != "..",
        "Invalid custom map path"
    );
    let path = root.join(directory).join(file);
    if !path.is_file() {
        return Ok(None);
    }
    let root = dunce::canonicalize(root)?;
    let path = dunce::canonicalize(path)?;
    ensure!(
        path.starts_with(root),
        "Custom map is outside the game's Maps directory"
    );
    if checksum(&path, control)? != crc {
        return Ok(None);
    }
    Ok(Some(path))
}

pub fn installed(asset: &str, crc: u32, control: &Control<'_>) -> Result<Option<PathBuf>> {
    let Some(roaming) = std::env::var_os("APPDATA") else {
        return Ok(None);
    };
    lookup(
        &PathBuf::from(roaming).join("Command & Conquer 3 Kane's Wrath/Maps"),
        asset,
        crc,
        control,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::automatic::Cancellation;
    #[test]
    fn installed_custom_map_requires_exact_directory_and_file_crc() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("example")).unwrap();
        std::fs::write(root.path().join("example/example.map"), b"ABC").unwrap();
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        let asset = "data/maps/internal/example/example.map";
        assert!(lookup(root.path(), asset, 459, &control).unwrap().is_some());
        assert!(lookup(root.path(), asset, 458, &control).unwrap().is_none());
        assert!(lookup(
            root.path(),
            &asset.replace("example", "missing"),
            459,
            &control
        )
        .unwrap()
        .is_none());
        assert!(lookup(root.path(), "data/maps/internal/../...map", 459, &control).is_err());
    }
}
