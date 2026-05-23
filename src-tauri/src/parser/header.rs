//! Read the binary header at the top of a `.KWReplay` file and the
//! ASCII key/value block that carries player roster + map path.

use std::io::Read;

use super::error::{ParseError, Result};
use super::reader::R;
use super::types::{Faction, Player, Replay};

const KW_MAGIC_SIZE: usize = 18;
const KW_U1_SIZE: usize = 33;
const KW_U2_SIZE: usize = 19;

/// Convenience wrapper — read header from a stream that doesn't need further
/// processing.
pub fn read_header<T: Read>(inner: &mut T) -> Result<Replay> {
    let mut r = R::new(inner);
    read_header_into(&mut r)
}

/// Read the full pre-chunk preamble. After this returns, `r` is positioned at
/// the start of the chunk stream so the caller can keep reading commands.
pub fn read_header_into<T: Read>(r: &mut R<'_, T>) -> Result<Replay> {
    let magic = r.read_cstr(KW_MAGIC_SIZE)?;
    if !magic.contains("REPLAY") && !magic.starts_with("C&C3") {
        return Err(ParseError::BadMagic { got: magic.clone() });
    }

    // game_network_info reads a single byte (5 = internet game, 4 = network,
    // anything else = skirmish). No extra bytes follow.
    let _hnumber1 = r.read_u8()?;

    let vermajor = r.read_u32_le()?;
    let verminor = r.read_u32_le()?;
    let buildmajor = r.read_u32_le()?;
    let buildminor = r.read_u32_le()?;

    let title = r.read_tb_str(None)?;
    let description = r.read_tb_str(None)?;
    let map_name = r.read_tb_str(None)?;
    let map_id = r.read_tb_str(None)?;

    // Per-slot id+name (+ team for internet games). We don't currently need
    // these values — the richer roster is in the ASCII `S=` block below.
    let player_cnt = r.read_u8()?;
    for _ in 0..=player_cnt {
        let _id = r.read_u32_le()?;
        let _name = r.read_tb_str(None)?;
        if _hnumber1 == 5 {
            let _team = r.read_u8()?;
        }
    }

    let _offset = r.read_u32_le()?;
    let repl_length = r.read_u32_le()?;
    let _repl_magic = r.read_cstr(repl_length as usize)?;

    let timestamp = r.read_u32_le()?;
    r.skip(KW_U1_SIZE)?;

    let header_len = r.read_u32_le()?;
    let raw_header = r.read_cstr(header_len as usize)?;

    // Tail of the pre-chunk preamble. We consume but ignore.
    let _replay_saver = r.read_u8()?;
    let _zero3 = r.read_u32_le()?;
    let _zero4 = r.read_u32_le()?;
    let filename_length = r.read_u32_le()?;
    let _filename = r.read_tb_str(Some(filename_length as usize))?;
    let _date_time = r.read_tb_str(Some(8))?;
    let vermagic_len = r.read_u32_le()?;
    let _vermagic = r.read_cstr(vermagic_len as usize)?;
    let _magic_hash = r.read_u32_le()?;
    let _zero5 = r.read_u8()?;
    r.skip(KW_U2_SIZE * 4)?;

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
        duration_frames: None,
    })
}

/// Decode the `key=value;key=value;…` ASCII header.
fn decode_header(header: &str) -> Result<(String, String, Vec<Player>)> {
    let mut map_path = String::new();
    let mut map_crc = String::new();
    let mut players: Vec<Player> = Vec::new();

    for pair in header.split(';') {
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
        actual_faction: Faction::Random,
        team: -1,
        color: -1,
        handicap: 0,
        is_ai: kind == 'C',
        is_observer: false,
        is_commentator: false,
    };

    if kind == 'H' {
        // Human: 0:H<name> 1:ip 2:? 3:tt_or_ft 4:color 5:faction 6:? 7:team 8:hcap 9:? 10:? 11:clan
        if parts.len() >= 6 {
            p.color = parts[4].parse().unwrap_or(-1);
            let fac_raw: i32 = parts[5].parse().unwrap_or(1);
            p.chosen_faction = Faction::from_raw(fac_raw).unwrap_or(Faction::Random);
            p.actual_faction = p.chosen_faction;
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
        // AI: 0:C<difficulty> 1:color 2:faction 3:? 4:team 5:handicap 6:personality
        if parts.len() >= 3 {
            p.color = parts[1].parse().unwrap_or(-1);
            let fac_raw: i32 = parts[2].parse().unwrap_or(1);
            p.chosen_faction = Faction::from_raw(fac_raw).unwrap_or(Faction::Random);
            p.actual_faction = p.chosen_faction;
        }
        if parts.len() >= 5 {
            p.team = parts[4].parse::<i32>().unwrap_or(-1) + 1;
        }
        if parts.len() >= 6 {
            p.handicap = parts[5].parse().unwrap_or(0);
        }
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
