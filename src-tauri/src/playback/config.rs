//! Resolve the selected SkuDef's content chain. Relative directives are
//! relative to their containing config, including Core/1.2 -> ../1.1.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};

#[derive(Debug)]
pub struct GameConfig {
    pub sku: PathBuf,
    pub root: PathBuf,
    pub executable: PathBuf,
    pub archives: Vec<PathBuf>,
    pub search_paths: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

fn local_path(parent: &Path, value: &str) -> PathBuf {
    parent.join(value.replace('\\', "/"))
}

/// Build a stock content chain without reading the user's community-patch
/// selection. Only game-owned base/language config files are followed.
pub fn base(root: &Path, version: [u32; 4]) -> Result<GameConfig> {
    ensure!(
        version[0] == 1 && (version[1] <= 2) && version[2..] == [0, 0],
        "This replay requires an unsupported game engine version."
    );
    let root = dunce::canonicalize(root).context("The game folder is unavailable")?;
    let version = format!("{}.{}", version[0], version[1]);
    let executable = root.join("RetailExe").join(&version).join("cnc3ep1.dat");
    ensure!(
        executable.is_file(),
        "Game version {version} is not installed in this folder."
    );
    let mut languages: Vec<_> = fs::read_dir(&root)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            n.starts_with("Lang-") && root.join(n).join(&version).join("config.txt").is_file()
        })
        .collect();
    languages.sort_by_key(|n| (n != "Lang-english", n.clone()));
    let language = languages
        .first()
        .context("The game's language files are missing. Repair the game installation.")?;
    let audio_name = format!("{}Audio", language.trim_start_matches("Lang-"));
    let audio = fs::read_dir(&root)?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&audio_name)
        })
        .map(|e| e.file_name())
        .context("The game's audio files are missing.")?;
    let layers = [
        root.join(language).join(&version),
        root.join(audio).join(&version),
        root.join("Core").join(&version),
        root.join("Meta").join(&version),
        root.join("RetailExe").join(&version),
        root.join("Movies/1.0"),
    ];
    let mut config = GameConfig {
        sku: PathBuf::new(),
        root,
        executable,
        archives: Vec::new(),
        search_paths: Vec::new(),
        warnings: Vec::new(),
    };
    let mut active = HashSet::new();
    let mut visited = HashSet::new();
    for layer in layers {
        walk(
            &layer.join("config.txt"),
            &mut config,
            &mut active,
            &mut visited,
            0,
        )
        .context("The game's base files could not be loaded. Repair the game installation.")?;
    }
    ensure!(
        !config.archives.is_empty(),
        "The game's base content archives are missing."
    );
    Ok(config)
}

pub fn read(sku: &Path) -> Result<GameConfig> {
    let sku = dunce::canonicalize(sku).context("Choose an existing game .SkuDef configuration")?;
    ensure!(
        sku.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("skudef")),
        "Choose a .SkuDef file"
    );
    let mut config = GameConfig {
        root: sku
            .parent()
            .context("Configuration has no parent directory")?
            .to_path_buf(),
        sku: sku.clone(),
        executable: PathBuf::new(),
        archives: Vec::new(),
        search_paths: Vec::new(),
        warnings: Vec::new(),
    };
    walk(
        &sku,
        &mut config,
        &mut HashSet::new(),
        &mut HashSet::new(),
        0,
    )?;
    ensure!(
        !config.executable.as_os_str().is_empty(),
        "Configuration has no set-exe directive"
    );
    config.executable = dunce::canonicalize(&config.executable)
        .context("The engine selected by this configuration is missing")?;
    ensure!(config.executable.is_file(), "Selected engine is not a file");
    ensure!(
        config
            .executable
            .file_name()
            .is_some_and(|s| s.eq_ignore_ascii_case("cnc3ep1.dat")),
        "Configuration must select Kane's Wrath's cnc3ep1.dat engine"
    );
    Ok(config)
}

fn walk(
    path: &Path,
    config: &mut GameConfig,
    active: &mut HashSet<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<()> {
    ensure!(
        depth <= 32 && visited.len() < 1024,
        "Configuration includes exceed the supported limit"
    );
    let path = dunce::canonicalize(path)?;
    ensure!(
        !active.contains(&path),
        "Configuration include cycle at {}",
        path.display()
    );
    if !visited.insert(path.clone()) {
        return Ok(());
    }
    active.insert(path.clone());
    ensure!(
        fs::metadata(&path)?.len() <= 1024 * 1024,
        "Configuration exceeds 1 MiB"
    );
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Read configuration {}", path.display()))?;
    let parent = path.parent().context("Configuration has no parent")?;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with("//")
        {
            continue;
        }
        let (key, value) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let value = value.trim().trim_matches('"');
        match key.to_ascii_lowercase().as_str() {
            "set-exe" => {
                ensure!(!value.is_empty(), "Empty set-exe directive");
                ensure!(
                    config.executable.as_os_str().is_empty(),
                    "Multiple set-exe directives are ambiguous"
                );
                config.executable = local_path(parent, value);
            }
            "add-config" => {
                ensure!(!value.is_empty(), "Empty add-config directive");
                let child = local_path(parent, value);
                if child.is_file() {
                    walk(&child, config, active, visited, depth + 1)?;
                } else {
                    config.warnings.push(format!(
                        "Configured content file is missing: {}",
                        child.display()
                    ));
                }
            }
            "add-big" => {
                ensure!(!value.is_empty(), "Empty add-big directive");
                let archive = local_path(parent, value);
                if archive.is_file() {
                    let archive = dunce::canonicalize(archive)?;
                    if !config.archives.contains(&archive) {
                        config.archives.push(archive);
                    }
                } else {
                    config.warnings.push(format!(
                        "Configured archive is missing: {}",
                        archive.display()
                    ));
                }
            }
            "add-search-path" => {
                if value.eq_ignore_ascii_case("big:") {
                    continue;
                }
                if value.contains(':') && !Path::new(value).is_absolute() {
                    bail!("Unsupported search path: {value}");
                }
                let directory = local_path(parent, value);
                if directory.is_dir() {
                    config.search_paths.push(dunce::canonicalize(directory)?);
                }
            }
            _ => bail!(
                "Unsupported configuration directive {key:?} in {}",
                path.display()
            ),
        }
    }
    active.remove(&path);
    Ok(())
}
