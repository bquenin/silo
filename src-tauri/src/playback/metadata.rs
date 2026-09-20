//! Compiled MapMetaData supplies the replay's MC compatibility value.
//! It is not a cryptographic checksum: compare it together with the complete
//! map path, then verify the containing pack and its bundled scripts.
//!
//! Layout: WrathEd2012 SAGE/Games/Kane's Wrath/Includes/MetaDataCommon.xml,
//! MapMetaData.xml and SAGE.Stream/{Header,AssetEntry}.cs (manifest version 5).
use super::archive;
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Stock metadata is RefPack-compressed inside its BIG. Bound both the
/// compressed entry and expanded data; never decode arbitrary game payloads.
fn decode(bytes: &[u8], limit: usize) -> Result<Vec<u8>> {
    if bytes.get(1) != Some(&0xfb) || bytes[0] & 0x3e != 0x10 {
        ensure!(bytes.len() <= limit, "Map metadata exceeds its size limit");
        return Ok(bytes.to_vec());
    }
    let mut at = 2usize;
    let width = if bytes[0] & 0x80 != 0 { 4 } else { 3 };
    let mut read_size = || -> Result<usize> {
        let raw = bytes
            .get(at..at + width)
            .context("Truncated RefPack size")?;
        at += width;
        Ok(raw.iter().fold(0usize, |n, b| (n << 8) | usize::from(*b)))
    };
    if bytes[0] & 1 != 0 {
        read_size()?;
    }
    let expected = read_size()?;
    ensure!(
        expected <= limit,
        "Expanded map metadata exceeds its size limit"
    );
    let mut output = Vec::with_capacity(expected);
    loop {
        let mut byte = || -> Result<usize> {
            let value = *bytes.get(at).context("Truncated RefPack command")?;
            at += 1;
            Ok(usize::from(value))
        };
        let code = byte()?;
        let (literal, length, distance) = match code {
            0..=0x7f => (
                code & 3,
                ((code & 0x1c) >> 2) + 3,
                ((code & 0x60) << 3) + byte()? + 1,
            ),
            0x80..=0xbf => {
                let second = byte()?;
                (
                    second >> 6,
                    (code & 0x3f) + 4,
                    ((second & 0x3f) << 8) + byte()? + 1,
                )
            }
            0xc0..=0xdf => {
                let second = byte()?;
                let third = byte()?;
                (
                    code & 3,
                    ((code & 0x0c) << 6) + byte()? + 5,
                    ((code & 0x10) << 12) + (second << 8) + third + 1,
                )
            }
            0xe0..=0xfb => (((code & 0x1f) + 1) << 2, 0, 0),
            _ => (code & 3, 0, 0),
        };
        ensure!(
            output.len() + literal + length <= expected,
            "RefPack output exceeds declared size"
        );
        output.extend_from_slice(
            bytes
                .get(at..at + literal)
                .context("Truncated RefPack literal")?,
        );
        at += literal;
        if length > 0 {
            ensure!(
                distance > 0 && distance <= output.len(),
                "Invalid RefPack back reference"
            );
            for _ in 0..length {
                output.push(output[output.len() - distance]);
            }
        }
        if code >= 0xfc {
            break;
        }
    }
    ensure!(
        output.len() == expected && at == bytes.len(),
        "Incomplete or trailing RefPack data"
    );
    Ok(output)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .context("Truncated compiled map metadata")?
            .try_into()?,
    ))
}

