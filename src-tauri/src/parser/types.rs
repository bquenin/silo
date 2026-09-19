use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Lobby-chosen faction (the engine-side raw value, before Random is resolved).
///
/// Values match KWReplayAutoSaver's `kw_faction_tab`:
///   1=Rnd, 2=Obs, 3=PostCommentator, 4=?, 5=?,
///   6=GDI, 7=SteelTalons, 8=ZOCOM, 9=Nod, 10=BlackHand,
///   11=MarkedOfKane, 12=Scrin, 13=Reaper17, 14=Traveler59
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    Random,
    Observer,
    PostCommentator,
    Unknown4,
    Unknown5,
    Gdi,
    SteelTalons,
    Zocom,
    Nod,
    BlackHand,
    MarkedOfKane,
    Scrin,
    Reaper17,
    Traveler59,
}

impl Faction {
    pub fn from_raw(v: i32) -> Option<Self> {
        Some(match v {
            1 => Self::Random,
            2 => Self::Observer,
            3 => Self::PostCommentator,
            4 => Self::Unknown4,
            5 => Self::Unknown5,
            6 => Self::Gdi,
            7 => Self::SteelTalons,
            8 => Self::Zocom,
            9 => Self::Nod,
            10 => Self::BlackHand,
            11 => Self::MarkedOfKane,
            12 => Self::Scrin,
            13 => Self::Reaper17,
            14 => Self::Traveler59,
            _ => return None,
        })
    }

    /// Short label, matches the strings the Python parser produces (`decode_faction`).
    pub fn short(&self) -> &'static str {
        match self {
            Self::Random => "Rnd",
            Self::Observer => "Obs",
            Self::PostCommentator => "PostCommentator",
            Self::Unknown4 => "f3",
            Self::Unknown5 => "f4",
            Self::Gdi => "GDI",
            Self::SteelTalons => "ST",
            Self::Zocom => "ZCM",
            Self::Nod => "Nod",
            Self::BlackHand => "BH",
            Self::MarkedOfKane => "MoK",
            Self::Scrin => "Sc",
            Self::Reaper17 => "R17",
            Self::Traveler59 => "T59",
        }
    }
}

/// A roster slot in the replay's `S=` header pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub slot: u32,
    pub name: String,
    pub clan: String,
    /// What the player picked in the lobby (Random / Gdi / etc.).
    pub chosen_faction: Faction,
    /// What the engine actually assigned them. Equals `chosen_faction`
    /// unless `chosen_faction == Random` and we successfully resolved it
    /// from the first build command (see `resolver::resolve_actual_factions`).
    pub actual_faction: Faction,
    pub team: i32,
    pub color: i32,
    pub handicap: i32,
    pub is_ai: bool,
    pub is_observer: bool,
    pub is_commentator: bool,
}

/// Parsed metadata for one `.KWReplay` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub file_path: Option<PathBuf>,
    pub magic: String,
    pub game: String,
    pub version: (u32, u32, u32, u32),
    pub title: String,
    pub description: String,
    pub map_name: String,
    pub map_id: String,
    pub map_path: String,
    pub map_crc: String,
    pub timestamp: u32,
    pub players: Vec<Player>,
    /// Raw header string after the magic + version block; useful for debugging
    /// and for the `S=` / `M=` decoding.
    pub raw_header: String,
    /// Total simulation ticks in the body's command stream — the highest
    /// `time_code` we observed. `None` if the metadata-only parse path
    /// was used (header.rs doesn't walk the body). Convert to wall-clock
    /// seconds via `frames / TICKS_PER_SECOND` (15 ticks per second).
    /// `Some(0)` means the body was truncated / unwalkable.
    #[serde(default)]
    pub duration_frames: Option<u32>,
}
