//! Data-only extraction of the provider's ZIP / NSISBI map packs.
//!
//! NSIS structures follow Source/exehead/fileform.h (NSIS / NSISBI).
//! We read file records only: no installer instructions, plugins, engine
//! replacements, registry changes, or supplied configuration are executed.
//! Supported layouts: ANSI/Unicode NSIS with solid LZMA or non-solid DEFLATE,
//! and NSISBI with chunked non-solid LZMA. See docs/map-pack-sources.md.
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

struct CheckedReader<'a, R> {
    inner: R,
    failed: &'a Cell<bool>,
}

impl<R: Read> Read for CheckedReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let result = self.inner.read(buffer);
        if !buffer.is_empty() && matches!(&result, Ok(0) | Err(_)) {
            self.failed.set(true);
        }
        result
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
            || super::sources::map_archive(&name, &revision)
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

#[derive(Clone, Copy)]
enum Layout {
    Deflate,
    ChunkedLzma,
}

impl Layout {
    fn item_bytes(self) -> u64 {
        match self {
            Self::Deflate => 4,
            Self::ChunkedLzma => 8,
        }
    }

    fn first_header_bytes(self) -> u64 {
        match self {
            Self::Deflate => 28,
            Self::ChunkedLzma => 36,
        }
    }

    fn item_header(self, file: &mut File, offset: u64, end: u64) -> Result<(bool, u64)> {
        if matches!(self, Self::ChunkedLzma) {
            return item_header(file, offset, end);
        }
        ensure!(
            offset.checked_add(4).is_some_and(|n| n <= end),
            "Installer item outside package"
        );
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = [0; 4];
        file.read_exact(&mut bytes)?;
        let length = u32::from_le_bytes(bytes);
        let size = (length & 0x7fff_ffff) as u64;
        ensure!(size <= end - offset - 4, "Truncated installer item");
        Ok((length >> 31 != 0, size))
    }

