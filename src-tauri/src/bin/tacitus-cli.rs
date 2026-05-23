//! `tacitus` CLI — import, search, and play replays from the command line.
//!
//! Subcommands all accept `--json` for machine-readable output.
//!
//!   tacitus import <PATH>           — walk a file or dir, insert .kwreplay
//!   tacitus ls [--limit N]          — list catalogue entries
//!   tacitus count                   — total replays in catalogue
//!   tacitus parse <FILE>            — parse one file, print metadata
//!   tacitus parse --full <FILE>     — parse + resolve Random factions
//!   tacitus stats                   — high-level corpus aggregates
//!
//! Common flags:
//!   --db PATH       — override the catalogue path (default OS-appropriate)
//!   --json          — emit JSON instead of human-friendly text

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};

use tacitus_lib::db::Db;
use tacitus_lib::ingest;
use tacitus_lib::parser;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {:#}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    if args.len() < 2 || matches!(args[1].as_str(), "-h" | "--help" | "help") {
        print_usage();
        return Ok(());
    }

    // Parse global flags + figure out subcommand position.
    let mut json = false;
    let mut db_path: Option<PathBuf> = None;
    let mut rest: Vec<&str> = Vec::new();
    let mut i = 1;
    let mut subcmd: Option<String> = None;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--json" => json = true,
            "--db" => {
                i += 1;
                db_path = Some(PathBuf::from(args.get(i).context("--db needs a path")?));
            }
            s if subcmd.is_none() => subcmd = Some(s.into()),
            other => rest.push(other),
        }
        i += 1;
    }
    let subcmd = subcmd.ok_or_else(|| anyhow!("missing subcommand"))?;

    let db_path = db_path.unwrap_or_else(ingest::default_catalogue_path);

    match subcmd.as_str() {
        "import" => cmd_import(&db_path, &rest, json),
        "ls" => cmd_ls(&db_path, &rest, json),
        "count" => cmd_count(&db_path, json),
        "parse" => cmd_parse(&rest, json),
        "stats" => cmd_stats(&db_path, json),
        other => Err(anyhow!("unknown subcommand: {}", other)),
    }
}

fn print_usage() {
    println!(
        "{}\n",
        r#"tacitus — KW replay corpus manager (CLI)

USAGE:
    tacitus [--db PATH] [--json] <SUBCOMMAND> [ARGS]

SUBCOMMANDS:
    import <PATH>       Walk PATH (file or dir) and ingest every .kwreplay
    ls [--limit N]      List replays, newest first (default 20)
    count               Print total replays in catalogue
    parse <FILE>        Parse one file; print map + players + factions
    parse --full FILE   Same as parse, but also resolve Random factions
    stats               Per-faction / per-matchup corpus aggregates

GLOBAL FLAGS:
    --db PATH           Use catalogue at PATH (default: OS-appropriate)
    --json              Emit JSON output instead of human-friendly text"#
    );
}

fn cmd_import(db_path: &PathBuf, rest: &[&str], json: bool) -> Result<()> {
    let path = rest.first().ok_or_else(|| anyhow!("usage: tacitus import <PATH>"))?;
    let mut db = Db::open(db_path)?;
    let report = ingest::ingest_path(&mut db, &PathBuf::from(path))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "scanned {} | inserted {} | duplicates {} | errors {}",
            report.scanned, report.inserted, report.duplicates, report.errors.len()
        );
        for e in &report.errors {
            println!("  ERR {}: {}", e.path, e.message);
        }
    }
    Ok(())
}

fn cmd_ls(db_path: &PathBuf, rest: &[&str], json: bool) -> Result<()> {
    let mut limit: i64 = 20;
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--limit" {
            i += 1;
            limit = rest.get(i).ok_or_else(|| anyhow!("--limit needs N"))?.parse()?;
        }
        i += 1;
    }
    let db = Db::open(db_path)?;
    let rows = db.list_replays(limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for r in &rows {
            println!(
                "[{:>4}] {} · {}p · {}",
                r.id,
                truncate(&r.map_name, 40),
                r.n_players,
                r.file_path.as_deref().unwrap_or("?")
            );
        }
    }
    Ok(())
}

