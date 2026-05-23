//! `tacitus` CLI — import, search, and play replays from the command line.
//!
//! Subcommands all accept `--json` for machine-readable output.
//!
//!   tacitus import <PATH>           — walk a file or dir, insert .kwreplay
//!   tacitus ls [--limit N]          — list catalogue entries
//!   tacitus search [filters...]     — filter the catalogue (see below)
//!   tacitus count                   — total replays in catalogue
//!   tacitus parse <FILE>            — parse one file, print metadata
//!   tacitus parse --full <FILE>     — parse + resolve Random factions
//!   tacitus stats                   — high-level corpus aggregates
//!   tacitus backfill-duration       — re-parse rows missing duration_frames
//!
//! Search filters (combinable; all narrow the result set):
//!   --player NAME       substring match on any player.name (case-insensitive)
//!   --map SUB           substring match on map_name (case-insensitive)
//!   --faction CODE      any player's actual_faction == CODE (GDI, Nod, Sc, BH, …)
//!   --mode 1v1|2v2|3v3|4v4|ffa
//!   --year YYYY         matches the replay's recorded timestamp
//!   --since YYYY-MM-DD  timestamp >= date (UTC)
//!   --until YYYY-MM-DD  timestamp <= date (UTC)
//!   --min-minutes N     duration_frames / 30 >= N*60
//!   --max-minutes N     duration_frames / 30 <= N*60
//!   --limit N           default 50
//!   --sort recorded|map|players|length  default recorded
//!   --order asc|desc                    default desc
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
        "search" => cmd_search(&db_path, &rest, json),
        "count" => cmd_count(&db_path, json),
        "parse" => cmd_parse(&rest, json),
        "stats" => cmd_stats(&db_path, json),
        "backfill-duration" => cmd_backfill_duration(&db_path, json),
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
    search [filters]    Filter catalogue (see --help for filter flags)
    count               Print total replays in catalogue
    parse <FILE>        Parse one file; print map + players + factions
    parse --full FILE   Same as parse, but also resolve Random factions
    stats               Per-faction / per-matchup corpus aggregates

SEARCH FLAGS (combinable):
    --player NAME       substring match on any player.name (case-insensitive)
    --map SUB           substring match on map_name (case-insensitive)
    --faction CODE      any player's actual_faction == CODE
    --mode 1v1|2v2|3v3|4v4|ffa
    --year YYYY
    --since YYYY-MM-DD
    --until YYYY-MM-DD
    --limit N           default 50
    --sort KEY          recorded (default) | map | players | length
    --order asc|desc    default desc

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

