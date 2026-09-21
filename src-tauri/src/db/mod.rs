//! SQLite-backed catalogue.

mod schema;

use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use crate::parser::{Faction, Player, Replay};

/// Open a Silo catalogue at `path`. Creates the file + schema if missing.
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

    /// Insert a replay and its players, or refresh parsed fields by file_hash.
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
            .optional()?;
        if let Some(id) = existing {
            // Reimporting applies parser fixes while preserving identity,
            // import order, and any user annotations.
            tx.execute(
                "UPDATE replays SET file_path = COALESCE(?1, file_path),
                    duration_frames = COALESCE(?2, duration_frames), n_players = ?3
                 WHERE id = ?4",
                params![
                    r.file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned()),
                    r.duration_frames.filter(|&ticks| ticks > 0),
                    r.players
                        .iter()
                        .filter(|p| !p.is_observer && !p.is_commentator)
                        .count() as i64,
                    id,
                ],
            )?;
            for player in &r.players {
                if player.actual_faction != Faction::Random {
                    tx.execute(
                        "UPDATE players SET actual_faction = ?1 WHERE replay_id = ?2 AND slot = ?3",
                        params![player.actual_faction.short(), id, player.slot],
                    )?;
                }
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
                r.file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
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
                r.players
                    .iter()
                    .filter(|p| !p.is_observer && !p.is_commentator)
                    .count() as i64,
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
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM replays", [], |row| row.get(0))?;
        Ok(n)
    }

    pub fn replay_target(&self, id: i64) -> Result<crate::playback::ReplayTarget> {
        Ok(self.conn.query_row(
            "SELECT file_path, file_hash, map_name, map_path, map_crc,
                    version_major, version_minor, build_major, build_minor,
                    (SELECT COUNT(*) FROM players
                     WHERE replay_id = replays.id
                       AND COALESCE(is_observer, 0) = 0
                       AND COALESCE(is_commentator, 0) = 0)
             FROM replays WHERE id = ?1",
            params![id],
            |row| {
                Ok(crate::playback::ReplayTarget {
                    id,
                    file_path: row.get(0)?,
                    file_hash: row.get(1)?,
                    map_name: row.get(2)?,
                    map_path: row.get(3)?,
                    map_crc: row.get(4)?,
                    version: [row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?],
                    n_players: row.get(9)?,
                })
            },
        )?)
    }

    /// List the first `limit` replays, newest-imported first, with their
    /// player rosters attached. Participant counts exclude spectators; roster
    /// flags allow callers to display or omit them.
    pub fn list_replays(&self, limit: i64) -> Result<Vec<ReplayRow>> {
        self.list_replays_before(limit, None)
    }

    pub fn list_replays_before(
        &self,
        limit: i64,
        before: Option<ReplayCursor>,
    ) -> Result<Vec<ReplayRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_hash, file_path, map_name, n_players, timestamp,
                    imported_at, duration_frames
             FROM replays
             WHERE ?2 IS NULL OR imported_at < ?2 OR (imported_at = ?2 AND id < ?3)
             ORDER BY imported_at DESC, id DESC
             LIMIT ?1",
        )?;
        let mut rows: Vec<ReplayRow> = stmt
            .query_map(
                params![
                    limit,
                    before.as_ref().map(|c| c.imported_at),
                    before.as_ref().map(|c| c.id)
                ],
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

        if rows.is_empty() {
            return Ok(rows);
        }

        // Bound SQLite's parameter count even for an unlimited CLI listing.
        for page in rows.chunks_mut(1000) {
            let ids: Vec<String> = page.iter().map(|r| r.id.to_string()).collect();
            let placeholders = vec!["?"; ids.len()].join(",");
            let q = format!(
                "SELECT replay_id, slot, name, clan, chosen_faction, actual_faction,
                    team, is_ai, is_observer, is_commentator
             FROM players WHERE replay_id IN ({})
             ORDER BY replay_id, slot",
                placeholders
            );
            let params: Vec<&dyn rusqlite::ToSql> =
                ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

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
            for r in page {
                if let Some(v) = by_id.remove(&r.id) {
                    r.players = v;
                }
                r.n_players = r
                    .players
                    .iter()
                    .filter(|p| !p.is_observer && !p.is_commentator)
                    .count() as i64;
            }
        }
        Ok(rows)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplayCursor {
    pub imported_at: i64,
    pub id: i64,
}

/// Team composition of active participants; spectators never define a mode.
pub fn mode_of(players: &[PlayerSummary]) -> String {
    let mut teams = std::collections::HashMap::new();
    for p in players
        .iter()
        .filter(|p| !p.is_observer && !p.is_commentator)
    {
        let key = if p.team > 0 {
            (true, p.team)
        } else {
            (false, p.slot)
        };
        *teams.entry(key).or_insert(0usize) += 1;
    }
    let mut sizes: Vec<_> = teams.into_values().collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    if sizes.len() >= 3 && sizes.iter().all(|&n| n == 1) {
        "FFA".into()
    } else if sizes.len() >= 2 {
        sizes
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("v")
    } else {
        format!("{}p", sizes.iter().sum::<usize>())
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
    /// backfilled via `silo backfill-duration`. Convert to seconds via
    /// `frames / TICKS_PER_SECOND` (15 ticks per second).
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
    (
        Faction::Random,
        Player {
            slot: 0,
            name: String::new(),
            clan: String::new(),
            chosen_faction: Faction::Random,
            actual_faction: Faction::Random,
            team: 0,
            color: 0,
            handicap: 0,
            is_ai: false,
            is_observer: false,
            is_commentator: false,
        },
        PathBuf::new(),
    )
}
