//! Data-only extraction of the provider's ZIP / NSISBI map packs.
//!
//! NSIS structures follow Source/exehead/fileform.h (NSIS / NSISBI).
//! We read file records only: no installer instructions, plugins, engine
//! replacements, registry changes, or supplied configuration are executed.
//! Supported installer layout: Unicode, non-solid LZMA, with the NSISBI
//! 64-bit item lengths and 24-bit chunk framing used by the historical packs.
use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use lzma_rust2::LzmaReader;

use super::automatic::{Control, Progress};

const MAX_FILE: u64 = 5 * 1024 * 1024 * 1024;
const MAX_EXPANDED: u64 = 16 * 1024 * 1024 * 1024;
const MAX_HEADER: u64 = 16 * 1024 * 1024;
const SIGNATURE: &[u8] = b"\xef\xbe\xad\xdeNullsoftInst";

/// One byte budget for all selected files. Installer progress counts compressed
/// input, whose size is known before decoding; ZIP progress counts output.
struct ExtractionProgress<'a, 'c> {
    control: &'a Control<'c>,
    total: u64,
    message: String,
    completed: Cell<u64>,
    last_update: Cell<Instant>,
}

impl<'a, 'c> ExtractionProgress<'a, 'c> {
    fn new(control: &'a Control<'c>, total: u64, label: &str) -> Result<Self> {
        control.check()?;
        let progress = Self {
            control,
            total,
            message: format!("Unpacking the {label}…"),
            completed: Cell::new(0),
            last_update: Cell::new(Instant::now() - Duration::from_millis(150)),
        };
        progress.emit(0);
        Ok(progress)
    }

    fn emit(&self, completed: u64) {
        (self.control.progress)(Progress {
            phase: "extracting",
            message: self.message.clone(),
            completed,
            total: Some(self.total),
        });
    }

    fn advance(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.completed.set(self.completed.get() + bytes);
        if self.last_update.get().elapsed() >= Duration::from_millis(150) {
            // Buffered input can reach EOF before decoding and validation end.
            // Reserve 100% for successful completion of every selected file.
            self.emit(self.completed.get().min(self.total.saturating_sub(1)));
            self.last_update.set(Instant::now());
        }
    }

    fn finish(&self) -> Result<()> {
        self.control.check()?;
        ensure!(
            self.completed.get() == self.total,
            "The map package changed while unpacking."
        );
        self.emit(self.total);
        self.control.check()
    }
}

struct ProgressReader<R, F> {
    inner: R,
    on_read: F,
}

impl<R: Read, F: FnMut(u64)> Read for ProgressReader<R, F> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        (self.on_read)(read as u64);
        Ok(read)
    }
}

