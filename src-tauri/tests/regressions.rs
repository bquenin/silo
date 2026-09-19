use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use tacitus_lib::{
    db::{Db, ReplayCursor},
    ingest, parser,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "tacitus-regression-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn cli(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_tacitus-cli"))
            .current_dir(&self.0)
            .arg("--db")
            .arg(self.file("catalogue.sqlite3"))
            .arg("--json")
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(self.0.parent(), Some(std::env::temp_dir().as_path()));
        assert!(self
            .0
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("tacitus-regression-"));
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn u32_bytes(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_le_bytes());
}
fn text(out: &mut Vec<u8>, s: &str) {
    out.extend(s.encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
}
fn header(roster: &str) -> Vec<u8> {
    let mut b = b"C&C3 REPLAY HEADER".to_vec();
    b.push(4);
    for n in [1, 2, 0, 0] {
        u32_bytes(&mut b, n);
    }
    for value in ["Fixture", "", "Map", "MapID"] {
        text(&mut b, value);
    }
    b.push(0);
    u32_bytes(&mut b, 0);
    text(&mut b, "Player");
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 8);
    b.extend([0; 8]);
    u32_bytes(&mut b, 1_700_000_000);
    b.extend([0; 33]);
    let raw = format!("M=283data/maps/official/test;MC=1;S={roster};");
    u32_bytes(&mut b, raw.len() as u32);
    b.extend(raw.as_bytes());
    b.push(0);
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 0); // Empty embedded filename is valid.
    b.extend([0; 16]);
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 0);
    b.push(0);
    b.extend([0; 76]);
    b
}
fn replay(roster: &str, ticks: u32) -> Vec<u8> {
    let mut b = header(roster);
    u32_bytes(&mut b, ticks);
    b.push(2);
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 0);
    u32_bytes(&mut b, 0x7fff_ffff);
    b
}
fn roster(teams: &[(i32, i32)]) -> String {
    teams
        .iter()
        .enumerate()
        .map(|(i, (team, faction))| format!("HPlayer{i},0,0,0,0,{faction},0,{team},0,0,0,"))
        .collect::<Vec<_>>()
        .join(":")
}
fn seed(f: &Fixture, name: &str, teams: &[(i32, i32)], ticks: u32) {
    let path = f.file(&format!("{name}.KWReplay"));
    fs::write(&path, replay(&roster(teams), ticks)).unwrap();
    let mut db = Db::open(f.file("catalogue.sqlite3")).unwrap();
    assert_eq!(ingest::ingest_path(&mut db, &path).unwrap().inserted, 1);
}
fn json(output: std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn oversized_chunk_returns_a_parse_error() {
    let f = Fixture::new();
    let mut bytes = header(&roster(&[(-1, 6)]));
    u32_bytes(&mut bytes, 10);
    bytes.push(1);
    u32_bytes(&mut bytes, 32 * 1024 * 1024);
    let path = f.file("oversized.KWReplay");
    fs::write(&path, bytes).unwrap();
    assert!(matches!(
        parser::parse_full(path),
        Err(parser::ParseError::LimitExceeded { field: "chunk", .. })
    ));
}

#[test]
fn invalid_human_and_ai_teams_are_errors_not_panics() {
    let f = Fixture::new();
    for raw in [
        "HPlayer,0,0,0,0,6,0,2147483647,0,0,0,",
        "CB,0,6,0,2147483647,0,0",
    ] {
        let path = f.file("invalid.KWReplay");
        fs::write(&path, replay(raw, 300)).unwrap();
        assert!(matches!(
            parser::parse_full(path),
            Err(parser::ParseError::BadHeader(_))
        ));
    }
}

#[test]
fn relative_cli_import_persists_an_absolute_path_and_duration() {
    let f = Fixture::new();
    fs::write(
        f.file("relative.KWReplay"),
        replay(&roster(&[(-1, 6), (-1, 9)]), 18000),
    )
    .unwrap();
    assert_eq!(json(f.cli(&["import", "relative.KWReplay"]))["inserted"], 1);
    let db = Db::open(f.file("catalogue.sqlite3")).unwrap();
    let target = db.replay_target(1).unwrap();
    let path = Path::new(target.file_path.as_ref().unwrap());
    assert!(path.is_absolute() && path.is_file());
    assert_eq!(db.list_replays(1).unwrap()[0].duration_frames, Some(18000));
}

#[test]
fn missing_import_path_is_an_error_in_both_api_and_cli() {
    let f = Fixture::new();
    let mut db = Db::open(f.file("catalogue.sqlite3")).unwrap();
    assert!(ingest::ingest_path(&mut db, &f.file("missing")).is_err());
    assert!(!f.cli(&["import", "missing"]).status.success());
}

#[test]
fn partial_import_reports_failures_and_exits_nonzero() {
    let f = Fixture::new();
    fs::write(f.file("valid.KWReplay"), replay(&roster(&[(-1, 6)]), 900)).unwrap();
    fs::write(f.file("invalid.KWReplay"), b"invalid").unwrap();
    let output = f.cli(&["import", "."]);
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["inserted"], 1);
    assert_eq!(report["errors"].as_array().unwrap().len(), 1);
    assert!(report["errors"][0]["path"]
        .as_str()
        .unwrap()
        .ends_with("invalid.KWReplay"));
}

