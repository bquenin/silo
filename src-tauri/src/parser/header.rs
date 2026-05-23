//! Read the binary header at the top of a `.KWReplay` file and the
//! ASCII key/value block that carries player roster + map path.

use std::io::Read;

use super::error::{ParseError, Result};
use super::reader::R;
use super::types::{Faction, Player, Replay};

const KW_MAGIC_SIZE: usize = 18;
const KW_U1_SIZE: usize = 33;

pub fn read_header<T: Read>(inner: &mut T) -> Result<Replay> {
    let mut r = R::new(inner);

    let magic = r.read_cstr(KW_MAGIC_SIZE)?;
    if !magic.starts_with("C&C3") && !magic.starts_with("KW") && !magic.contains("REPLAY") {
        // KW magic in the file we've seen is "C&C3 REPLAY HEADER" (18 chars).
        return Err(ParseError::BadMagic { got: magic.clone() });
    }

    // network info: hnumber1 is ONE BYTE (read_byte in the Python parser, not
    // read_uint32). 5 = internet game, 4 = network game, otherwise skirmish.
    // No additional bytes follow regardless of value.
    let hnumber1 = r.read_u8()?;

    let vermajor = r.read_u32_le()?;
    let verminor = r.read_u32_le()?;
    let buildmajor = r.read_u32_le()?;
    let buildminor = r.read_u32_le()?;

    let title = r.read_tb_str(None)?;
    let description = r.read_tb_str(None)?;
    let map_name = r.read_tb_str(None)?;
    let map_id = r.read_tb_str(None)?;

    // Player slots in the binary header — just id + name (+ team if hn1==5).
    // The richer roster lives in the ASCII header below; we still skip these
    // properly so the file offset stays aligned.
    let player_cnt = r.read_u8()?;
    for _ in 0..=player_cnt {
        let _id = r.read_u32_le()?;
        let _name = r.read_tb_str(None)?;
        if hnumber1 == 5 {
            let _team = r.read_u8()?;
        }
    }
    // Suppress unused-var warning on hnumber1 when not internet game
    let _ = hnumber1;

    let _offset = r.read_u32_le()?;
    let repl_length = r.read_u32_le()?;
    let _repl_magic = r.read_cstr(repl_length as usize)?;

    // CNC3 / RA3 carry a 22-byte mod_info here. KW does not, so we skip the
    // branch. If we ever support CNC3/RA3 we'd add it.

    let timestamp = r.read_u32_le()?;
    r.skip(KW_U1_SIZE)?;

    let header_len = r.read_u32_le()?;
    let raw_header = r.read_cstr(header_len as usize)?;

    let (map_path, map_crc, players) = decode_header(&raw_header)?;

    Ok(Replay {
        file_path: None,
        magic,
        game: "KW".into(),
        version: (vermajor, verminor, buildmajor, buildminor),
        title,
        description,
        map_name,
        map_id,
        map_path,
        map_crc,
        timestamp,
        players,
        raw_header,
    })
}

/// Decode the `key=value;key=value;…` ASCII header.
///
/// Only `M=`, `MC=`, and `S=` are interesting for v1. The `S=` value is a
/// colon-separated list of player records like `H<name>,<ip>,...,<clan>`
/// for humans or `C<difficulty>,<color>,<faction>,...` for AI.
fn decode_header(header: &str) -> Result<(String, String, Vec<Player>)> {
    let mut map_path = String::new();
    let mut map_crc = String::new();
    let mut players: Vec<Player> = Vec::new();

    for pair in header.split(';') {
        // Some player names contain `=` so we must only split at the FIRST one,
        // per the Python parser's comment.
        let mut it = pair.splitn(2, '=');
        let (k, v) = match (it.next(), it.next()) {
            (Some(k), Some(v)) => (k, v),
            _ => continue,
        };
        match k {
            "M" => map_path = v.to_string(),
            "MC" => map_crc = v.to_string(),
            "S" => players = decode_player_roster(v)?,
            _ => {}
        }
    }

    Ok((map_path, map_crc, players))
}

fn decode_player_roster(s: &str) -> Result<Vec<Player>> {
    let mut out = Vec::new();
    for (slot, raw) in s.split(':').enumerate() {
        if raw.is_empty() {
            continue;
        }
        if let Some(p) = decode_player(raw, slot as u32)? {
            out.push(p);
        }
    }
    Ok(out)
}

fn decode_player(raw: &str, slot: u32) -> Result<Option<Player>> {
    // Split on commas; the first field carries the leading H or C prefix
    // bonded to the player name (e.g. "Hbike-RUsh+ownz+").
    let parts: Vec<&str> = raw.split(',').collect();
    if parts.is_empty() {
        return Ok(None);
    }
    let first = parts[0];
    let kind = match first.chars().next() {
        Some('H') => 'H',
        Some('C') => 'C',
        _ => return Ok(None),
    };
    let name = first[1..].to_string();

    let mut p = Player {
        slot,
        name,
        clan: String::new(),
        chosen_faction: Faction::Random,
        team: -1,
        color: -1,
        handicap: 0,
        is_ai: kind == 'C',
        is_observer: false,
        is_commentator: false,
    };

    if kind == 'H' {
        // Human layout (per Python `decode_human`):
        //   0:H<name> 1:ip 2:? 3:tt_or_ft 4:color 5:faction 6:? 7:team 8:hcap 9:? 10:? 11:clan
        if parts.len() >= 6 {
            p.color = parts[4].parse().unwrap_or(-1);
            let fac_raw: i32 = parts[5].parse().unwrap_or(1);
            p.chosen_faction = Faction::from_raw(fac_raw).unwrap_or(Faction::Random);
        }
        if parts.len() >= 8 {
            p.team = parts[7].parse::<i32>().unwrap_or(-1) + 1;
        }
        if parts.len() >= 9 {
            p.handicap = parts[8].parse().unwrap_or(0);
        }
        if parts.len() >= 12 {
            p.clan = parts[11].to_string();
        }
    } else {
        // AI layout (per Python `decode_ai`):
        //   0:C<difficulty> 1:color 2:faction 3:? 4:team 5:handicap 6:personality
        if parts.len() >= 3 {
            p.color = parts[1].parse().unwrap_or(-1);
            let fac_raw: i32 = parts[2].parse().unwrap_or(1);
            p.chosen_faction = Faction::from_raw(fac_raw).unwrap_or(Faction::Random);
        }
        if parts.len() >= 5 {
            p.team = parts[4].parse::<i32>().unwrap_or(-1) + 1;
        }
        if parts.len() >= 6 {
            p.handicap = parts[5].parse().unwrap_or(0);
        }
        // Map the AI difficulty code to a friendly name (CE/CM/CH/CB).
        p.name = match p.name.as_str() {
            "E" => "Easy (AI)".into(),
            "M" => "Medium (AI)".into(),
            "H" => "Hard (AI)".into(),
            "B" => "Brutal (AI)".into(),
            other => other.into(),
        };
    }

    p.is_observer = matches!(p.chosen_faction, Faction::Observer);
    p.is_commentator = matches!(p.chosen_faction, Faction::PostCommentator)
        || p.name.to_lowercase().contains("post commentator");

    Ok(Some(p))
}
