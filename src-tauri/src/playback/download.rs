//! Download exact historical packs. Installer payloads are unpacked as data;
//! downloaded executables are never run.
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use reqwest::Url;
use sha2::{Digest, Sha256};

use super::{
    automatic::{Control, Progress},
    content, sources,
};

const MAX_PACKAGE_BYTES: u64 = 5 * 1024 * 1024 * 1024;
const MAX_PAGE_BYTES: usize = 4 * 1024 * 1024;

pub fn supported(revision: &str) -> bool {
    sources::available(revision)
        || Regex::new(r"(?i)^R(21|22|23|24|25)[a-z]?$")
            .unwrap()
            .is_match(revision)
}

/// Fail closed if the provider changes its version-list markup. Exact labels
/// select a historical version; a latest-version link is never a substitute.
fn version_url(html: &str, revision: &str) -> Result<Option<Url>> {
    let row = Regex::new(r#"(?s)<li\b[^>]*class="[^"]*\byh_version-item\b[^"]*"[^>]*>(.*?)</li>"#)?;
    let filename = Regex::new(r#"data-full="([^"]+)""#)?;
    let expected_name = Regex::new(&format!(
        r"(?i)^{}[-_](1vs1|2vs2|4vs4|4v4|legacy|all-in-one)[-_]map[-_]pack(?:-[1-9][0-9]*)?\.zip$",
        regex::escape(revision)
    ))?;
    let link = Regex::new(r#"href="([^"]*yh_download_id=[^"]+)""#)?;
    for row in row.captures_iter(html) {
        let text = &row[1];
        let Some(name) = filename.captures(text) else {
            continue;
        };
        // A same-revision 1.03 engine variant is not interchangeable with
        // the ordinary pack. Require the whole known package name.
        if !expected_name.is_match(&name[1]) {
            continue;
        }
        ensure!(
            !text.contains("data-is-password-protected=\"1\"")
                && !text.contains("data-is-test-version=\"1\""),
            "This map revision is not a public download."
        );
        let Some(link) = link.captures(text) else {
            continue;
        };
        let url = Url::parse(
            &link[1]
                .replace("&#038;", "&")
                .replace("&#38;", "&")
                .replace("&amp;", "&"),
        )?;
        ensure!(
            url.scheme() == "https"
                && url.host_str() == Some("kaneswrath.com")
                && url
                    .query_pairs()
                    .any(|(k, v)| k == "attachment_id" && v.chars().all(|c| c.is_ascii_digit())),
            "The map provider returned an unexpected download address."
        );
        return Ok(Some(url));
    }
    Ok(None)
}

async fn cancelled(control: &Control<'_>) {
    while control.check().is_ok() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent("Tacitus/0.1 (replay content cache)")
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(1800))
        .build()?)
}

async fn page(
    client: &reqwest::Client,
    url: &str,
    control: &Control<'_>,
) -> Result<Option<String>> {
    let response = tokio::select! {
        response = client.get(url).send() => response?,
        _ = cancelled(control) => bail!("Replay preparation cancelled."),
    };
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let mut response = response.error_for_status()?;
    let mut bytes = Vec::new();
    loop {
        let chunk = tokio::select! {
            chunk = response.chunk() => chunk?,
            _ = cancelled(control) => bail!("Replay preparation cancelled."),
        };
        let Some(chunk) = chunk else { break };
        ensure!(
            bytes.len() + chunk.len() <= MAX_PAGE_BYTES,
            "The map catalogue page exceeds the supported size."
        );
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(String::from_utf8(bytes)?))
}

async fn package(
    client: &reqwest::Client,
    url: &Url,
    referer: &str,
    path: &Path,
    revision: &str,
    label: &str,
    control: &Control<'_>,
) -> Result<String> {
    let response = tokio::select! {
        response = client.get(url.clone()).header(reqwest::header::REFERER, referer).send() => response?,
        _ = cancelled(control) => bail!("Replay preparation cancelled."),
    };
    let mut response = response.error_for_status()?;
    let total = response.content_length();
    ensure!(
        total.is_none_or(|n| n <= MAX_PACKAGE_BYTES),
        "The map package exceeds the supported download size."
    );
    let mut file = File::create(path)?;
    let mut hash = Sha256::new();
    let mut downloaded = 0;
    let mut last_update = std::time::Instant::now() - Duration::from_secs(1);
    loop {
        let chunk = tokio::select! {
            chunk = response.chunk() => chunk?,
            _ = cancelled(control) => bail!("Replay preparation cancelled."),
        };
        let Some(chunk) = chunk else { break };
        downloaded += chunk.len() as u64;
        ensure!(
            downloaded <= MAX_PACKAGE_BYTES,
            "The map package exceeds the supported download size."
        );
        file.write_all(&chunk)?;
        hash.update(&chunk);
        if last_update.elapsed() >= Duration::from_millis(150) {
            (control.progress)(Progress {
                phase: "downloading",
                message: format!("Downloading the {revision} {label}…"),
                completed: downloaded,
                total,
            });
            last_update = std::time::Instant::now();
        }
    }
    ensure!(
        downloaded > 0 && total.is_none_or(|n| n == downloaded),
        "The map download was incomplete. Press Play to try again."
    );
    file.sync_all()?;
    control.check()?;
    Ok(hex::encode(hash.finalize()))
}

fn lock_cache(cache: &Path) -> Result<File> {
    fs::create_dir_all(cache)?;
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(cache.join("download.lock"))?;
    lock.try_lock().context("Another Tacitus instance may be preparing content. Wait for it to finish, then press Play.")?;
    Ok(lock)
}

/// Called with the cache's OS lock held. A terminated process releases that
/// lock, so only abandoned, explicitly owned staging folders can be removed.
fn cleanup_staging(stages: &Path) -> Result<()> {
    let root = dunce::canonicalize(stages)?;
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            || !entry.file_name().to_string_lossy().starts_with("download-")
        {
            continue;
        }
        let path = dunce::canonicalize(entry.path())?;
        if path.parent() == Some(root.as_path())
            && fs::read(path.join("tacitus-download")).ok().as_deref() == Some(b"1\n")
        {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

/// Import an archive obtained through a managed source, such as Command Post.
/// The caller supplies the verified transfer digest and the replay's exact
/// requirement. Keep the original package and never execute its installer.
pub fn import_package(
    cache: &Path,
    package: &Path,
    asset: &str,
    revision: &str,
    crc: u32,
    source: &str,
    expected_sha256: &str,
    control: &Control<'_>,
) -> Result<PathBuf> {
    ensure!(
        expected_sha256.len() == 64 && expected_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "A verified SHA-256 digest is required to import a map package."
    );
    let source_url = Url::parse(source)?;
    ensure!(
        source_url.scheme() == "https"
            && source_url.username().is_empty()
            && source_url.password().is_none()
            && source_url.query_pairs().all(|(key, _)| {
                ![
                    "token",
                    "access_token",
                    "password",
                    "username",
                    "authorization",
                    "signature",
                ]
                .contains(&key.to_ascii_lowercase().as_str())
                    && !key.to_ascii_lowercase().starts_with("x-amz-")
            }),
        "Use the shareable HTTPS source URL, without download credentials."
    );
    let _cache_lock = lock_cache(cache)?;
    let stages = cache.join("staging");
    fs::create_dir_all(&stages)?;
    cleanup_staging(&stages)?;
    let stage = tempfile::Builder::new()
        .prefix("download-")
        .tempdir_in(&stages)?;
    fs::write(stage.path().join("tacitus-download"), "1\n")?;
    control.stage("verifying", "Verifying the supplied map package…")?;
    ensure!(
        fs::metadata(package)?.len() <= MAX_PACKAGE_BYTES,
        "The map package exceeds the supported size."
    );
    let hash = content::hash_file(package, control)?.0;
    ensure!(
        hash.eq_ignore_ascii_case(expected_sha256),
        "The supplied map package does not match its verified SHA-256 digest."
    );
    let output = super::package::unpack(
        package,
        stage.path(),
        revision,
        "supplied map pack",
        control,
    )?;
    // Check the BIG index before publication; a similarly named map or another
    // revision cannot turn a failed import into a persistent cache entry.
    let mut found = false;
    for entry in fs::read_dir(&output)? {
        let path = entry?.path();
        if super::archive::entries(&path)?
            .iter()
            .any(|entry| entry == asset)
        {
            ensure!(
                super::metadata::matches(&path, asset, crc)?,
                "The supplied package has a conflicting map compatibility value."
            );
            found = true;
        }
    }
    ensure!(
        found,
        "The supplied package does not contain the replay's exact map asset and compatibility value."
    );
    content::publish(&output, cache, revision, source, &hash, control)
}

pub fn acquire(
    cache: &Path,
    asset: &str,
    revision: Option<&str>,
    crc: u32,
    sources: &[String],
    control: &Control<'_>,
) -> Result<()> {
    ensure!(
        sources::supports_compatibility(revision, crc) || revision.is_some_and(supported),
        "Tacitus has no verified automatic download source for this map's compatibility value {crc:X}."
    );
    control.check()?;
    let _cache_lock = lock_cache(cache)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let client = client()?;
    let stages = cache.join("staging");
    fs::create_dir_all(&stages)?;
    cleanup_staging(&stages)?;
    let mut failures = Vec::new();
    for source_page in sources {
        let label = super::selection::pack_label(source_page);
        control.stage("locating", "Finding the exact map pack for this replay…")?;
        // Preserve category ordering (a 1v1 pack before a multi-player pack),
        // while preferring Command Post's verified links within each category.
        if let Some(kind) = super::selection::PackKind::from_page(source_page) {
            for candidate in sources::compatible(revision, crc, kind) {
                match acquire_from(
                    cache, &stages, asset, crc, label, &candidate, &runtime, &client, control,
                ) {
                    Ok(true) => return Ok(()),
                    // Registry mirrors can contain the wrong pack despite
                    // their label. Try the remaining exact-version sources.
                    Ok(false) => {}
                    Err(error) => {
                        control.check()?;
                        failures.push(format!("{}: {error:#}", candidate.source));
                    }
                }
            }
        }
        // Unversioned community maps are selected through their registry code
        // and exact compiled metadata, never the latest website download.
        let Some(revision) = revision else { continue };
        let html = match runtime.block_on(page(&client, source_page, control)) {
            Ok(Some(html)) => html,
            Ok(None) => continue,
            Err(error) => {
                control.check()?;
                failures.push(format!("{source_page}: {error}"));
                continue;
            }
        };
        let url = match version_url(&html, revision) {
            Ok(Some(url)) => url,
            Ok(None) => continue,
            Err(error) => {
                failures.push(format!("{source_page}: {error:#}"));
                continue;
            }
        };
        let candidate = sources::Candidate {
            source: url.to_string(),
            url,
            revision: revision.into(),
            sha256: None,
        };
        match acquire_from(
            cache, &stages, asset, crc, label, &candidate, &runtime, &client, control,
        ) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => {
                control.check()?;
                failures.push(format!("{source_page}: {error:#}"));
            }
        }
    }
    control.check()?;
    if !failures.is_empty() {
        bail!(
            "The exact map pack could not be prepared from the available sources. Press Play to retry. {}",
            failures.join("; ")
        );
    }
    bail!("The exact map and its matching scripts are not available from the supported download sources (compatibility {crc:X}).")
}

fn acquire_from(
    cache: &Path,
    stages: &Path,
    asset: &str,
    crc: u32,
    label: &str,
    candidate: &sources::Candidate,
    runtime: &tokio::runtime::Runtime,
    client: &reqwest::Client,
    control: &Control<'_>,
) -> Result<bool> {
    let url = &candidate.url;
    let revision = &candidate.revision;
    if content::contains_other_map(cache, &candidate.source, revision, asset)? {
        return Ok(false);
    }
    let stage = tempfile::Builder::new()
        .prefix("download-")
        .tempdir_in(stages)?;
    fs::write(stage.path().join("tacitus-download"), "1\n")?;
    // Keep a complete, hashed download if extraction fails, so
    // retrying preparation does not require downloading it again.
    let downloads = cache.join("downloads");
    fs::create_dir_all(&downloads)?;
    let key = hex::encode(Sha256::digest(url.as_str().as_bytes()));
    let package_path = downloads.join(format!("{key}.zip"));
    let checksum_path = downloads.join(format!("{key}.sha256"));
    let previous = fs::read_to_string(&checksum_path)
        .ok()
        .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()));
    let cached_hash = if package_path.is_file() && previous.is_some() {
        control.stage("verifying", "Verifying the previously downloaded pack…")?;
        let actual = content::hash_file(&package_path, control)?.0;
        previous.filter(|hash| {
            *hash == actual
                && candidate
                    .sha256
                    .as_ref()
                    .is_none_or(|expected| expected.eq_ignore_ascii_case(hash))
        })
    } else {
        None
    };
    let hash = if let Some(hash) = cached_hash {
        hash
    } else {
        control.stage(
            "downloading",
            format!("Downloading the {revision} {label}…"),
        )?;
        let pending = stage.path().join("package.part");
        let hash = runtime
            .block_on(package(
                client,
                url,
                &candidate.source,
                &pending,
                revision,
                label,
                control,
            ))
            .context("The required map pack could not be downloaded. Press Play to try again.")?;
        ensure!(
            candidate
                .sha256
                .as_ref()
                .is_none_or(|expected| expected.eq_ignore_ascii_case(&hash)),
            "The downloaded package does not match the verified source SHA-256 digest."
        );
        super::package::validate_container(&pending)
            .context("The map provider returned an invalid package. Press Play to retry.")?;
        if package_path.is_file() {
            fs::remove_file(&package_path)?;
        }
        fs::rename(pending, &package_path)?;
        fs::write(&checksum_path, &hash)?;
        hash
    };
    let output = super::package::unpack(&package_path, stage.path(), revision, label, control)?;
    let published = content::publish(&output, cache, revision, &candidate.source, &hash, control)?;
    // Extracted content is the persistent cache; the compressed copy is
    // no longer needed after successful verification and publication.
    fs::remove_file(package_path)?;
    fs::remove_file(checksum_path)?;
    // Inspect this newly verified package, not an older corrupt cache
    // entry that happens to claim the requested map in its manifest.
    content::has_compatible_map(&published, asset, crc)
}

