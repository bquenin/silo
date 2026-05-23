//! Ingest pipeline: hash + parse + insert.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::db::Db;
use crate::parser;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct IngestReport {
    pub scanned: usize,
    pub inserted: usize,
    pub duplicates: usize,
    pub errors: Vec<IngestError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestError {
    pub path: String,
    pub message: String,
}

pub fn ingest_path(db: &mut Db, path: &Path) -> Result<IngestReport> {
    let mut report = IngestReport::default();
    if path.is_file() {
        ingest_one(db, path, &mut report);
    } else {
        for entry in WalkDir::new(path) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.path().extension().and_then(|e| e.to_str()) != Some("kwreplay") {
                continue;
            }
            ingest_one(db, entry.path(), &mut report);
        }
    }
    Ok(report)
}

fn ingest_one(db: &mut Db, path: &Path, report: &mut IngestReport) {
    report.scanned += 1;
    let (hash, size) = match hash_file(path) {
        Ok(v) => v,
        Err(e) => {
            report.errors.push(IngestError {
                path: path.to_string_lossy().into_owned(),
                message: format!("hash: {}", e),
            });
            return;
        }
    };
    let replay = match parser::parse_full(path) {
        Ok(r) => r,
        Err(e) => {
            report.errors.push(IngestError {
                path: path.to_string_lossy().into_owned(),
                message: format!("parse: {}", e),
            });
            return;
        }
    };
    match db.insert_replay(&hash, size, &replay) {
        Ok(true) => report.inserted += 1,
        Ok(false) => report.duplicates += 1,
        Err(e) => report.errors.push(IngestError {
            path: path.to_string_lossy().into_owned(),
            message: format!("insert: {}", e),
        }),
    }
}

fn hash_file(path: &Path) -> std::io::Result<(String, u64)> {
    let f = File::open(path)?;
    let size = f.metadata()?.len();
    let mut r = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok((hex::encode(digest), size))
}

/// Default catalogue location: `%APPDATA%\tacitus\catalogue.sqlite3` on Windows,
/// `~/.local/share/tacitus/catalogue.sqlite3` on Linux/macOS.
pub fn default_catalogue_path() -> PathBuf {
    if let Some(p) = std::env::var_os("APPDATA") {
        let mut p = PathBuf::from(p);
        p.push("tacitus");
        p.push("catalogue.sqlite3");
        return p;
    }
    if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".local/share/tacitus/catalogue.sqlite3");
        return p;
    }
    PathBuf::from("catalogue.sqlite3")
}
