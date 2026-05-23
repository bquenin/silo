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

mod error;
mod header;
mod reader;
mod types;

pub use error::{ParseError, Result};
pub use types::{Faction, Player, Replay};

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Parse the metadata portion of a `.KWReplay` file at `path`.
///
/// Does not walk the command stream — that's a separate pass (see
/// `crate::resolver::resolve_actual_factions`).
pub fn parse_metadata(path: impl AsRef<Path>) -> Result<Replay> {
    let path = path.as_ref();
    let f = File::open(path).map_err(ParseError::Io)?;
    let mut r = BufReader::new(f);
    let mut replay = header::read_header(&mut r)?;
    replay.file_path = Some(path.to_path_buf());
    Ok(replay)
}
