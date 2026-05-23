//! Binary parser for `.KWReplay` files.
//!
//! Format reference: KWReplayAutoSaver's `kwreplay.py`.
//!
//! The file layout (KW only — CNC3/RA3 differ slightly and aren't supported here):
//!
//! - magic: 18 bytes ASCII
//! - hnumber1: uint32 (network info; if == 5, 8 more bytes follow)
//! - version: 4× uint32 (vermajor, verminor, buildmajor, buildminor)
//! - title, desc, map_name, map_id: UTF-16 LE null-terminated strings
//! - player_cnt: 1 byte; player_cnt + 1 player records follow
//!     - each: uint32 id, UTF-16 LE name, optional 1 byte team (if hnumber1==5)
//! - offset (u32), repl_length (u32 == 8), repl_magic (8 bytes)
//! - timestamp (u32 unix seconds)
//! - unknown1 (33 bytes)
//! - header_len (u32), header (length bytes ASCII) — k=v pairs separated by ';',
//!   the `S=` pair lists players as ':'-separated `H<name>,...` records
//! - … plus more fields we don't currently need.
//!
//! For the catalogue we extract: magic, version, map_name, map_path, timestamp,
//! the `S=` player roster (chosen faction, team, color, clan), and the original
//! filename.

mod commands;
mod error;
mod faction_table;
mod header;
mod reader;
mod resolver;
mod types;

pub use error::{ParseError, Result};
pub use types::{Faction, Player, Replay};

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Parse the metadata header only — fast (single linear scan, no chunk walk).
pub fn parse_metadata(path: impl AsRef<Path>) -> Result<Replay> {
    let path = path.as_ref();
    let f = File::open(path).map_err(ParseError::Io)?;
    let mut r = BufReader::new(f);
    let mut replay = header::read_header(&mut r)?;
    replay.file_path = Some(path.to_path_buf());
    Ok(replay)
}

/// Parse the metadata header AND walk the command stream to resolve any
/// Random-faction players' actual assignment from their first build command.
/// Also records the replay's total simulation-tick count as
/// `duration_frames` (max `time_code` seen during the walk). Convert to
/// wall-clock seconds via `frames / 30` (KW's logical tick rate at
/// game-speed 100).
///
/// More expensive than `parse_metadata` (reads ~the full file once) but
/// produces complete data — use this for ingest.
pub fn parse_full(path: impl AsRef<Path>) -> Result<Replay> {
    let path = path.as_ref();
    let f = File::open(path).map_err(ParseError::Io)?;
    let mut buf = BufReader::new(f);
    let mut r = reader::R::new(&mut buf);
    let mut replay = header::read_header_into(&mut r)?;
    replay.file_path = Some(path.to_path_buf());
    let max_tc = resolver::resolve_actual_factions(&mut r, &mut replay.players)?;
    replay.duration_frames = Some(max_tc);
    Ok(replay)
}

/// A queue (0x2D) or placedown (0x31) command, projected to just the
/// fields used for build-order analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildEvent {
    /// Simulation tick at game-speed 100. Seconds = `time_code / 30`.
    pub time_code: u32,
    pub player_slot: i32,
    /// `"queue"` for unit production starts, `"placedown"` for structure placement.
    pub kind: String,
    /// Template hash from the command payload.
    pub template_hash: u32,
}

/// Walk a replay's command stream and emit every queue (0x2D) and
/// placedown (0x31) command as a `BuildEvent`. Stops at `max_frames`
/// when set (e.g. `Some(300 * 30)` = first 5 minutes).
///
/// Useful for opening-build analysis without loading the full Replay
/// metadata.
pub fn extract_build_order(
    path: impl AsRef<Path>,
    max_frames: Option<u32>,
) -> Result<Vec<BuildEvent>> {
    let path = path.as_ref();
    let f = File::open(path).map_err(ParseError::Io)?;
    let mut buf = BufReader::new(f);
    let mut r = reader::R::new(&mut buf);
    // Burn through the header — read_header_into already does this and
    // leaves the cursor at the start of the chunk stream. We don't need
    // the parsed Replay here, but extracting it for free wouldn't hurt.
    let _ = header::read_header_into(&mut r)?;

    let mut out: Vec<BuildEvent> = Vec::new();
    let _ = commands::walk_commands(&mut r, &[0x2D, 0x31], |cmd| {
        if let Some(cap) = max_frames {
            if cmd.time_code > cap {
                return;
            }
        }
        let h = cmd.queue_template_hash().or_else(|| cmd.placedown_template_hash());
        let Some(template_hash) = h else { return };
        out.push(BuildEvent {
            time_code: cmd.time_code,
            player_slot: cmd.player_id,
            kind: if cmd.cmd_id == 0x2D { "queue" } else { "placedown" }.into(),
            template_hash,
        });
    });
    Ok(out)
}
