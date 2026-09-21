//! SQLite schema for the Silo catalogue.
//!
//! Run-once on every connection open via `apply`. Idempotent — all DDL is
//! `IF NOT EXISTS`.

use rusqlite::Connection;

pub fn apply(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS replays (
            id              INTEGER PRIMARY KEY,
            file_hash       TEXT NOT NULL UNIQUE,
            file_path       TEXT,
            file_size       INTEGER,
            game            TEXT,
            version_major   INTEGER,
            version_minor   INTEGER,
            build_major     INTEGER,
            build_minor     INTEGER,
            title           TEXT,
            description     TEXT,
            map_name        TEXT,
            map_id          TEXT,
            map_path        TEXT,
            map_crc         TEXT,
            timestamp       INTEGER,
            n_players       INTEGER,
            raw_header      TEXT,
            imported_at     INTEGER DEFAULT (strftime('%s','now'))
        );

        CREATE INDEX IF NOT EXISTS idx_replays_map        ON replays(map_name);
        CREATE INDEX IF NOT EXISTS idx_replays_timestamp  ON replays(timestamp);
        CREATE INDEX IF NOT EXISTS idx_replays_n_players  ON replays(n_players);

        -- Additive migration: `duration_frames` was added in 2026-05.
        -- `ALTER TABLE … ADD COLUMN` doesn't support IF NOT EXISTS, so we
        -- check sqlite_master first. New databases get it via the table
        -- definition below already including it (after the migration runs
        -- on any existing db, the column is present going forward).
        "#,
    )?;

    let has_column: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('replays') WHERE name = 'duration_frames'",
        [],
        |r| r.get(0),
    )?;
    if has_column == 0 {
        conn.execute("ALTER TABLE replays ADD COLUMN duration_frames INTEGER", [])?;
    }

    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_replays_duration   ON replays(duration_frames);

        CREATE TABLE IF NOT EXISTS players (
            id              INTEGER PRIMARY KEY,
            replay_id       INTEGER NOT NULL REFERENCES replays(id) ON DELETE CASCADE,
            slot            INTEGER,
            name            TEXT,
            clan            TEXT,
            chosen_faction  TEXT,
            actual_faction  TEXT,
            team            INTEGER,
            color           INTEGER,
            handicap        INTEGER,
            is_ai           INTEGER,
            is_observer     INTEGER,
            is_commentator  INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_players_replay     ON players(replay_id);
        CREATE INDEX IF NOT EXISTS idx_players_name       ON players(name COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_players_actual     ON players(actual_faction);

        CREATE TABLE IF NOT EXISTS tags (
            replay_id       INTEGER NOT NULL REFERENCES replays(id) ON DELETE CASCADE,
            tag             TEXT NOT NULL,
            PRIMARY KEY (replay_id, tag)
        );
        CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
        "#,
    )
}
