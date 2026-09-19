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