fn wanted(name: &str, revision: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let revision = revision.to_ascii_lowercase();
    !name.contains(['/', '\\', ':'])
        && name.is_ascii()
        && (name == "102scripts.big"
            || name == "102texturefix.big"
            || name == format!("{revision}scripts.big")
            || (name.starts_with(&revision)
                && name.ends_with("maps.big")
                && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.')))
}

fn copy_limited(
    mut input: impl Read,
    output: &mut impl Write,
    limit: u64,
    control: &Control<'_>,
) -> Result<u64> {
    let mut buffer = [0; 128 * 1024];
    let mut size = 0;
    loop {
        control.check()?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            return Ok(size);
        }
        size += n as u64;
        ensure!(
            size <= limit,
            "The map package expands beyond the supported size."
        );
        output.write_all(&buffer[..n])?;
    }
}

pub fn unpack(
    package: &Path,
    stage: &Path,
    revision: &str,
    label: &str,
    control: &Control<'_>,
) -> Result<PathBuf> {
    control.stage("extracting", format!("Unpacking the {label}…"))?;
    let mut zip = zip::ZipArchive::new(File::open(package)?)
        .context("The map provider did not return a valid ZIP package. Press Play to retry.")?;
    ensure!(
        zip.len() <= 4096,
        "The map package contains too many files."
    );
    let output = stage.join("content");
    fs::create_dir(&output)?;
    let mut files = BTreeMap::new();
    let mut installers = Vec::new();
    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        let name = entry.name().rsplit(['/', '\\']).next().unwrap_or("");
        if wanted(name, revision) {
            ensure!(
                entry.is_file() && !entry.is_symlink() && entry.size() <= MAX_FILE,
                "Invalid map archive in the downloaded package."
            );
            ensure!(
                files
                    .insert(name.to_ascii_lowercase(), (i, entry.size()))
                    .is_none(),
                "Duplicate map archives in the package."
            );
        } else if name.to_ascii_lowercase().ends_with(".exe") {
            installers.push(i);
        }
    }
    if !files.is_empty() {
        ensure!(files.len() <= 128, "Too many map archives in the package.");
        let expected: u64 = files.values().map(|(_, size)| size).sum();
        ensure!(
            expected <= MAX_EXPANDED,
            "The map package expands beyond the supported size."
        );
        let progress = ExtractionProgress::new(control, expected, label)?;
        let mut total = 0;
        for (name, (index, size)) in files {
            let mut entry = zip.by_index(index)?;
            let tracked = ProgressReader {
                inner: &mut entry,
                on_read: |bytes| progress.advance(bytes),
            };
            let written = copy_limited(
                tracked,
                &mut File::create(output.join(name))?,
                MAX_FILE.min(MAX_EXPANDED - total),
                control,
            )?;
            ensure!(size == written, "Incomplete map archive.");
            total += written;
        }
        progress.finish()?;
    } else {
        ensure!(
            installers.len() == 1,
            "This map package has an unsupported layout."
        );
        let mut entry = zip.by_index(installers[0])?;
        ensure!(
            entry.is_file() && !entry.is_symlink() && entry.size() <= MAX_FILE,
            "Invalid installer payload."
        );
        let payload = stage.join("payload.dat");
        let mut file = File::create(&payload)?;
        copy_limited(&mut entry, &mut file, MAX_FILE, control)?;
        drop(file);
        extract_installer(&payload, &output, revision, label, control).context(
            "This map pack could not be unpacked safely. Its installer format may be unsupported.",
        )?;
    }
    Ok(output)
}

fn u32_at(data: &[u8], at: usize) -> Result<u32> {
    let bytes = data
        .get(at..at.checked_add(4).context("Invalid installer offset")?)
        .context("Truncated installer header")?;
    Ok(u32::from_le_bytes(bytes.try_into()?))
}

fn u64_at(data: &[u8], at: usize) -> Result<u64> {
    Ok(u32_at(data, at)? as u64 | (u32_at(data, at + 4)? as u64) << 32)
}

fn item_header(file: &mut File, offset: u64, end: u64) -> Result<(bool, u64)> {
    ensure!(
        offset.checked_add(8).is_some_and(|n| n <= end),
        "Installer item outside package"
    );
    file.seek(SeekFrom::Start(offset))?;
    let mut length = [0; 8];
    file.read_exact(&mut length)?;
    let length = u64::from_le_bytes(length);
    let compressed = length >> 63 != 0;
    let size = length & (u64::MAX >> 1);
    ensure!(size <= end - offset - 8, "Truncated installer item");
    Ok((compressed, size))
}