#[test]
fn reimport_refreshes_legacy_metadata_without_reordering_or_duplicating() {
    let f = Fixture::new();
    seed(&f, "existing", &[(-1, 6), (-1, 9), (-1, 3)], 18000);
    let conn = rusqlite::Connection::open(f.file("catalogue.sqlite3")).unwrap();
    conn.execute("UPDATE replays SET file_path='existing.KWReplay', duration_frames=NULL, n_players=3, imported_at=100", []).unwrap();
    conn.execute("UPDATE players SET actual_faction='Rnd' WHERE slot=0", [])
        .unwrap();
    assert_eq!(
        json(f.cli(&["import", "existing.KWReplay"]))["duplicates"],
        1
    );
    let db = Db::open(f.file("catalogue.sqlite3")).unwrap();
    let rows = db.list_replays(-1).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].imported_at, 100);
    assert_eq!(rows[0].n_players, 2);
    assert_eq!(rows[0].duration_frames, Some(18000));
    assert_eq!(rows[0].players[0].actual_faction, "GDI");
    assert!(Path::new(rows[0].file_path.as_ref().unwrap()).is_absolute());
}

#[test]
fn mode_filters_and_statistics_use_teams_without_spectators() {
    let f = Fixture::new();
    seed(&f, "ffa4", &[(-1, 6); 4], 18000);
    seed(&f, "ffa8", &[(-1, 6); 8], 18000);
    seed(&f, "2v2", &[(0, 6), (0, 6), (1, 9), (1, 9)], 18000);
    seed(&f, "3v1", &[(0, 6), (0, 6), (0, 6), (1, 9)], 18000);
    seed(&f, "1v1-commentator", &[(-1, 6), (-1, 9), (-1, 3)], 18000);
    let ffa = json(f.cli(&["search", "--mode", "ffa"]));
    assert_eq!(ffa.as_array().unwrap().len(), 2);
    let team = json(f.cli(&["search", "--mode", "2v2"]));
    assert_eq!(team.as_array().unwrap().len(), 1);
    assert!(team[0]["file_path"]
        .as_str()
        .unwrap()
        .ends_with("2v2.KWReplay"));
    let duel = json(f.cli(&["search", "--mode", "1v1"]));
    assert_eq!(duel[0]["n_players"], 2);
    let stats = json(f.cli(&["stats"]));
    assert!(stats["modes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["mode"] == "1v1" && m["n_players"] == 2 && m["count"] == 1));
    assert!(stats["modes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["mode"] == "3v1"));
}

#[test]
fn duration_filters_use_fifteen_ticks_per_second() {
    let f = Fixture::new();
    seed(&f, "twenty-minutes", &[(-1, 6), (-1, 9)], 18000);
    assert_eq!(
        json(f.cli(&["search", "--min-minutes", "15"]))
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        json(f.cli(&["search", "--max-minutes", "15"]))
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert!(!f
        .cli(&["search", "--min-minutes", "9223372036854775807"])
        .status
        .success());
}

#[test]
fn keyset_pages_reach_old_rows_without_tie_duplicates_or_new_imports() {
    let f = Fixture::new();
    let db = Db::open(f.file("catalogue.sqlite3")).unwrap();
    let mut conn = rusqlite::Connection::open(f.file("catalogue.sqlite3")).unwrap();
    let tx = conn.transaction().unwrap();
    for id in 1..=2505 {
        tx.execute("INSERT INTO replays (id,file_hash,map_name,n_players,timestamp,imported_at) VALUES (?1,?2,'Map',0,1,100)", rusqlite::params![id, id.to_string()]).unwrap();
    }
    tx.commit().unwrap();
    let mut ids = Vec::new();
    let mut before = None;
    loop {
        let page = db.list_replays_before(1000, before).unwrap();
        let Some(last) = page.last() else { break };
        before = Some(ReplayCursor {
            imported_at: last.imported_at,
            id: last.id,
        });
        ids.extend(page.iter().map(|r| r.id));
        if ids.len() == 1000 {
            conn.execute("INSERT INTO replays (file_hash,map_name,n_players,timestamp,imported_at) VALUES ('new','New map',0,1,101)", []).unwrap();
        }
    }
    assert_eq!(ids, (1..=2505).rev().collect::<Vec<_>>());
}
