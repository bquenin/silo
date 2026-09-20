//! First-launch preparation for the single-file Windows distribution.
//! Publish a runtime only after extraction completes, and serialize concurrent launches.

use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const REQUIRED_FILES: &[&str] = &[
    "msedgewebview2.exe",
    "msedge.dll",
    "msedge_elf.dll",
    "icudtl.dat",
];
const COMPLETE: &str = ".tacitus-complete";

#[cfg(feature = "portable")]
pub fn prepare_embedded() -> Result<PathBuf> {
    const CAB: &[u8] = include_bytes!(env!("TACITUS_WEBVIEW2_CAB"));
    const DIRECTORY: &str = env!("TACITUS_WEBVIEW2_DIRECTORY");
    const SHA256: &str = env!("TACITUS_WEBVIEW2_SHA256");
    let root = std::env::var_os("LOCALAPPDATA")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .context("Windows did not provide a local application data folder")?
        .join("tacitus")
        .join("runtimes");
    ensure_runtime(&root, DIRECTORY, SHA256, |staging| {
        let cab = staging.join("webview2.cab");
        fs::write(&cab, CAB).context("Could not write the embedded WebView2 runtime")?;
        run_windows_tool(
            "expand.exe",
            &[cab.as_os_str(), "-F:*".as_ref(), staging.as_os_str()],
        )
        .context("Could not unpack the embedded WebView2 runtime")?;
        // Required for unpackaged Fixed Version >= 120 on Windows 10. The current
        // user owns this directory, so granting the renderer read access needs no elevation.
        let unpacked = staging.join(DIRECTORY);
        run_windows_tool(
            "icacls.exe",
            &[
                unpacked.as_os_str(),
                "/grant".as_ref(),
                "*S-1-15-2-2:(OI)(CI)(RX)".as_ref(),
                "*S-1-15-2-1:(OI)(CI)(RX)".as_ref(),
                "/T".as_ref(),
                "/Q".as_ref(),
            ],
        )
        .context("Could not prepare WebView2 file permissions")?;
        Ok(())
    })
}

fn ensure_runtime(
    root: &Path,
    directory: &str,
    sha256: &str,
    install: impl FnOnce(&Path) -> Result<()>,
) -> Result<PathBuf> {
    fs::create_dir_all(root).context("Could not create the runtime cache")?;
    let runtime = root.join(format!("{directory}-{sha256}"));
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("prepare.lock"))
        .context("Could not open the runtime cache lock")?;
    lock.lock().context("Could not lock the runtime cache")?;
    if fs::read_to_string(runtime.join(COMPLETE)).ok().as_deref() == Some(sha256)
        && validate_runtime(&runtime).is_ok()
    {
        return Ok(runtime);
    }

    let staging = tempfile::Builder::new()
        .prefix(".prepare-")
        .tempdir_in(root)
        .context("Could not create a temporary runtime folder")?;
    install(staging.path())?;
    let unpacked = staging.path().join(directory);
    validate_runtime(&unpacked)?;
    fs::write(unpacked.join(COMPLETE), sha256)?;
    if runtime.exists() {
        // This path is constructed only from the compile-time runtime version/hash;
        // it cannot point at the catalogue, map cache, or any user-selected directory.
        fs::remove_dir_all(&runtime).context("Could not replace the incomplete runtime cache")?;
    }
    fs::rename(&unpacked, &runtime).context("Could not publish the prepared runtime")?;
    Ok(runtime)
}

fn validate_runtime(runtime: &Path) -> Result<()> {
    for file in REQUIRED_FILES {
        let path = runtime.join(file);
        let metadata = fs::metadata(&path)
            .with_context(|| format!("The embedded runtime is missing {}", path.display()))?;
        anyhow::ensure!(
            metadata.is_file() && metadata.len() > 0,
            "The embedded runtime contains an invalid {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(feature = "portable")]
fn run_windows_tool(name: &str, args: &[&std::ffi::OsStr]) -> Result<()> {
    use std::os::windows::process::CommandExt;
    let system = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .context("Windows did not provide its system folder")?
        .join("System32");
    let output = std::process::Command::new(system.join(name))
        .args(args)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output()
        .with_context(|| format!("Could not start {name}"))?;
    anyhow::ensure!(
        output.status.success(),
        "{name} failed ({}): {} {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[cfg(feature = "portable")]
pub fn show_error(error: &anyhow::Error) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let message: Vec<u16> = format!("Tacitus could not prepare its bundled browser.\n\n{error:#}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let title: Vec<u16> = "Tacitus".encode_utf16().chain(Some(0)).collect();
    // Both buffers are null-terminated and live until this synchronous call returns.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DIRECTORY: &str = "test-runtime";
    const HASH: &str = "test-hash";

    fn install(staging: &Path) -> Result<()> {
        let runtime = staging.join(DIRECTORY);
        fs::create_dir(&runtime)?;
        for file in REQUIRED_FILES {
            fs::write(runtime.join(file), b"runtime fixture")?;
        }
        Ok(())
    }

    #[test]
    fn complete_runtime_is_reused_without_extraction() -> Result<()> {
        let root = tempfile::tempdir()?;
        let first = ensure_runtime(root.path(), DIRECTORY, HASH, install)?;
        let again = ensure_runtime(root.path(), DIRECTORY, HASH, |_| {
            panic!("unnecessary extraction")
        })?;
        assert_eq!(first, again);
        assert_eq!(fs::read_to_string(first.join(COMPLETE))?, HASH);
        Ok(())
    }

    #[test]
    fn interrupted_or_invalid_extraction_is_not_published_and_can_retry() -> Result<()> {
        let root = tempfile::tempdir()?;
        let failed = ensure_runtime(root.path(), DIRECTORY, HASH, |staging| {
            install(staging)?;
            anyhow::bail!("interrupted")
        });
        assert!(failed.is_err());
        assert!(!root.path().join(format!("{DIRECTORY}-{HASH}")).exists());
        let invalid = ensure_runtime(root.path(), DIRECTORY, HASH, |staging| {
            install(staging)?;
            fs::write(staging.join(DIRECTORY).join("msedge.dll"), [])?;
            Ok(())
        });
        assert!(invalid.is_err());
        assert!(!root.path().join(format!("{DIRECTORY}-{HASH}")).exists());
        ensure_runtime(root.path(), DIRECTORY, HASH, install)?;
        Ok(())
    }

    #[test]
    fn missing_runtime_files_are_repaired_without_touching_other_data() -> Result<()> {
        let root = tempfile::tempdir()?;
        let runtime = ensure_runtime(root.path(), DIRECTORY, HASH, install)?;
        fs::write(root.path().join("keep-me"), b"unrelated data")?;
        fs::remove_file(runtime.join("msedge.dll"))?;
        let repaired = ensure_runtime(root.path(), DIRECTORY, HASH, install)?;
        validate_runtime(&repaired)?;
        assert_eq!(fs::read(root.path().join("keep-me"))?, b"unrelated data");
        Ok(())
    }

    #[test]
    fn concurrent_launches_extract_only_once() -> Result<()> {
        let root = tempfile::tempdir()?;
        let installs = std::sync::atomic::AtomicUsize::new(0);
        std::thread::scope(|scope| {
            let jobs: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        ensure_runtime(root.path(), DIRECTORY, HASH, |staging| {
                            installs.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            install(staging)
                        })
                        .unwrap()
                    })
                })
                .collect();
            for job in jobs {
                job.join().unwrap();
            }
        });
        assert_eq!(installs.load(std::sync::atomic::Ordering::SeqCst), 1);
        Ok(())
    }
}