#[cfg(test)]
mod tests {
    use super::super::automatic::Cancellation;
    use super::*;

    #[test]
    fn cache_lock_prevents_overlapping_downloads_and_recovers_only_owned_staging() {
        let root = tempfile::tempdir().unwrap();
        let lock = lock_cache(root.path()).unwrap();
        assert!(lock_cache(root.path()).is_err());
        let stages = root.path().join("staging");
        let abandoned = stages.join("download-owned");
        let unrelated = stages.join("download-unowned");
        fs::create_dir_all(&abandoned).unwrap();
        fs::create_dir_all(&unrelated).unwrap();
        fs::write(abandoned.join("tacitus-download"), "1\n").unwrap();
        fs::write(abandoned.join("partial.dat"), "partial").unwrap();
        fs::write(unrelated.join("keep.dat"), "keep").unwrap();
        drop(lock);
        let _next_attempt = lock_cache(root.path()).unwrap();
        cleanup_staging(&stages).unwrap();
        assert!(!abandoned.exists());
        assert!(unrelated.join("keep.dat").is_file());
    }

    fn serve(response: Vec<u8>) -> (Url, std::thread::JoinHandle<()>) {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!("http://{}/pack", listener.local_addr().unwrap())).unwrap();
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).unwrap() > 0);
            let _ = stream.write_all(&response);
        });
        (url, thread)
    }

    #[test]
    fn pinned_source_rejects_changed_download_before_retaining_or_extracting_it() {
        let root = tempfile::tempdir().unwrap();
        let stages = root.path().join("staging");
        fs::create_dir(&stages).unwrap();
        let body = b"MZchanged installer payload";
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend(body);
        let (url, server) = serve(response);
        let candidate = sources::Candidate {
            source: url.to_string(),
            url,
            revision: "R12d".into(),
            sha256: Some("0".repeat(64)),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let cancelled = Cancellation::default();
        let control = Control {
            cancelled: &cancelled,
            progress: &|_| {},
        };
        let result = acquire_from(
            root.path(),
            &stages,
            "data/maps/official/example/example.map",
            0x1a,
            "map pack",
            &candidate,
            &runtime,
            &client,
            &control,
        );
        server.join().unwrap();
        assert!(result.unwrap_err().to_string().contains("SHA-256"));
        assert_eq!(
            fs::read_dir(root.path().join("downloads")).unwrap().count(),
            0
        );
        assert_eq!(fs::read_dir(stages).unwrap().count(), 0);
        assert!(!root.path().join("packages").exists());
    }

    #[test]
    fn downloads_reject_truncated_bodies_and_cancel_after_progress() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let root = tempfile::tempdir().unwrap();
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        let (url, server) = serve(
            b"HTTP/1.1 200 OK\r\nContent-Length: 200\r\nConnection: close\r\n\r\npartial".to_vec(),
        );
        assert!(runtime
            .block_on(package(
                &client,
                &url,
                "",
                &root.path().join("partial"),
                "R24g",
                "1v1 map pack",
                &control
            ))
            .is_err());
        server.join().unwrap();
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\ncontent".to_vec();
        let (url, server) = serve(response.clone());
        let hash = runtime
            .block_on(package(
                &client,
                &url,
                "",
                &root.path().join("complete"),
                "R24g",
                "1v1 map pack",
                &control,
            ))
            .unwrap();
        assert_eq!(hash, hex::encode(Sha256::digest(b"content")));
        server.join().unwrap();
        let (url, server) = serve(response);
        let progress = |_| {
            cancel.cancel();
        };
        let control = Control {
            cancelled: &cancel,
            progress: &progress,
        };
        assert!(runtime
            .block_on(package(
                &client,
                &url,
                "",
                &root.path().join("cancelled"),
                "R24g",
                "1v1 map pack",
                &control
            ))
            .unwrap_err()
            .to_string()
            .contains("cancelled"));
        server.join().unwrap();
        assert!(!root.path().join("packages").exists());
    }

    #[test]
    fn cancellation_interrupts_waiting_for_http_headers() {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!("http://{}/pack", listener.local_addr().unwrap())).unwrap();
        let cancel = std::sync::Arc::new(Cancellation::default());
        let server_cancel = cancel.clone();
        let (done, wait) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).unwrap() > 0);
            server_cancel.cancel();
            let _ = wait.recv_timeout(Duration::from_secs(5));
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        let before = std::time::Instant::now();
        let result = runtime.block_on(page(&client, url.as_str(), &control));
        done.send(()).unwrap();
        server.join().unwrap();
        assert!(result.unwrap_err().to_string().contains("cancelled"));
        assert!(before.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn picks_only_the_exact_historical_version() {
        assert!(supported("R24g"));
        assert!(!supported("R24g/../../"));
        let html = r#"<li class="yh_version-item"><span data-full="R24j-1vs1-Map-Pack.zip"></span><a href="https://kaneswrath.com/?yh_download_id=1&#038;attachment_id=2">Download</a></li>"#;
        assert_eq!(
            version_url(html, "R24j").unwrap().unwrap().as_str(),
            "https://kaneswrath.com/?yh_download_id=1&attachment_id=2"
        );
        assert!(version_url(html, "R24").unwrap().is_none());
        assert!(version_url(html, "R24g").unwrap().is_none());
        // WordPress adds a numeric suffix when an uploaded filename already
        // exists. R23f and R24 1v1 releases use this form.
        assert!(
            version_url(&html.replace("Map-Pack.zip", "Map-Pack-1.zip"), "R24j")
                .unwrap()
                .is_some()
        );
        assert!(
            version_url(&html.replace("Map-Pack.zip", "Map-Pack-12.zip"), "R24j")
                .unwrap()
                .is_some()
        );
        assert!(
            version_url(&html.replace("Map-Pack.zip", "Map-Pack-1.zip"), "R24")
                .unwrap()
                .is_none()
        );
        assert!(
            version_url(&html.replace("Map-Pack.zip", "Map-Pack-1.03.zip"), "R24j")
                .unwrap()
                .is_none()
        );
        assert!(version_url(&html.replace("kaneswrath.com", "example.org"), "R24j").is_err());
        let historical = html.replace("R24j-1vs1-Map-Pack", "R21h_1vs1_Map_Pack");
        assert!(version_url(&historical, "R21h").unwrap().is_some());
        assert!(version_url(&historical, "R21").unwrap().is_none());
        assert!(version_url(
            &historical.replace("_Map_Pack.zip", "_Map_Pack_1.03.zip"),
            "R21h"
        )
        .unwrap()
        .is_none());
        assert!(supported("R20e"));
    }
}