fn parse(binary: &[u8], manifest: &[u8]) -> Result<BTreeMap<String, u32>> {
    let binary = decode(binary, 16 * 1024 * 1024)?;
    let binary = binary.as_slice();
    ensure!(
        manifest.len() >= 92
            && manifest[..4] == [0, 1, 5, 0]
            && (1..=4096).contains(&u32_at(manifest, 12)?)
            && manifest.len() >= 48 + u32_at(manifest, 12)? as usize * 44,
        "Unsupported compiled MapMetaData layout"
    );
    ensure!(
        u32_at(binary, 0)? == u32_at(manifest, 4)?
            && binary.len() == 4 + u32_at(manifest, 16)? as usize,
        "Map metadata stream does not match its manifest"
    );
    let mut result = BTreeMap::new();
    let mut cursor = 4usize;
    for i in 0..u32_at(manifest, 12)? as usize {
        let record = 48 + i * 44;
        let size = u32_at(manifest, record + 32)? as usize;
        let end = cursor
            .checked_add(size)
            .context("Invalid compiled asset size")?;
        let data = binary
            .get(cursor..end)
            .context("Compiled asset is outside stream")?;
        cursor = end;
        if size > 0 && u32_at(manifest, record)? == 0x5f969146 {
            ensure!(
                u32_at(manifest, record + 8)? == 1059476008,
                "Unsupported MapMetaData type version"
            );
            // R2–R15 store one map per asset; recent packs use one list.
            // Each asset's pointers are relative to its own data chunk.
            for (name, crc) in parse_asset(data)? {
                ensure!(
                    result.insert(name, crc).is_none_or(|old| old == crc),
                    "Conflicting map metadata compatibility values"
                );
            }
        }
    }
    ensure!(
        cursor == binary.len(),
        "Unaccounted compiled map metadata bytes"
    );
    Ok(result)
}

/// The first base archive with stock metadata supplies the active stream.
/// Older metadata must not be used when an earlier mounted stream differs.
pub fn stock_crc(archives: &[PathBuf], asset: &str) -> Result<Option<u32>> {
    for path in archives {
        let entries = archive::index(path)?;
        let Some(binary) = entries.iter().find(|e| e.name == "data/mapmetadata.bin") else {
            continue;
        };
        let manifest = entries
            .iter()
            .find(|e| e.name == "data/mapmetadata.manifest")
            .context("Stock map metadata manifest is missing")?;
        let maps = parse(
            &archive::read_entry(path, binary, 16 * 1024 * 1024)?,
            &archive::read_entry(path, manifest, 1024 * 1024)?,
        )?;
        return Ok(maps.get(asset).copied());
    }
    Ok(None)
}

fn parse_asset(data: &[u8]) -> Result<BTreeMap<String, u32>> {
    let count = u32_at(data, 4)? as usize;
    let start = u32_at(data, 8)? as usize;
    ensure!(
        (1..=4096).contains(&count)
            && start >= 12
            && start
                .checked_add(count * 60)
                .is_some_and(|end| end <= data.len()),
        "Map metadata records are out of bounds"
    );
    let mut result = BTreeMap::new();
    for i in 0..count {
        let record = start + i * 60;
        let crc = u32_at(data, record + 24)?;
        let length = u32_at(data, record + 28)? as usize;
        let offset = u32_at(data, record + 32)? as usize;
        ensure!(
            (1..=4096).contains(&length) && offset >= start + count * 60,
            "Invalid map metadata filename"
        );
        let name = data
            .get(
                offset
                    ..offset
                        .checked_add(length)
                        .context("Invalid map filename offset")?,
            )
            .context("Map filename is outside compiled metadata")?;
        let name = archive::normalize(std::str::from_utf8(name)?);
        ensure!(
            name.starts_with("data/maps/official/")
                && name.ends_with(".map")
                && !name.contains('\0')
                && !name
                    .split('/')
                    .any(|p| p.is_empty() || p == "." || p == ".."),
            "Invalid map metadata asset path"
        );
        // Some original packs repeat a map under multiple display entries.
        // Identical compatibility values are harmless; conflicting ones fail.
        ensure!(
            result.insert(name, crc).is_none_or(|old| old == crc),
            "Conflicting map metadata compatibility values"
        );
    }
    Ok(result)
}