/// Decode one bounded, non-solid data item. NSISBI compresses independent
/// chunks with their own five-byte LZMA properties and an explicit end marker.
fn item(
    file: &mut File,
    offset: u64,
    end: u64,
    output: &mut impl Write,
    limit: u64,
    control: &Control<'_>,
    progress: Option<&ExtractionProgress<'_, '_>>,
) -> Result<u64> {
    let (compressed, size) = item_header(file, offset, end)?;
    if let Some(progress) = progress {
        progress.advance(8);
    }
    let tracked = ProgressReader {
        inner: file,
        on_read: |bytes| {
            if let Some(progress) = progress {
                progress.advance(bytes);
            }
        },
    };
    let mut input = tracked.take(size);
    if !compressed {
        let written = copy_limited(&mut input, output, limit, control)?;
        ensure!(written == size, "Truncated uncompressed installer item");
        return Ok(written);
    }
    let mut written = 0;
    loop {
        control.check()?;
        let mut prefix = [0; 3];
        input.read_exact(&mut prefix)?;
        let length = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], 0]) as u64;
        if length == 0 {
            ensure!(
                input.limit() == 0 && written > 0,
                "Invalid installer chunk terminator"
            );
            return Ok(written);
        }
        ensure!(
            length >= 10 && length <= input.limit(),
            "Invalid installer chunk length"
        );
        let mut chunk = (&mut input).take(length);
        let mut props = [0; 5];
        chunk.read_exact(&mut props)?;
        let dict = u32::from_le_bytes(props[1..].try_into()?);
        ensure!(
            props[0] == 0x5d && dict > 0 && dict <= 64 * 1024 * 1024,
            "Unsupported installer compression"
        );
        {
            // The range decoder reads individual bytes. Buffer its file input
            // so a multi-GiB map does not incur millions of OS reads.
            let mut buffered = BufReader::new(&mut chunk);
            let mut decoder =
                LzmaReader::new_with_props(&mut buffered, u64::MAX, props[0], dict, None)?;
            written += copy_limited(
                &mut decoder,
                output,
                (limit - written).min(64 * 1024 * 1024),
                control,
            )?;
            ensure!(
                buffered.buffer().is_empty(),
                "Trailing data in installer chunk"
            );
        }
        ensure!(chunk.limit() == 0, "Trailing data in installer chunk");
    }
}

fn string_at(header: &[u8], table: usize, end: usize, index: u32) -> Result<String> {
    let start = table
        .checked_add(
            (index as usize)
                .checked_mul(2)
                .context("Invalid string offset")?,
        )
        .context("Invalid string offset")?;
    let bytes = header
        .get(start..end)
        .context("Installer string outside table")?;
    let mut units = Vec::new();
    for pair in bytes.as_chunks::<2>().0.iter().take(4096) {
        let unit = u16::from_le_bytes(*pair);
        if unit == 0 {
            return Ok(String::from_utf16(&units)?);
        }
        units.push(unit);
    }
    anyhow::bail!("Unterminated installer string")
}

fn records(header: &[u8], revision: &str) -> Result<BTreeMap<String, u64>> {
    let entries = u32_at(header, 20)? as usize;
    let count = u32_at(header, 24)? as usize;
    let strings = u32_at(header, 28)? as usize;
    let languages = u32_at(header, 36)? as usize;
    ensure!(
        entries >= 60
            && count > 0
            && count <= 100_000
            && entries.checked_add(count * 36) == Some(strings)
            && strings < languages
            && languages <= header.len(),
        "Unsupported installer record layout"
    );
    // The supported NSIS Unicode string table begins with its empty string.
    ensure!(
        header.get(strings..strings + 2) == Some(&[0, 0]),
        "Unsupported installer string encoding"
    );
    let mut files = BTreeMap::new();
    let mut in_patch = false;
    for n in 0..count {
        let at = entries + n * 36;
        match u32_at(header, at)? {
            11 if u32_at(header, at + 8)? != 0 => {
                let directory = string_at(header, strings, languages, u32_at(header, at + 4)?)?;
                in_patch = directory
                    .replace('/', "\\")
                    .to_ascii_lowercase()
                    .ends_with("\\patch103");
            }
            20 if in_patch => {
                let name = string_at(header, strings, languages, u32_at(header, at + 8)?)?;
                if wanted(&name, revision) {
                    // Some packs include the texture fix twice. NSIS's later
                    // file record replaces the earlier one at the same path.
                    files.insert(name.to_ascii_lowercase(), u64_at(header, at + 12)?);
                }
            }
            _ => {}
        }
    }
    ensure!(
        !files.is_empty() && files.len() <= 128,
        "No replay content in installer"
    );
    Ok(files)
}