    fn decode(
        self,
        file: &mut File,
        offset: u64,
        end: u64,
        output: &mut impl Write,
        limit: u64,
        control: &Control<'_>,
        progress: Option<&ExtractionProgress<'_, '_>>,
    ) -> Result<u64> {
        if matches!(self, Self::ChunkedLzma) {
            return item(file, offset, end, output, limit, control, progress);
        }
        let (compressed, size) = self.item_header(file, offset, end)?;
        if let Some(progress) = progress {
            progress.advance(4);
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
        let written = inflate(&mut input, output, limit, control)?;
        ensure!(input.limit() == 0, "Truncated installer item");
        Ok(written)
    }
}

/// Require an actual end-of-stream marker: a decoder returning EOF alone does
/// not establish that a truncated raw DEFLATE stream was complete.
fn inflate(
    mut input: impl Read,
    output: &mut impl Write,
    limit: u64,
    control: &Control<'_>,
) -> Result<u64> {
    let mut decoder = flate2::Decompress::new(false);
    let mut compressed = [0; 128 * 1024];
    let mut expanded = [0; 128 * 1024];
    let (mut start, mut end) = (0, 0);
    loop {
        control.check()?;
        if start == end {
            end = input.read(&mut compressed)?;
            start = 0;
        }
        let (before_in, before_out) = (decoder.total_in(), decoder.total_out());
        let status = decoder.decompress(
            &compressed[start..end],
            &mut expanded,
            flate2::FlushDecompress::None,
        )?;
        let consumed = (decoder.total_in() - before_in) as usize;
        let written = (decoder.total_out() - before_out) as usize;
        ensure!(
            decoder.total_out() <= limit,
            "The map package expands beyond the supported size."
        );
        output.write_all(&expanded[..written])?;
        start += consumed;
        if status == flate2::Status::StreamEnd {
            ensure!(
                start == end && input.read(&mut [0])? == 0,
                "Trailing data in installer item"
            );
            return Ok(decoder.total_out());
        }
        ensure!(
            consumed > 0 || written > 0,
            "Truncated DEFLATE installer item"
        );
    }
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

fn string_at(header: &[u8], table: usize, end: usize, index: u32, unicode: bool) -> Result<String> {
    let start = table
        .checked_add(
            (index as usize)
                .checked_mul(if unicode { 2 } else { 1 })
                .context("Invalid string offset")?,
        )
        .context("Invalid string offset")?;
    let bytes = header
        .get(start..end)
        .context("Installer string outside table")?;
    if !unicode {
        let length = bytes
            .iter()
            .take(4096)
            .position(|b| *b == 0)
            .context("Unterminated installer string")?;
        // NSIS variable codes are opaque bytes here. We only compare the
        // ASCII Patch103 suffix and allowlisted ASCII filenames, never expand
        // variables or interpret the installer's output paths.
        return Ok(bytes[..length].iter().map(|b| char::from(*b)).collect());
    }
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
    let entry_size = if count > 0 && count <= 100_000 {
        [28, 36]
            .into_iter()
            .find(|size| entries.checked_add(count * size) == Some(strings))
    } else {
        None
    };
    ensure!(
        entries >= 60 && entry_size.is_some() && strings < languages && languages <= header.len(),
        "Unsupported installer record layout"
    );
    // Both encodings begin with the empty string. A second zero byte marks
    // the UTF-16 table; legacy ANSI tables continue with their first string.
    ensure!(
        header.get(strings) == Some(&0),
        "Unsupported installer string encoding"
    );
    let unicode = header.get(strings + 1) == Some(&0);
    let entry_size = entry_size.unwrap();
    let mut files = BTreeMap::new();
    let mut in_patch = false;
    for n in 0..count {
        let at = entries + n * entry_size;
        match u32_at(header, at)? {
            11 if u32_at(header, at + 8)? != 0 => {
                let directory =
                    string_at(header, strings, languages, u32_at(header, at + 4)?, unicode)?;
                in_patch = directory
                    .replace('/', "\\")
                    .to_ascii_lowercase()
                    .ends_with("\\patch103");
            }
            20 if in_patch => {
                let name = string_at(header, strings, languages, u32_at(header, at + 8)?, unicode)?;
                if wanted(&name, revision) {
                    // Some packs include the texture fix twice. NSIS's later
                    // file record replaces the earlier one at the same path.
                    let offset = if entry_size == 36 {
                        u64_at(header, at + 12)?
                    } else {
                        u32_at(header, at + 12)? as u64
                    };
                    files.insert(name.to_ascii_lowercase(), offset);
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

/// Solid NSIS stores its header and all file items in one LZMA stream. Decode
/// that bounded stream into an owned temporary data file before following
/// offsets. It is never an executable and is removed when this function exits.
fn extract_solid(
    file: &mut File,
    offset: u64,
    end: u64,
    expected_header: u64,
    output: &Path,
    revision: &str,
    label: &str,
    control: &Control<'_>,
) -> Result<()> {
    ensure!(
        offset.checked_add(10).is_some_and(|n| n <= end),
        "Truncated solid installer stream"
    );
    file.seek(SeekFrom::Start(offset))?;
    let mut props = [0; 5];
    file.read_exact(&mut props)?;
    let dict = u32::from_le_bytes(props[1..].try_into()?);
    ensure!(
        props[0] == 0x5d && dict > 0 && dict <= 64 * 1024 * 1024,
        "Unsupported solid installer compression"
    );
    let progress = ExtractionProgress::new(control, end - offset, label)?;
    progress.advance(5);
    let mut compressed = file.take(end - offset - 5);
    let premature_eof = Cell::new(false);
    let mut decoded =
        tempfile::tempfile_in(output.parent().context("Missing extraction directory")?)?;
    let decoded_size = {
        // The decoder's byte adapter substitutes zero on read errors. Record
        // premature EOF and I/O failures independently of its end marker.
        let checked = CheckedReader {
            inner: &mut compressed,
            failed: &premature_eof,
        };
        let tracked = ProgressReader {
            inner: checked,
            on_read: |bytes| progress.advance(bytes),
        };
        let mut buffered = BufReader::new(tracked);
        let mut decoder =
            LzmaReader::new_with_props(&mut buffered, u64::MAX, props[0], dict, None)?;
        let size = copy_limited(&mut decoder, &mut decoded, MAX_EXPANDED, control)?;
        ensure!(!premature_eof.get(), "Truncated solid installer stream");
        ensure!(
            buffered.buffer().is_empty(),
            "Trailing data in solid installer stream"
        );
        size
    };
    ensure!(compressed.limit() == 0, "Truncated solid installer stream");
    decoded.seek(SeekFrom::Start(0))?;
    let mut length = [0; 4];
    decoded.read_exact(&mut length)?;
    ensure!(
        u32::from_le_bytes(length) as u64 == expected_header,
        "Solid installer header size does not match"
    );
    let mut header = vec![0; expected_header as usize];
    decoded.read_exact(&mut header)?;
    let data = 4 + expected_header;
    let files = records(&header, revision)?;
    let mut total = 0;
    for (name, offset) in files {
        control.check()?;
        let at = data
            .checked_add(offset)
            .context("Invalid solid installer offset")?;
        let (compressed, size) = Layout::Deflate.item_header(&mut decoded, at, decoded_size)?;
        ensure!(
            !compressed,
            "Unexpected compressed file in solid installer stream"
        );
        ensure!(
            size <= MAX_FILE && size <= MAX_EXPANDED - total,
            "The map package expands beyond the supported size."
        );
        let written = copy_limited(
            (&mut decoded).take(size),
            &mut File::create(output.join(name))?,
            size,
            control,
        )?;
        ensure!(written == size, "Incomplete file in solid installer stream");
        total += written;
    }
    progress.finish()
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
    let flags = u32_at(&prefix, start)?;
    let layout = match flags & !0x0e {
        0 => Layout::Deflate,
        0x50 if u64_at(&prefix, start + 28)? == 0 => Layout::ChunkedLzma,
        _ => anyhow::bail!("Unsupported NSIS variant"),
    };
    let expected_header = u32_at(&prefix, start + 20)? as u64;
    ensure!(
        (60..=MAX_HEADER).contains(&expected_header),
        "Invalid installer header size"
    );
    let end = (start as u64)
        .checked_add(u32_at(&prefix, start + 24)? as u64)
        .context("Invalid installer size")?;
    ensure!(
        end <= size && end > start as u64 + layout.first_header_bytes() + layout.item_bytes(),
        "Installer extends outside package"
    );
    // Solid LZMA begins with five properties bytes and a zero range-coder
    // byte, rather than a per-item length. Other codecs/layouts fail closed.
    if matches!(layout, Layout::Deflate)
        && prefix.get(start + 28) == Some(&0x5d)
        && prefix.get(start + 33) == Some(&0)
    {
        let compressed_end = end - if flags & 4 == 0 { 4 } else { 0 };
        return extract_solid(
            &mut file,
            start as u64 + 28,
            compressed_end,
            expected_header,
            output,
            revision,
            label,
            control,
        );
    }
    let header_offset = start as u64 + layout.first_header_bytes();
    let (_, length) = layout.item_header(&mut file, header_offset, end)?;
    let data = (header_offset + layout.item_bytes())
        .checked_add(length)
        .context("Invalid installer data offset")?;
    let mut header = Vec::new();
    layout.decode(
        &mut file,
        header_offset,
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
        let (_, size) = layout.item_header(&mut file, *offset, end)?;
        expected += layout.item_bytes() + size;
    }
    let progress = ExtractionProgress::new(control, expected, label)?;
    let mut total = 0;
    for (name, offset) in files {
        control.check()?;
        let mut target = File::create(output.join(&name))?;
        total += layout.decode(
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

    // Synthetic ANSI NSIS header and file records, compressed independently
    // with Python's lzma.FORMAT_RAW (LZMA1, lc=3, lp=0, pb=2, dict=8 MiB).
    const SOLID: &[u8] = include_bytes!("fixtures/solid-ansi.lzma");
    const SOLID_HEADER: u32 = 318;

    fn solid_installer(root: &Path, compressed: &[u8], header_size: u32) -> PathBuf {
        let mut payload = vec![0; 512];
        payload[..2].copy_from_slice(b"MZ");
        payload.extend(0u32.to_le_bytes());
        payload.extend(SIGNATURE);
        payload.extend(header_size.to_le_bytes());
        payload.extend((28 + compressed.len() as u32 + 4).to_le_bytes());
        payload.extend(compressed);
        // The outer ZIP validates payload integrity; the NSIS CRC isn't used.
        payload.extend(0u32.to_le_bytes());
        let path = root.join("installer.dat");
        fs::write(&path, payload).unwrap();
        path
    }

    #[test]
    fn solid_ansi_nsis_extracts_only_allowlisted_patch_content() {
        let root = tempfile::tempdir().unwrap();
        let input = solid_installer(root.path(), SOLID, SOLID_HEADER);
        let output = root.path().join("content");
        fs::create_dir(&output).unwrap();
        let cancel = Cancellation::default();
        let events = RefCell::new(Vec::new());
        let on_progress = |event| events.borrow_mut().push(event);
        let control = Control {
            cancelled: &cancel,
            progress: &on_progress,
        };
        extract_installer(&input, &output, "R16", "1v1 map pack", &control).unwrap();
        assert_eq!(
            fs::read(output.join("102plusmaps.big")).unwrap(),
            b"exact R16 map"
        );
        assert_eq!(
            fs::read(output.join("102scripts.big")).unwrap(),
            b"matching R16 scripts"
        );
        assert_eq!(fs::read_dir(&output).unwrap().count(), 2);
        assert!(!root.path().join("102Scripts.big").exists());
        assert_progress(&events.borrow(), SOLID.len() as u64, true);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    }

    #[test]
    fn solid_lzma_rejects_truncation_trailing_data_and_excessive_dictionaries() {
        let mut trailing = SOLID.to_vec();
        trailing.push(0);
        let mut large_dictionary = SOLID.to_vec();
        large_dictionary[1..5].copy_from_slice(&u32::MAX.to_le_bytes());
        for (case, bytes) in [
            ("truncated", &SOLID[..SOLID.len() - 2]),
            ("trailing", trailing.as_slice()),
            ("dictionary", large_dictionary.as_slice()),
        ] {
            let root = tempfile::tempdir().unwrap();
            let input = solid_installer(root.path(), bytes, SOLID_HEADER);
            let output = root.path().join("content");
            fs::create_dir(&output).unwrap();
            let cancel = Cancellation::default();
            let events = RefCell::new(Vec::new());
            let on_progress = |event| events.borrow_mut().push(event);
            let control = Control {
                cancelled: &cancel,
                progress: &on_progress,
            };
            assert!(
                extract_installer(&input, &output, "R16", "1v1 map pack", &control).is_err(),
                "Accepted {case} stream"
            );
            assert!(events
                .borrow()
                .iter()
                .all(|e: &Progress| e.total != Some(e.completed)));
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        }
    }

    #[test]
    fn solid_lzma_honors_cancellation_during_decode() {
        let root = tempfile::tempdir().unwrap();
        let input = solid_installer(root.path(), SOLID, SOLID_HEADER);
        let output = root.path().join("content");
        fs::create_dir(&output).unwrap();
        let cancel = Cancellation::default();
        let on_progress = |event: Progress| {
            if event.completed > 0 {
                cancel.cancel();
            }
        };
        let control = Control {
            cancelled: &cancel,
            progress: &on_progress,
        };
        assert!(
            extract_installer(&input, &output, "R16", "1v1 map pack", &control)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        assert_eq!(fs::read_dir(&output).unwrap().count(), 0);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    }

    fn deflated(bytes: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn classic_deflate_requires_complete_streams_and_respects_limits_and_cancellation() {
        let payload = b"map payload".repeat(40_000);
        let compressed = deflated(&payload);
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        let mut out = Vec::new();
        assert_eq!(
            inflate(
                compressed.as_slice(),
                &mut out,
                payload.len() as u64,
                &control
            )
            .unwrap(),
            payload.len() as u64
        );
        assert_eq!(out, payload);
        assert!(inflate(
            &compressed[..compressed.len() - 1],
            &mut Vec::new(),
            u64::MAX,
            &control
        )
        .is_err());
        let mut trailing = compressed.clone();
        trailing.push(0);
        assert!(inflate(trailing.as_slice(), &mut Vec::new(), u64::MAX, &control).is_err());
        assert!(inflate(compressed.as_slice(), &mut Vec::new(), 100, &control).is_err());
        cancel.cancel();
        assert!(
            inflate(compressed.as_slice(), &mut Vec::new(), u64::MAX, &control)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
    }

    #[test]
    fn classic_nsis_extracts_exact_historical_archive_aliases_as_data() {
        let mut header = vec![0; 60 + 28 * 3];
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
        let map = add_string("R201v1Maps.big");
        let scripts = add_string("102Scripts.big");
        let end = header.len() as u32;
        for (at, value) in [(20, 60), (24, 3), (28, strings), (36, end)] {
            header[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        let item = |bytes: &[u8]| {
            let compressed = deflated(bytes);
            let mut item = (0x8000_0000 | compressed.len() as u32)
                .to_le_bytes()
                .to_vec();
            item.extend(compressed);
            item
        };
        let mut data = item(b"map data");
        let scripts_offset = data.len() as u32;
        data.extend(item(b"matching scripts"));
        for (n, values) in [
            [11, patch, 1, 0, 0, 0, 0],
            [20, 0, map, 0, 123456, 0, 0],
            [20, 0, scripts, scripts_offset, 123456, 0, 0],
        ]
        .into_iter()
        .enumerate()
        {
            for (i, value) in values.into_iter().enumerate() {
                let at = 60 + n * 28 + i * 4;
                header[at..at + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        let header_item = item(&header);
        let mut payload = vec![0; 512];
        payload[..2].copy_from_slice(b"MZ");
        payload.extend(0u32.to_le_bytes());
        payload.extend(SIGNATURE);
        payload.extend((header.len() as u32).to_le_bytes());
        payload.extend((28 + header_item.len() as u32 + data.len() as u32).to_le_bytes());
        payload.extend(header_item);
        payload.extend(data);
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("installer.dat");
        fs::write(&input, payload).unwrap();
        let output = root.path().join("content");
        fs::create_dir(&output).unwrap();
        let cancel = Cancellation::default();
        let control = Control {
            cancelled: &cancel,
            progress: &|_| {},
        };
        extract_installer(&input, &output, "R20e", "1v1 map pack", &control).unwrap();
        assert_eq!(
            fs::read(output.join("r201v1maps.big")).unwrap(),
            b"map data"
        );
        assert_eq!(
            fs::read(output.join("102scripts.big")).unwrap(),
            b"matching scripts"
        );
        assert_eq!(fs::read_dir(output).unwrap().count(), 2);
    }

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