pub fn matches(path: &Path, asset: &str, crc: u32) -> Result<bool> {
    let entries = archive::index(path)?;
    if !entries.iter().any(|e| e.name == asset) {
        return Ok(false);
    }
    let mut matched = false;
    for entry in &entries {
        if !entry.name.starts_with("data/additionalmaps/mapmetadata")
            || !entry.name.ends_with(".bin")
        {
            continue;
        }
        let manifest_name = format!("{}.manifest", entry.name.trim_end_matches(".bin"));
        let manifest = entries
            .iter()
            .find(|e| e.name == manifest_name)
            .context("Compiled map metadata manifest is missing")?;
        let manifest = archive::read_entry(path, manifest, 1024 * 1024)?;
        let maps = parse(
            &archive::read_entry(path, entry, 16 * 1024 * 1024)?,
            &manifest,
        )?;
        if let Some(value) = maps.get(asset) {
            if *value != crc {
                return Ok(false);
            }
            matched = true;
        }
    }
    Ok(matched)
}

#[cfg(test)]
pub fn fixture(names: &[&str], crc: u32) -> (Vec<u8>, Vec<u8>) {
    let mut data = vec![0; 12 + names.len() * 60];
    data[4..8].copy_from_slice(&(names.len() as u32).to_le_bytes());
    data[8..12].copy_from_slice(&12u32.to_le_bytes());
    for (i, name) in names.iter().enumerate() {
        let at = 12 + i * 60;
        let offset = data.len() as u32;
        data[at + 24..at + 28].copy_from_slice(&crc.to_le_bytes());
        data[at + 28..at + 32].copy_from_slice(&(name.len() as u32).to_le_bytes());
        data[at + 32..at + 36].copy_from_slice(&offset.to_le_bytes());
        data.extend(name.as_bytes());
    }
    let mut manifest = vec![0; 92];
    manifest[..4].copy_from_slice(&[0, 1, 5, 0]);
    for (offset, value) in [
        (12, 1u32),
        (16, data.len() as u32),
        (48, 0x5f969146),
        (56, 1059476008),
        (80, data.len() as u32),
    ] {
        manifest[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    let mut binary = vec![0; 4];
    binary.extend(data);
    (binary, manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compressed_metadata_checks_back_references_sizes_and_stream_end() {
        let data = b"\x10\xfb\0\0\x08\xe0ABCD\x04\x03\xfc";
        assert_eq!(decode(data, 8).unwrap(), b"ABCDABCD");
        assert!(decode(data, 7).is_err());
        assert!(decode(&data[..data.len() - 1], 8).is_err());
        let mut trailing = data.to_vec();
        trailing.push(0);
        assert!(decode(&trailing, 8).is_err());
        let mut invalid = data.to_vec();
        invalid[11] = 10;
        assert!(decode(&invalid, 8).is_err());
    }
    #[test]
    fn legacy_metadata_with_multiple_assets_uses_each_assets_own_pointers() {
        let first = "data/maps/official/first 1.02+ edition/first 1.02+ edition.map";
        let second = "data/maps/official/second 1.02+ edition/second 1.02+ edition.map";
        let (mut binary, mut manifest) = fixture(&[first], 4);
        let (next, next_manifest) = fixture(&[second], 6);
        binary.extend(&next[4..]);
        manifest.extend(&next_manifest[48..92]);
        manifest[12..16].copy_from_slice(&2u32.to_le_bytes());
        manifest[16..20].copy_from_slice(&((binary.len() - 4) as u32).to_le_bytes());
        let maps = parse(&binary, &manifest).unwrap();
        assert_eq!(maps[first], 4);
        assert_eq!(maps[second], 6);
    }
    #[test]
    fn compiled_metadata_requires_matching_manifest_and_bounded_records() {
        let asset = "data/maps/official/map__18/map__18.map";
        let (binary, manifest) = fixture(&[asset], 43);
        assert_eq!(parse(&binary, &manifest).unwrap()[asset], 43);
        let mut wrong = binary.clone();
        wrong[0] = 1;
        assert!(parse(&wrong, &manifest).is_err());
        let mut outside = binary.clone();
        outside[48..52].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse(&outside, &manifest).is_err());
        let (mut duplicate, duplicate_manifest) = fixture(&[asset, asset], 43);
        assert_eq!(parse(&duplicate, &duplicate_manifest).unwrap()[asset], 43);
        duplicate[100..104].copy_from_slice(&44u32.to_le_bytes());
        assert!(parse(&duplicate, &duplicate_manifest).is_err());
    }
}
