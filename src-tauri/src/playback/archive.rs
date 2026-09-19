//! BIGF/BIG4 archive directory reading.
//! Only the bounded directory is read; multi-gigabyte map payloads stay on disk.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{bail, ensure, Context, Result};

pub fn entries(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut header = [0u8; 16];
    reader.read_exact(&mut header)?;
    ensure!(
        &header[..4] == b"BIGF" || &header[..4] == b"BIG4",
        "Not a BIGF/BIG4 archive"
    );
    // Archive-size endianness differs between formats. The directory fields
    // are big endian in both, and the actual file length is authoritative.
    let count = u32::from_be_bytes(header[8..12].try_into()?) as u64;
    let end = u32::from_be_bytes(header[12..16].try_into()?) as u64;
    ensure!(
        (16..=file_size).contains(&end) && end <= 64 * 1024 * 1024,
        "Invalid BIG directory size"
    );
    ensure!(count <= (end - 16) / 9, "Invalid BIG entry count");
    let mut directory = reader.take(end - 16);
    let mut result = Vec::new();
    for _ in 0..count {
        let mut record = [0u8; 8];
        directory
            .read_exact(&mut record)
            .context("Truncated BIG directory")?;
        let offset = u32::from_be_bytes(record[..4].try_into()?) as u64;
        let size = u32::from_be_bytes(record[4..].try_into()?) as u64;
        ensure!(
            offset >= end && offset + size <= file_size,
            "BIG entry outside archive"
        );
        let mut name = String::new();
        loop {
            let mut byte = [0];
            directory
                .read_exact(&mut byte)
                .context("Unterminated BIG filename")?;
            if byte[0] == 0 {
                break;
            }
            if name.len() >= 4096 {
                bail!("BIG filename exceeds 4096 bytes");
            }
            // Decode each filename byte as Latin-1.
            name.push(char::from(byte[0]));
        }
        if size > 0 {
            result.push(normalize(&name));
        }
    }
    Ok(result)
}

pub fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}