fn cmd_search(db_path: &PathBuf, rest: &[&str], json: bool) -> Result<()> {
    use rusqlite::{params_from_iter, Connection};
    use tacitus_lib::db::{PlayerSummary, ReplayRow};
    // Ensure additive migrations run on legacy catalogues before we
    // build queries that reference the new column.
    drop(Db::open(db_path)?);

    // -- parse flags --
    let mut player: Option<String> = None;
    let mut map: Option<String> = None;
    let mut faction: Option<String> = None;
    let mut mode: Option<i64> = None;
    let mut year: Option<i64> = None;
    let mut since: Option<i64> = None;
    let mut until: Option<i64> = None;
    let mut min_minutes: Option<i64> = None;
    let mut max_minutes: Option<i64> = None;
    let mut limit: i64 = 50;
    let mut sort_key = "recorded".to_string();
    let mut order = "desc".to_string();

    let parse_ymd = |s: &str| -> Result<i64> {
        // YYYY-MM-DD → unix timestamp (UTC midnight).
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(anyhow!("bad date {s:?}; expected YYYY-MM-DD"));
        }
        let (y, m, d): (i32, u32, u32) = (parts[0].parse()?, parts[1].parse()?, parts[2].parse()?);
        // simple UTC-midnight epoch via chrono
        let dt = chrono::NaiveDate::from_ymd_opt(y, m, d)
            .ok_or_else(|| anyhow!("invalid date {s:?}"))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        Ok(dt.timestamp())
    };

    let mut i = 0;
    while i < rest.len() {
        let f = rest[i];
        let next = |i: &mut usize, name: &str| -> Result<&str> {
            *i += 1;
            rest.get(*i).copied().ok_or_else(|| anyhow!("{} needs a value", name))
        };
        match f {
            "--player" => player = Some(next(&mut i, "--player")?.into()),
            "--map" => map = Some(next(&mut i, "--map")?.into()),
            "--faction" => faction = Some(next(&mut i, "--faction")?.into()),
            "--mode" => {
                let v = next(&mut i, "--mode")?;
                let n = match v.to_ascii_lowercase().as_str() {
                    "1v1" => 2, "ffa" => 3, "2v2" => 4, "3v3" => 6, "4v4" => 8,
                    other => other.parse::<i64>()
                        .map_err(|_| anyhow!("unknown mode {other:?}; use 1v1/2v2/3v3/4v4/ffa or n_players int"))?,
                };
                mode = Some(n);
            }
            "--year" => year = Some(next(&mut i, "--year")?.parse()?),
            "--since" => since = Some(parse_ymd(next(&mut i, "--since")?)?),
            "--until" => until = Some(parse_ymd(next(&mut i, "--until")?)?),
            "--min-minutes" => min_minutes = Some(next(&mut i, "--min-minutes")?.parse()?),
            "--max-minutes" => max_minutes = Some(next(&mut i, "--max-minutes")?.parse()?),
            "--limit" => limit = next(&mut i, "--limit")?.parse()?,
            "--sort" => sort_key = next(&mut i, "--sort")?.into(),
            "--order" => order = next(&mut i, "--order")?.to_ascii_lowercase(),
            other => return Err(anyhow!("unknown search flag {other:?}")),
        }
        i += 1;
    }

    let order_sql = if order == "asc" { "ASC" } else { "DESC" };
    let order_by = match sort_key.as_str() {
        "recorded" => format!("r.timestamp {order_sql}"),
        "map" => format!("r.map_name {order_sql}"),
        "players" => format!("r.n_players {order_sql}, r.timestamp DESC"),
        "length" => format!("r.duration_frames {order_sql}, r.timestamp DESC"),
        other => return Err(anyhow!("unknown --sort key {other:?}")),
    };

    // -- build query --
    let mut where_parts: Vec<String> = Vec::new();
    let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(p) = &player {
        where_parts.push(
            "EXISTS (SELECT 1 FROM players p WHERE p.replay_id = r.id \
             AND p.is_observer = 0 AND p.is_commentator = 0 \
             AND LOWER(p.name) LIKE LOWER(?))".into(),
        );
        binds.push(Box::new(format!("%{p}%")));
    }
    if let Some(m) = &map {
        where_parts.push("LOWER(r.map_name) LIKE LOWER(?)".into());
        binds.push(Box::new(format!("%{m}%")));
    }
    if let Some(f) = &faction {
        where_parts.push(
            "EXISTS (SELECT 1 FROM players p WHERE p.replay_id = r.id \
             AND p.actual_faction = ?)".into(),
        );
        binds.push(Box::new(f.clone()));
    }
    if let Some(n) = mode {
        // `r.n_players` is the raw slot count *including* observers/
        // post-commentators. The user-facing mode (1v1, 2v2, …) counts
        // only real human players, so we compute that on the fly.
        where_parts.push(
            "(SELECT COUNT(*) FROM players p WHERE p.replay_id = r.id \
             AND p.is_observer = 0 AND p.is_commentator = 0) = ?".into(),
        );
        binds.push(Box::new(n));
    }
    if let Some(y) = year {
        // year(ts) = ?  → use strftime
        where_parts.push("CAST(strftime('%Y', r.timestamp, 'unixepoch') AS INTEGER) = ?".into());
        binds.push(Box::new(y));
    }
    if let Some(s) = since {
        where_parts.push("r.timestamp >= ?".into());
        binds.push(Box::new(s));
    }
    if let Some(u) = until {
        // Inclusive end-of-day
        where_parts.push("r.timestamp <= ?".into());
        binds.push(Box::new(u + 86399));
    }
    if let Some(m) = min_minutes {
        // 30 ticks/sec at game-speed 100 → minutes * 60 * 30 frames.
        where_parts.push("r.duration_frames >= ?".into());
        binds.push(Box::new(m * 60 * 30));
    }
    if let Some(m) = max_minutes {
        where_parts.push("r.duration_frames <= ?".into());
        binds.push(Box::new(m * 60 * 30));
    }

    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };
    let sql = format!(
        "SELECT r.id, r.file_hash, r.file_path, r.map_name, r.n_players, \
                r.timestamp, r.imported_at, r.duration_frames \
         FROM replays r{where_sql} ORDER BY {order_by} LIMIT ?"
    );
    binds.push(Box::new(limit));

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<ReplayRow> = stmt
        .query_map(
            params_from_iter(binds.iter().map(|b| &**b as &dyn rusqlite::ToSql)),
            |row| {
                Ok(ReplayRow {
                    id: row.get(0)?,
                    file_hash: row.get(1)?,
                    file_path: row.get(2)?,
                    map_name: row.get(3)?,
                    n_players: row.get(4)?,
                    timestamp: row.get(5)?,
                    imported_at: row.get(6)?,
                    duration_frames: row.get(7)?,
                    players: Vec::new(),
                })
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Attach players for found replays (same shape as ls --json).
    let mut rows = rows;
    if !rows.is_empty() {
        let ids: Vec<String> = rows.iter().map(|r| r.id.to_string()).collect();
        let placeholders = vec!["?"; ids.len()].join(",");
        let pq = format!(
            "SELECT replay_id, slot, name, clan, chosen_faction, actual_faction, \
                    team, is_ai, is_observer, is_commentator \
             FROM players WHERE replay_id IN ({placeholders}) ORDER BY replay_id, slot"
        );
        let mut by_id: std::collections::HashMap<i64, Vec<PlayerSummary>> = std::collections::HashMap::new();
        let mut pstmt = conn.prepare(&pq)?;
        let prows = pstmt.query_map(
            params_from_iter(ids.iter().map(|s| s as &dyn rusqlite::ToSql)),
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    PlayerSummary {
                        slot: row.get(1)?,
                        name: row.get(2)?,
                        clan: row.get(3)?,
                        chosen_faction: row.get(4)?,
                        actual_faction: row.get(5)?,
                        team: row.get(6)?,
                        is_ai: row.get::<_, i64>(7)? != 0,
                        is_observer: row.get::<_, i64>(8)? != 0,
                        is_commentator: row.get::<_, i64>(9)? != 0,
                    },
                ))
            },
        )?;
        for r in prows {
            let (rid, p) = r?;
            by_id.entry(rid).or_default().push(p);
        }
        for r in &mut rows {
            if let Some(v) = by_id.remove(&r.id) {
                r.players = v;
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        println!("matched {} replay(s)", rows.len());
        for r in &rows {
            let dt = if r.timestamp > 0 {
                chrono::DateTime::<chrono::Utc>::from_timestamp(r.timestamp, 0)
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "—".into())
            } else {
                "—".into()
            };
            let humans: Vec<&str> = r.players.iter()
                .filter(|p| !p.is_observer && !p.is_commentator)
                .map(|p| p.name.as_str())
                .collect();
            // Duration → "MM:SS" at 30 frames/sec; "  —  " when missing.
            let dur = match r.duration_frames {
                Some(f) if f > 0 => {
                    let s = f / 30;
                    format!("{:>2}:{:02}", s / 60, s % 60)
                }
                _ => "  —  ".into(),
            };
            println!(
                "[{:>4}] {} {} · {}p · {} · {}",
                r.id,
                dt,
                dur,
                r.n_players,
                truncate(&r.map_name, 30),
                truncate(&humans.join(" vs "), 50)
            );
        }
    }
    Ok(())
}

