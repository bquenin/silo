//! SQLite-backed catalogue.

mod schema;

use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{params, Connection, OpenFlags};

use crate::parser::{Faction, Player, Replay};

/// Open a Tacitus catalogue at `path`. Creates the file + schema if missing.
pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        schema::apply(&conn)?;
        Ok(Self { conn })
    }

    /// Insert (or ignore — by file_hash) a replay and its players.
    /// Returns Ok(true) if inserted, Ok(false) if duplicate.
    pub fn insert_replay(&mut self, hash: &str, file_size: u64, r: &Replay) -> Result<bool> {
        let tx = self.conn.transaction()?;

        // Check for existing row by hash
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM replays WHERE file_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .ok();
        if let Some(_id) = existing {
            // Update file_path to the most-recent known location, then bail.
            if let Some(p) = &r.file_path {
                tx.execute(
                    "UPDATE replays SET file_path = ?1 WHERE file_hash = ?2",
                    params![p.to_string_lossy(), hash],
                )?;
            }
            tx.commit()?;
            return Ok(false);
        }

        tx.execute(
            "INSERT INTO replays (
                file_hash, file_path, file_size, game,
                version_major, version_minor, build_major, build_minor,
                title, description, map_name, map_id, map_path, map_crc,
                timestamp, n_players, raw_header, duration_frames
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            params![
                hash,
                r.file_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
                file_size as i64,
                r.game,
                r.version.0,
                r.version.1,
                r.version.2,
                r.version.3,
                r.title,
                r.description,
                r.map_name,
                r.map_id,
                r.map_path,
                r.map_crc,
                r.timestamp,
                r.players.len() as i64,
                r.raw_header,
                r.duration_frames,
            ],
        )?;
        let replay_id = tx.last_insert_rowid();

        for p in &r.players {
            tx.execute(
                "INSERT INTO players (
                    replay_id, slot, name, clan, chosen_faction, actual_faction,
                    team, color, handicap, is_ai, is_observer, is_commentator
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    replay_id,
                    p.slot,
                    p.name,
                    p.clan,
                    p.chosen_faction.short(),
                    p.actual_faction.short(),
                    p.team,
                    p.color,
                    p.handicap,
                    p.is_ai as i64,
                    p.is_observer as i64,
                    p.is_commentator as i64,
                ],
            )?;
        }
        tx.commit()?;
        Ok(true)
    }

    pub fn count_replays(&self) -> Result<i64> {
        let n: i64 = self.conn.query_row("SELECT COUNT(*) FROM replays", [], |row| row.get(0))?;
        Ok(n)
    }

    /// List the first `limit` replays, newest-imported first, with their
    /// player rosters attached (filtered to non-observer humans + AI so the
    /// UI gets the actual matchup view).
    pub fn list_replays(&self, limit: i64) -> Result<Vec<ReplayRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_hash, file_path, map_name, n_players, timestamp,
                    imported_at, duration_frames
             FROM replays
             ORDER BY imported_at DESC
             LIMIT ?1",
        )?;
        let mut rows: Vec<ReplayRow> = stmt
            .query_map(params![limit], |row| {
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
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if rows.is_empty() {
            return Ok(rows);
        }

        // Build the IN (...) list and fetch all players in one query.
        let ids: Vec<String> = rows.iter().map(|r| r.id.to_string()).collect();
        let placeholders = vec!["?"; ids.len()].join(",");
        let q = format!(
            "SELECT replay_id, slot, name, clan, chosen_faction, actual_faction,
                    team, is_ai, is_observer, is_commentator
             FROM players WHERE replay_id IN ({})
             ORDER BY replay_id, slot",
            placeholders
        );
        let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut by_id: std::collections::HashMap<i64, Vec<PlayerSummary>> =
            std::collections::HashMap::new();
        {
            let mut pstmt = self.conn.prepare(&q)?;
            let prows = pstmt.query_map(params.as_slice(), |row| {
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
            })?;
            for r in prows {
                let (rid, p) = r?;
                by_id.entry(rid).or_default().push(p);
            }
        }
        for r in &mut rows {
            if let Some(v) = by_id.remove(&r.id) {
                r.players = v;
            }
        }
        Ok(rows)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReplayRow {
    pub id: i64,
    pub file_hash: String,
    pub file_path: Option<String>,
    pub map_name: String,
    pub n_players: i64,
    pub timestamp: i64,
    pub imported_at: i64,
    /// Total simulation ticks (max time_code from chunk walk). `None` for
    /// replays parsed before the duration column was added, until they're
    /// backfilled via `tacitus backfill-duration`. Convert to seconds via
    /// `frames / 30` (KW logical tick rate at game-speed 100).
    pub duration_frames: Option<i64>,
    pub players: Vec<PlayerSummary>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerSummary {
    pub slot: i64,
    pub name: String,
    pub clan: String,
    pub chosen_faction: String,
    pub actual_faction: String,
    pub team: i64,
    pub is_ai: bool,
    pub is_observer: bool,
    pub is_commentator: bool,
}

/// Suppress unused-import warnings while we ramp up the API surface.
#[allow(dead_code)]
fn _suppress() -> (Faction, Player, PathBuf) {
    (Faction::Random, Player {
        slot: 0, name: String::new(), clan: String::new(),
        chosen_faction: Faction::Random, actual_faction: Faction::Random,
        team: 0, color: 0, handicap: 0,
        is_ai: false, is_observer: false, is_commentator: false,
    }, PathBuf::new())
}
