//! Smoke tests against actual `.kwreplay` files in SILO_REPLAY_CORPUS.
//!
//! Skipped when the environment variable is unset. The optional collection
//! should include matches where players chose Random.

mod common;

use std::path::PathBuf;

use silo_lib::parser;

#[test]
fn parses_one_replay() {
    let Some(root) = common::corpus_dir() else {
        return;
    };
    let mut sample: Option<PathBuf> = None;
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.unwrap();
        if entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("kwreplay"))
        {
            sample = Some(entry.path().to_path_buf());
            break;
        }
    }
    let sample = sample.expect("no .kwreplay file in corpus");
    let r = parser::parse_metadata(&sample).expect("parse failed");

    println!("file: {}", sample.display());
    println!("magic: {:?}", r.magic);
    println!("game: {}  version: {:?}", r.game, r.version);
    println!("map_name: {:?}", r.map_name);
    println!("map_path: {:?}", r.map_path);
    println!("map_crc: {:?}", r.map_crc);
    println!("timestamp: {}", r.timestamp);
    println!("players ({}):", r.players.len());
    for p in &r.players {
        println!(
            "  slot {} {:?} faction={} team={} clan={:?} ai={} obs={}",
            p.slot,
            p.name,
            p.chosen_faction.short(),
            p.team,
            p.clan,
            p.is_ai,
            p.is_observer
        );
    }

    assert!(r.map_name.len() > 0, "map name should be non-empty");
    assert!(r.players.len() > 0, "should have at least one player");
}

#[test]
fn resolves_random_factions() {
    let Some(root) = common::corpus_dir() else {
        return;
    };
    // Pick the first replay that has at least one Random human.
    let mut tried = 0;
    let mut resolved_count = 0;
    let mut still_random = 0;
    for entry in walkdir::WalkDir::new(&root) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("kwreplay"))
        {
            continue;
        }
        tried += 1;
        let r = match parser::parse_full(entry.path()) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let has_random = r
            .players
            .iter()
            .any(|p| matches!(p.chosen_faction, parser::Faction::Random) && !p.is_observer);
        if !has_random {
            continue;
        }
        for p in &r.players {
            if !matches!(p.chosen_faction, parser::Faction::Random) || p.is_observer {
                continue;
            }
            if matches!(p.actual_faction, parser::Faction::Random) {
                still_random += 1;
            } else {
                resolved_count += 1;
            }
        }
        if tried > 200 {
            break;
        } // sample
    }
    println!(
        "resolved {} random→faction; {} still random",
        resolved_count, still_random
    );
    assert!(resolved_count > 0, "no random players resolved at all");
    // Vast majority should resolve.
    let total = resolved_count + still_random;
    assert!(
        (resolved_count as f64) / (total as f64) >= 0.8,
        "low resolution rate: {} / {}",
        resolved_count,
        total
    );
}

#[test]
fn parses_many_replays() {
    let Some(root) = common::corpus_dir() else {
        return;
    };
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut errors: Vec<(PathBuf, String)> = Vec::new();
    for entry in walkdir::WalkDir::new(&root) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("kwreplay"))
        {
            continue;
        }
        total += 1;
        match parser::parse_metadata(entry.path()) {
            Ok(_) => ok += 1,
            Err(e) => errors.push((entry.path().to_path_buf(), e.to_string())),
        }
    }
    println!("parsed {} / {} replays OK", ok, total);
    for (p, e) in errors.iter().take(10) {
        println!("  FAIL {}: {}", p.display(), e);
    }
    assert!(total > 0, "no replays found");
    // Allow up to 10% failures for malformed / non-KW files (some .cnc3replay
    // may have ended up in the corpus).
    let max_fail = (total as f64 * 0.10) as usize;
    assert!(
        errors.len() <= max_fail.max(5),
        "too many failures: {} / {}",
        errors.len(),
        total
    );
}