fn extract_installer(
    payload: &Path,
    output: &Path,
    revision: &str,
    label: &str,
    control: &Control<'_>,
) -> Result<()> {
    let mut file = File::open(payload)?;
    let size = file.metadata()?.len();
    let mut prefix = Vec::new();
    (&mut file).take(8 * 1024 * 1024).read_to_end(&mut prefix)?;
    ensure!(
        prefix.starts_with(b"MZ"),
        "Invalid installer executable header"
    );
    let start = (0..prefix.len().saturating_sub(36))
        .step_by(512)
        .find(|&i| &prefix[i + 4..i + 20] == SIGNATURE)
        .context("NSIS header not found")?;
    // 0x10: long item offsets; 0x40: chunk framing. External/stub installers
    // require additional files and are deliberately not interpreted here.
    ensure!(
        u32_at(&prefix, start)? & !0x0e == 0x50 && u64_at(&prefix, start + 28)? == 0,
        "Unsupported NSIS variant"
    );
    let expected_header = u32_at(&prefix, start + 20)? as u64;
    ensure!(
        (60..=MAX_HEADER).contains(&expected_header),
        "Invalid installer header size"
    );
    let end = (start as u64)
        .checked_add(u32_at(&prefix, start + 24)? as u64)
        .context("Invalid installer size")?;
    ensure!(
        end <= size && end > start as u64 + 44,
        "Installer extends outside package"
    );
    let length = u64_at(&prefix, start + 36)? & (u64::MAX >> 1);
    let data = (start as u64 + 44)
        .checked_add(length)
        .context("Invalid installer data offset")?;
    let mut header = Vec::new();
    item(
        &mut file,
        start as u64 + 36,
        end,
        &mut header,
        expected_header,
        control,
        None,
    )?;
    ensure!(
        header.len() as u64 == expected_header,
        "Incomplete installer header"
    );
    let mut files = records(&header, revision)?;
    let mut expected = 0;
    for offset in files.values_mut() {
        control.check()?;
        *offset = data
            .checked_add(*offset)
            .context("Invalid installer file offset")?;
        let (_, size) = item_header(&mut file, *offset, end)?;
        expected += 8 + size;
    }
    let progress = ExtractionProgress::new(control, expected, label)?;
    let mut total = 0;
    for (name, offset) in files {
        control.check()?;
        let mut target = File::create(output.join(&name))?;
        total += item(
            &mut file,
            offset,
            end,
            &mut target,
            MAX_FILE.min(MAX_EXPANDED - total),
            control,
            Some(&progress),
        )?;
    }
    progress.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::automatic::Cancellation;
    use std::cell::RefCell;

    fn compressed_item(chunks: usize) -> Vec<u8> {
        let compressed = [
            93, 0, 0, 128, 0, 0, 33, 18, 68, 248, 77, 228, 25, 81, 243, 76, 200, 139, 21, 132, 138,
            46, 80, 3, 250, 246, 141, 49, 255, 249, 161, 224, 0,
        ];
        let size = chunks * (compressed.len() + 3) + 3;
        let mut bytes = ((size as u64) | (1 << 63)).to_le_bytes().to_vec();
        for _ in 0..chunks {
            bytes.extend_from_slice(&(compressed.len() as u32).to_le_bytes()[..3]);
            bytes.extend(compressed);
        }
        bytes.extend([0, 0, 0]);
        bytes
    }

    fn header() -> Vec<u8> {
        let mut header = vec![0; 60 + 36 * 6];
        let strings = header.len() as u32;
        let mut add_string = |value: &str| {
            let offset = (header.len() as u32 - strings) / 2;
            for unit in value.encode_utf16().chain([0]) {
                header.extend(unit.to_le_bytes());
            }
            offset
        };
        add_string("");
        let patch = add_string("$INSTDIR\\Patch103");
        let core = add_string("$INSTDIR\\Core");
        let texture = add_string("102texturefix.big");
        let map = add_string("R24g1v1Maps.big");
        let unsafe_name = add_string("..\\102scripts.big");
        let end = header.len() as u32;
        for (at, value) in [(20, 60), (24, 6), (28, strings), (36, end)] {
            header[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (n, values) in [
            [11, patch, 1, 0, 0, 0, 0, 0, 0],
            [20, 0, texture, 4, 0, 0, 0, 0, 0],
            [20, 0, texture, 8, 0, 0, 0, 0, 0],
            [20, 0, unsafe_name, 8, 0, 0, 0, 0, 0],
            [11, core, 1, 0, 0, 0, 0, 0, 0],
            [20, 0, map, 12, 0, 0, 0, 0, 0],
        ]
        .into_iter()
        .enumerate()
        {
            for (i, value) in values.into_iter().enumerate() {
                let at = 60 + n * 36 + i * 4;
                header[at..at + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        header
    }

    #[test]
    fn installer_records_select_only_patch_content_and_resolve_replacements() {
        let mut header = header();
        assert_eq!(
            records(&header, "R24g").unwrap(),
            BTreeMap::from([("102texturefix.big".into(), 8)])
        );
        header[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(records(&header, "R24g").is_err());
        assert!(!wanted("R24j1v1Maps.big", "R24g"));
        assert!(!wanted("C:102scripts.big", "R24g"));
    }

    #[test]
    fn chunked_lzma_checks_terminators_bounds_and_cancellation() {
        let mut bytes = compressed_item(1);
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("item.dat");
        fs::write(&path, &bytes).unwrap();
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        let read = |limit, end, output: &mut Vec<u8>| {
            item(
                &mut File::open(&path).unwrap(),
                0,
                end,
                output,
                limit,
                &control,
                None,
            )
        };
        let mut out = Vec::new();
        assert_eq!(read(100, bytes.len() as u64, &mut out).unwrap(), 17);
        assert_eq!(out, b"BIGF test payload");
        assert!(read(5, bytes.len() as u64, &mut Vec::new()).is_err());
        assert!(read(100, bytes.len() as u64 - 1, &mut Vec::new()).is_err());
        bytes.pop();
        fs::write(&path, &bytes).unwrap();
        assert!(read(100, bytes.len() as u64 + 1, &mut Vec::new()).is_err());
        cancel.cancel();
        assert!(read(100, bytes.len() as u64 + 1, &mut Vec::new())
            .unwrap_err()
            .to_string()
            .contains("cancelled"));
    }

    #[test]
    fn zip_extraction_ignores_installer_side_effects_and_contains_paths() {
        use zip::{write::SimpleFileOptions, ZipWriter};
        let root = tempfile::tempdir().unwrap();
        let zip_path = root.path().join("pack.zip");
        let mut writer = ZipWriter::new(File::create(&zip_path).unwrap());
        for (name, data) in [
            ("../../102Scripts.big", "scripts"),
            ("Patch103/R24g1v1Maps.big", "map"),
            ("CNC3EP1_english_1.2.SkuDef", "do not install"),
            ("engine.exe", "never run"),
        ] {
            writer
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(data.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
        let stage = root.path().join("staging");
        fs::create_dir(&stage).unwrap();
        let cancel = Cancellation::default();
        let events = RefCell::new(Vec::new());
        let on_progress = |event| events.borrow_mut().push(event);
        let control = Control {
            cancelled: &cancel,
            progress: &on_progress,
        };
        let output = unpack(&zip_path, &stage, "R24g", "1v1 map pack", &control).unwrap();
        assert_eq!(fs::read(output.join("102scripts.big")).unwrap(), b"scripts");
        assert_eq!(fs::read_dir(output).unwrap().count(), 2);
        assert!(!root.path().join("102Scripts.big").exists());
        assert!(!root.path().join("CNC3EP1_english_1.2.SkuDef").exists());
        assert_progress(&events.borrow(), 10, true);
    }

    fn assert_progress(events: &[Progress], total: u64, complete: bool) {
        let measured: Vec<_> = events
            .iter()
            .filter(|event| event.total.is_some())
            .collect();
        assert!(measured.len() >= 2);
        assert_eq!(measured[0].completed, 0);
        assert!(measured.iter().all(|event| event.phase == "extracting"
            && event.total == Some(total)
            && event.completed <= total));
        assert!(measured
            .windows(2)
            .all(|pair| pair[0].completed <= pair[1].completed));
        assert!(measured
            .iter()
            .any(|event| event.completed > 0 && event.completed < total));
        assert_eq!(measured.last().unwrap().completed == total, complete);
        assert!(measured[..measured.len() - 1]
            .iter()
            .all(|event| event.completed < total));
    }

    fn installer_zip(root: &Path, corrupt: bool) -> (PathBuf, u64) {
        use zip::{write::SimpleFileOptions, ZipWriter};
        let mut header = header();
        let texture = b"texture";
        let mut data = (texture.len() as u64).to_le_bytes().to_vec();
        data.extend(texture);
        // Both texture records refer to the first item. The map record uses
        // the second item; its output directory is Patch103 as well.
        for n in [1, 2] {
            header[60 + n * 36 + 12..60 + n * 36 + 20].copy_from_slice(&0u64.to_le_bytes());
        }
        let patch = u32_at(&header, 64).unwrap();
        header[60 + 4 * 36 + 4..60 + 4 * 36 + 8].copy_from_slice(&patch.to_le_bytes());
        header[60 + 5 * 36 + 12..60 + 5 * 36 + 20]
            .copy_from_slice(&(data.len() as u64).to_le_bytes());
        data.extend(compressed_item(3));
        if corrupt {
            // All declared lengths are valid; only the final terminator is bad.
            *data.last_mut().unwrap() = 1;
        }
        let total = data.len() as u64;
        let mut payload = vec![0; 512];
        payload[..2].copy_from_slice(b"MZ");
        payload.extend(0x50u32.to_le_bytes());
        payload.extend(SIGNATURE);
        payload.extend((header.len() as u32).to_le_bytes());
        payload.extend((36 + 8 + header.len() as u32 + data.len() as u32).to_le_bytes());
        payload.extend(0u64.to_le_bytes());
        payload.extend((header.len() as u64).to_le_bytes());
        payload.extend(header);
        payload.extend(data);
        let path = root.join("pack.zip");
        let mut zip = ZipWriter::new(File::create(&path).unwrap());
        zip.start_file("pack.exe", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&payload).unwrap();
        zip.finish().unwrap();
        (path, total)
    }

    #[test]
    fn installer_progress_covers_selected_files_and_chunks_without_resetting() {
        let root = tempfile::tempdir().unwrap();
        let (path, total) = installer_zip(root.path(), false);
        let stage = root.path().join("staging");
        fs::create_dir(&stage).unwrap();
        let cancel = Cancellation::default();
        let events = RefCell::new(Vec::new());
        let on_progress = |event| events.borrow_mut().push(event);
        let control = Control {
            cancelled: &cancel,
            progress: &on_progress,
        };
        let output = unpack(&path, &stage, "R24g", "1v1 map pack", &control).unwrap();
        assert_eq!(
            fs::read(output.join("102texturefix.big")).unwrap(),
            b"texture"
        );
        assert_eq!(
            fs::read(output.join("r24g1v1maps.big")).unwrap(),
            b"BIGF test payload".repeat(3)
        );
        assert_progress(&events.borrow(), total, true);
    }

    #[test]
    fn extraction_errors_and_cancellation_do_not_report_completion() {
        for corrupt in [true, false] {
            let root = tempfile::tempdir().unwrap();
            let (path, total) = installer_zip(root.path(), corrupt);
            let stage = root.path().join("staging");
            fs::create_dir(&stage).unwrap();
            let cancel = Cancellation::default();
            let events = RefCell::new(Vec::new());
            let on_progress = |event: Progress| {
                if !corrupt && event.completed > 0 {
                    cancel.cancel();
                }
                events.borrow_mut().push(event);
            };
            let control = Control {
                cancelled: &cancel,
                progress: &on_progress,
            };
            let error = unpack(&path, &stage, "R24g", "1v1 map pack", &control).unwrap_err();
            if !corrupt {
                assert!(format!("{error:#}").contains("cancelled"));
            }
            assert_progress(&events.borrow(), total, false);
        }
    }
}
