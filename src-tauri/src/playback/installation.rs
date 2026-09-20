//! Game-folder selection and migration from the former configuration picker.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub game_path: Option<PathBuf>,
    // Read old settings without exposing their implementation detail to the UI.
    #[serde(default, skip_serializing)]
    pub sku_path: Option<PathBuf>,
}

pub fn settings_path() -> PathBuf {
    crate::ingest::default_catalogue_path().with_file_name("launcher.json")
}

pub fn load_settings() -> Result<Settings> {
    load_from(&settings_path(), detect)
}

fn load_from(path: &Path, detect: impl FnOnce() -> Option<PathBuf>) -> Result<Settings> {
    let mut settings: Settings = if path.is_file() {
        serde_json::from_slice(&fs::read(path)?).context("Read saved game location")?
    } else {
        Settings::default()
    };
    if settings.game_path.is_none() {
        settings.game_path = settings
            .sku_path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf);
    }
    if !settings.game_path.as_deref().is_some_and(is_installation) {
        settings.game_path = detect();
    }
    Ok(settings)
}

pub fn save_settings(directory: &Path) -> Result<Settings> {
    ensure!(
        is_installation(directory),
        "Choose the Kane's Wrath installation folder containing RetailExe and Core."
    );
    let settings = Settings {
        game_path: Some(dunce::canonicalize(directory)?),
        sku_path: None,
    };
    let path = settings_path();
    fs::create_dir_all(path.parent().context("Game settings have no directory")?)?;
    let mut pending = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
    serde_json::to_writer_pretty(pending.as_file_mut(), &settings)?;
    pending.as_file().sync_all()?;
    pending.persist(&path).map_err(|e| e.error)?;
    Ok(settings)
}

pub fn is_installation(root: &Path) -> bool {
    root.join("Core").is_dir()
        && ["1.2", "1.1", "1.0"].iter().any(|version| {
            root.join("RetailExe")
                .join(version)
                .join("cnc3ep1.dat")
                .is_file()
        })
}

fn detect() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let pattern = regex::Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
    for key in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(programs) = std::env::var_os(key).map(PathBuf::from) {
            let steam = programs.join("Steam");
            candidates.push(steam.join("steamapps/common/Command and Conquer 3 - Kane's Wrath"));
            // Steam libraries on other drives are declared in libraryfolders.vdf.
            if let Ok(text) = fs::read_to_string(steam.join("steamapps/libraryfolders.vdf")) {
                for entry in pattern.captures_iter(&text) {
                    candidates.push(
                        PathBuf::from(entry[1].replace("\\\\", "\\"))
                            .join("steamapps/common/Command and Conquer 3 - Kane's Wrath"),
                    );
                }
            }
            candidates.push(programs.join("EA Games/Command and Conquer 3 Kanes Wrath"));
            candidates.push(programs.join("Electronic Arts/Command & Conquer 3 Kane's Wrath"));
            candidates.push(programs.join("Origin Games/Command and Conquer 3 Kanes Wrath"));
        }
    }
    candidates
        .into_iter()
        .find(|p| is_installation(p))
        .and_then(|p| dunce::canonicalize(p).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_selection_without_rewriting_it_or_exposing_sku() {
        let root = tempfile::tempdir().unwrap();
        let game = root.path().join("KW");
        fs::create_dir_all(game.join("Core")).unwrap();
        fs::create_dir_all(game.join("RetailExe/1.2")).unwrap();
        fs::write(game.join("RetailExe/1.2/cnc3ep1.dat"), b"engine fixture").unwrap();
        let path = root.path().join("launcher.json");
        let old =
            serde_json::json!({"sku_path": game.join("CNC3EP1_english_1.2.SkuDef")}).to_string();
        fs::write(&path, &old).unwrap();
        let settings = load_from(&path, || panic!("saved installation should win")).unwrap();
        assert_eq!(settings.game_path, Some(game));
        assert!(!serde_json::to_string(&settings)
            .unwrap()
            .contains("sku_path"));
        assert_eq!(fs::read_to_string(&path).unwrap(), old);
        fs::write(&path, r#"{"game_path":"missing game"}"#).unwrap();
        assert_eq!(
            load_from(&path, || Some(root.path().into()))
                .unwrap()
                .game_path,
            Some(root.path().into())
        );
    }
}
