//! End-to-end ingest from the optional TACITUS_REPLAY_CORPUS folder:
//! hash, parse, and insert replays into a fresh SQLite catalogue.

mod common;

use std::path::Path;

use tacitus_lib::db::Db;
use tacitus_lib::ingest;

#[test]
fn ingests_corpus_into_sqlite() {
    let Some(root) = common::corpus_dir() else {
        return;
    };
    let tmpdir =
        std::env::temp_dir().join(format!("tacitus-ingest-{}.sqlite3", std::process::id()));
    let _ = std::fs::remove_file(&tmpdir);
    let mut db = Db::open(&tmpdir).expect("open db");

    let report = ingest::ingest_path(&mut db, &root).expect("ingest");
    println!(
        "scanned={} inserted={} duplicates={} errors={}",
        report.scanned,
        report.inserted,
        report.duplicates,
        report.errors.len()
    );
    for e in report.errors.iter().take(5) {
        println!("  ERR {}: {}", e.path, e.message);
    }

    assert!(report.inserted > 0, "no replays were inserted");
    assert!(
        report.errors.len() <= report.scanned / 10,
        "too many ingest errors: {} / {}",
        report.errors.len(),
        report.scanned
    );
    assert_eq!(
        report.scanned,
        report.inserted + report.duplicates + report.errors.len()
    );

    let n = db.count_replays().expect("count");
    assert_eq!(n as usize, report.inserted, "count mismatch");

    // Re-ingest the same folder — should be all duplicates.
    let report2 = ingest::ingest_path(&mut db, &root).expect("re-ingest");
    println!(
        "re-ingest: scanned={} inserted={} duplicates={} errors={}",
        report2.scanned,
        report2.inserted,
        report2.duplicates,
        report2.errors.len()
    );
    assert_eq!(report2.inserted, 0, "second pass should insert nothing");
    assert_eq!(report2.scanned, report.scanned);
    assert_eq!(report2.duplicates, report.inserted + report.duplicates);
    assert_eq!(report2.errors.len(), report.errors.len());

    let _ = std::fs::remove_file(Path::new(&tmpdir));
}
