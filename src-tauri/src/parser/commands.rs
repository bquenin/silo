//! Walk the chunk stream that follows the header and decode just enough
//! commands to recover each player's actual faction when they picked Random.
//!
//! Binary layout (KW):
//!
//!   chunk:
//!     time_code:  u32  (0x7FFFFFFF terminates the body)
//!     ty:         u8
//!     size:       u32
//!     data:       [size bytes]
//!     unknown:    u32
//!
//! When `ty == 1`, `data` is:
//!     one:    u8 (== 1)
//!     ncmd:   u32
//!     payload: split into `ncmd` commands separated by 0xFF.
//!
//! Each command in the payload is:
//!     cmd_id:    u8
//!     pid_byte:  u8 → player_id = (pid_byte / 8) - 3
//!     content:   bytes up to and including the next 0xFF
//!
//! Of interest:
//!   * `cmd_id == 0x2D` (queue / resume production) — content[8..12] LE u32 is
//!     the unit template hash.
//!   * `cmd_id == 0x31` (placedown) — content[6..10] LE u32 is the building
//!     template hash.

use std::io::Read;

use super::error::Result;
use super::reader::R;

const CMD_QUEUE: u8 = 0x2D;
const CMD_PLACEDOWN: u8 = 0x31;
const END_MARKER: u32 = 0x7FFF_FFFF;

#[derive(Debug, Clone)]
pub struct Command {
    pub cmd_id: u8,
    pub player_id: i32,
    pub time_code: u32,
    pub payload: Vec<u8>,
}

impl Command {
    /// For QUEUE (0x2D) commands, the unit template hash is at content[8..12].
    pub fn queue_template_hash(&self) -> Option<u32> {
        if self.cmd_id == CMD_QUEUE && self.payload.len() >= 12 {
            Some(u32::from_le_bytes([
                self.payload[8],
                self.payload[9],
                self.payload[10],
                self.payload[11],
            ]))
        } else {
            None
        }
    }

    /// For PLACEDOWN (0x31) commands, the building template hash is at content[6..10].
    pub fn placedown_template_hash(&self) -> Option<u32> {
        if self.cmd_id == CMD_PLACEDOWN && self.payload.len() >= 10 {
            Some(u32::from_le_bytes([
                self.payload[6],
                self.payload[7],
                self.payload[8],
                self.payload[9],
            ]))
        } else {
            None
        }
    }
}

/// Walk the chunk stream. `for_each_cmd` is invoked for every decoded command
/// of interest (we only emit cmd_ids the caller cares about — passing an empty
/// filter emits everything). Returns the highest non-sentinel `time_code` seen
/// across the whole body, which equals the game's total simulation-tick count
/// (the engine writes one chunk per sim tick; KW's logical tick rate is 30 Hz
/// at game-speed 100, so wall-clock seconds = max_time_code / 30).
pub fn walk_commands<T: Read, F>(
    r: &mut R<'_, T>,
    cmd_filter: &[u8],
    mut for_each_cmd: F,
) -> Result<u32>
where
    F: FnMut(Command),
{
    let mut max_tc: u32 = 0;
    loop {
        let time_code = match r.read_u32_le() {
            Ok(v) => v,
            // Cleanly stop at EOF — the body has ended without an explicit marker.
            Err(_) => break,
        };
        if time_code == END_MARKER {
            break;
        }
        if time_code > max_tc {
            max_tc = time_code;
        }

        let ty = r.read_u8()?;
        let size = r.read_u32_le()?;
        let mut data = vec![0u8; size as usize];
        r.read_exact(&mut data)?;
        let _unknown = r.read_u32_le()?;

        if ty != 1 {
            continue;
        }
        if data.is_empty() {
            continue;
        }
        // data[0] is the leading 0x01 sentinel; bail if it's something else.
        if data[0] != 1 {
            continue;
        }
        if data.last() != Some(&0xFF) {
            // Some chunks have a weird tail — Python parser also skips these.
            continue;
        }
        // Next 4 bytes are ncmd. Then payload begins.
        if data.len() < 5 {
            continue;
        }
        let ncmd = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        let payload = &data[5..];

        split_commands(payload, ncmd, time_code, cmd_filter, &mut for_each_cmd);
    }
    Ok(max_tc)
}

/// Walk the bytes between command boundaries.
///
/// FSM mirroring the Python `Chunk.split_commands` exactly:
///   - mode CMD_ID:   1 byte → cmd_id
///   - mode PID:      1 byte → player_id = (byte / 8) - 3 (KW/CNC3)
///   - mode CONTENT:  collect bytes until 0xFF (inclusive)
fn split_commands<F: FnMut(Command)>(
    payload: &[u8],
    ncmd: u32,
    time_code: u32,
    filter: &[u8],
    mut for_each: F,
) {
    #[derive(Copy, Clone, PartialEq)]
    enum Mode { Cmd, Pid, Content }
    let mut mode = Mode::Cmd;
    let mut current = Command { cmd_id: 0, player_id: 0, time_code, payload: Vec::new() };
    let mut start = 0usize;
    let mut emitted = 0u32;

    for (i, &byte) in payload.iter().enumerate() {
        match mode {
            Mode::Cmd => {
                current = Command { cmd_id: byte, player_id: 0, time_code, payload: Vec::new() };
                mode = Mode::Pid;
            }
            Mode::Pid => {
                current.player_id = (byte as i32 / 8) - 3;
                start = i + 1;
                mode = Mode::Content;
            }
            Mode::Content => {
                if byte == 0xFF {
                    let end = i + 1;
                    current.payload = payload[start..end].to_vec();
                    if filter.is_empty() || filter.contains(&current.cmd_id) {
                        for_each(current.clone());
                    }
                    emitted += 1;
                    if ncmd != 1 {
                        mode = Mode::Cmd;
                    }
                }
            }
        }
        if emitted >= ncmd && mode == Mode::Cmd {
            break;
        }
    }
}