fn cmd_count(db_path: &PathBuf, json: bool) -> Result<()> {
    let db = Db::open(db_path)?;
    let n = db.count_replays()?;
    if json {
        println!("{{\"count\": {}}}", n);
    } else {
        println!("{}", n);
    }
    Ok(())
}

fn cmd_parse(rest: &[&str], json: bool) -> Result<()> {
    let mut full = false;
    let mut file: Option<&str> = None;
    for arg in rest {
        if *arg == "--full" { full = true; }
        else if file.is_none() { file = Some(*arg); }
    }
    let file = file.ok_or_else(|| anyhow!("usage: tacitus parse [--full] <FILE>"))?;
    let replay = if full {
        parser::parse_full(file)?
    } else {
        parser::parse_metadata(file)?
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&replay)?);
    } else {
        println!("file:      {:?}", replay.file_path.as_deref().unwrap_or(std::path::Path::new("")));
        println!("map:       {}", replay.map_name);
        println!("path:      {}", replay.map_path);
        println!("version:   {}.{}.{}.{}",
            replay.version.0, replay.version.1, replay.version.2, replay.version.3);
        println!("timestamp: {}", replay.timestamp);
        println!("players ({}):", replay.players.len());
        for p in &replay.players {
            let actual = if p.chosen_faction.short() != p.actual_faction.short() {
                format!(" → {}", p.actual_faction.short())
            } else {
                String::new()
            };
            println!(
                "  slot {} {:<24} faction={}{}  team={} clan={:?}{}",
                p.slot,
                truncate(&p.name, 24),
                p.chosen_faction.short(),
                actual,
                p.team,
                p.clan,
                if p.is_ai { " (AI)" } else { "" },
            );
        }
    }
    Ok(())
}

fn cmd_stats(db_path: &PathBuf, json: bool) -> Result<()> {
    use rusqlite::Connection;
    let conn = Connection::open(db_path)?;

    let mut total: i64 = 0;
    conn.query_row("SELECT COUNT(*) FROM replays", [], |r| {
        total = r.get(0)?;
        Ok(())
    })?;

    let mut mode_counts: Vec<(i64, i64)> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT n_players, COUNT(*) FROM replays GROUP BY n_players ORDER BY COUNT(*) DESC")?;
        let rows = stmt.query_map([], |r| Ok::<_, rusqlite::Error>((r.get(0)?, r.get(1)?)))?;
        for row in rows {
            mode_counts.push(row?);
        }
    }

    let mut faction_counts: Vec<(String, i64)> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT actual_faction, COUNT(*) FROM players
             WHERE is_observer=0 AND is_commentator=0
             GROUP BY actual_faction ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |r| Ok::<_, rusqlite::Error>((r.get(0)?, r.get(1)?)))?;
        for row in rows {
            faction_counts.push(row?);
        }
    }

    let mut top_maps: Vec<(String, i64)> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT map_name, COUNT(*) FROM replays GROUP BY map_name ORDER BY COUNT(*) DESC LIMIT 15",
        )?;
        let rows = stmt.query_map([], |r| Ok::<_, rusqlite::Error>((r.get(0)?, r.get(1)?)))?;
        for row in rows {
            top_maps.push(row?);
        }
    }

    if json {
        let payload = serde_json::json!({
            "total": total,
            "modes": mode_counts.iter().map(|(n, c)| serde_json::json!({"n_players": n, "count": c})).collect::<Vec<_>>(),
            "factions": faction_counts.iter().map(|(f, c)| serde_json::json!({"faction": f, "count": c})).collect::<Vec<_>>(),
            "top_maps": top_maps.iter().map(|(m, c)| serde_json::json!({"map": m, "count": c})).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Total replays:  {}", total);
        println!("\nGame modes:");
        for (n, c) in &mode_counts {
            let mode = match n { 2 => "1v1", 3 => "FFA", 4 => "2v2", 6 => "3v3", 8 => "4v4", _ => "other" };
            println!("  {:>5}  {} ({}p)", c, mode, n);
        }
        println!("\nFactions (actual):");
        for (f, c) in &faction_counts {
            println!("  {:>5}  {}", c, f);
        }
        println!("\nTop maps:");
        for (m, c) in &top_maps {
            println!("  {:>5}  {}", c, truncate(m, 50));
        }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max - 1).collect();
        t.push('…');
        t
    }
}