fn cmd_backfill_duration(db_path: &PathBuf, json: bool) -> Result<()> {
    use rusqlite::Connection;
    // Open via Db first so the additive `duration_frames` migration runs on
    // pre-migration catalogues.
    drop(Db::open(db_path)?);
    let conn = Connection::open(db_path)?;

    // Pick rows where duration_frames is NULL and we still have a readable file_path.
    let mut targets: Vec<(i64, String)> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, file_path FROM replays \
             WHERE duration_frames IS NULL AND file_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            targets.push(row?);
        }
    }

    let mut ok = 0usize;
    let mut missing = 0usize;
    let mut parse_err = 0usize;
    let mut total_walked = 0usize;
    for (id, file_path) in &targets {
        total_walked += 1;
        let path = PathBuf::from(file_path);
        if !path.exists() {
            missing += 1;
            continue;
        }
        match parser::parse_full(&path) {
            Ok(replay) => {
                let frames = replay.duration_frames.unwrap_or(0);
                conn.execute(
                    "UPDATE replays SET duration_frames = ?1 WHERE id = ?2",
                    rusqlite::params![frames as i64, id],
                )?;
                ok += 1;
            }
            Err(_) => {
                parse_err += 1;
                // Mark with 0 so we don't re-attempt forever.
                let _ = conn.execute(
                    "UPDATE replays SET duration_frames = 0 WHERE id = ?1",
                    rusqlite::params![id],
                );
            }
        }
        if total_walked % 100 == 0 {
            eprintln!("  ... {total_walked}/{} processed", targets.len());
        }
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "candidates": targets.len(),
                "updated": ok,
                "file_missing": missing,
                "parse_error": parse_err,
            })
        );
    } else {
        println!(
            "candidates: {} | updated: {} | file_missing: {} | parse_error: {}",
            targets.len(),
            ok,
            missing,
            parse_err
        );
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
