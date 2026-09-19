//! Read KW's typed command batches. Only a 0xFF at an argument-tag position
//! terminates a command; scalar values and coordinates may contain any byte.
//! The native serializer stores an 11-bit message type and 5-bit player index
//! in a little-endian word, followed by runs of typed arguments.

use std::io::Read;

use super::error::{ParseError, Result};
use super::reader::R;

const CMD_QUEUE: u16 = 0x22D;
const CMD_PLACEDOWN: u16 = 0x231;
const END_MARKER: u32 = 0x7FFF_FFFF;
const MAX_CHUNK_BYTES: usize = 16 * 1024 * 1024;
const MAX_COMMANDS: usize = 65_536;

#[derive(Debug)]
pub struct Command<'a> {
    pub message_type: u16,
    pub player_id: i32,
    pub payload: &'a [u8],
}

impl Command<'_> {
    pub fn queue_template_hash(&self) -> Option<u32> {
        (self.message_type == CMD_QUEUE)
            .then(|| self.payload.get(8..12))
            .flatten()
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn placedown_template_hash(&self) -> Option<u32> {
        (self.message_type == CMD_PLACEDOWN)
            .then(|| self.payload.get(6..10))
            .flatten()
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

/// Returns the highest completed chunk's simulation tick (15 ticks/second).
/// Faction recovery is best effort: unsupported argument types invalidate the
/// entire batch, never trigger a scan for guessed command boundaries.
pub fn walk_commands<T: Read, F>(
    r: &mut R<'_, T>,
    command_filter: &[u16],
    mut for_each_command: F,
) -> Result<u32>
where
    F: FnMut(Command<'_>),
{
    let mut max_tick = 0;
    loop {
        let tick = match r.read_u32_le() {
            Ok(tick) => tick,
            Err(ParseError::Eof { .. }) => break,
            Err(error) => return Err(error),
        };
        if tick == END_MARKER {
            break;
        }
        let kind = r.read_u8()?;
        let size = r.read_u32_le()? as usize;
        if size > MAX_CHUNK_BYTES {
            return Err(ParseError::LimitExceeded {
                field: "chunk",
                length: size,
                limit: MAX_CHUNK_BYTES,
            });
        }
        if kind == 1 {
            let mut data = vec![0; size];
            r.read_exact(&mut data)?;
            r.read_u32_le()?; // chunk trailer
                              // Validate the entire batch before exposing faction evidence.
            if let Ok(commands) = split_commands(&data, command_filter) {
                for command in commands {
                    for_each_command(command);
                }
            }
        } else {
            r.skip(size)?;
            r.read_u32_le()?;
        }
        max_tick = max_tick.max(tick);
    }
    Ok(max_tick)
}

fn split_commands<'a>(data: &'a [u8], filter: &[u16]) -> Result<Vec<Command<'a>>> {
    let malformed = || ParseError::BadBody("Malformed command batch".into());
    if data.len() < 5 || data[0] != 1 {
        return Err(malformed());
    }
    let count = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
    if count > MAX_COMMANDS || count > (data.len() - 5) / 3 {
        return Err(malformed());
    }
    let mut offset = 5;
    let mut commands = Vec::new();
    for _ in 0..count {
        let header = data.get(offset..offset + 2).ok_or_else(malformed)?;
        let header = u16::from_le_bytes(header.try_into().unwrap());
        offset += 2;
        let start = offset;
        loop {
            let tag = *data.get(offset).ok_or_else(malformed)?;
            offset += 1;
            if tag == 0xFF {
                break;
            }
            // Native argument serializers: INT, REAL, BOOL, OID, UINT32,
            // LOCATION and UINT16. Other types are not guessed.
            let width = match tag & 0x0F {
                0 | 1 | 3 | 5 | 9 => 4,
                2 => 1,
                6 => 12,
                10 => 2,
                other => {
                    return Err(ParseError::BadBody(format!(
                        "Unsupported argument tag {other}"
                    )))
                }
            };
            offset += (usize::from(tag >> 4) + 1) * width;
            if offset > data.len() {
                return Err(malformed());
            }
        }
        let message_type = header & 0x7FF;
        if filter.is_empty() || filter.contains(&message_type) {
            commands.push(Command {
                message_type,
                player_id: i32::from(header >> 11) - 3,
                payload: &data[start..offset],
            });
        }
    }
    if offset != data.len() {
        return Err(malformed());
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batch(commands: &[Vec<u8>]) -> Vec<u8> {
        let mut data = vec![1];
        data.extend_from_slice(&(commands.len() as u32).to_le_bytes());
        data.extend(commands.iter().flatten());
        data
    }

    #[test]
    fn embedded_separator_in_any_coordinate_preserves_the_following_queue() {
        for index in 0..12 {
            let mut location = vec![0x47, 0x1A, 6];
            location.extend([0; 12]);
            location[3 + index] = 0xFF;
            location.push(0xFF);
            let queue = vec![
                0x2D, 0x1A, 0, 1, 0, 0, 0, 2, 0, 3, 0x8D, 0x53, 0x26, 0, 0xFF,
            ];
            let data = batch(&[location, queue]);
            let commands = split_commands(&data, &[CMD_QUEUE]).unwrap();
            assert_eq!(commands.len(), 1);
            assert_eq!(commands[0].player_id, 0);
            assert_eq!(commands[0].queue_template_hash(), Some(0x0026538D));
        }
    }

    #[test]
    fn truncated_unknown_or_miscounted_batches_supply_no_commands() {
        for command in [vec![0x47, 0x1A, 6, 0xFF], vec![0x47, 0x1A, 0x0E, 0xFF]] {
            assert!(split_commands(&batch(&[command]), &[]).is_err());
        }
        let mut data = batch(&[vec![0x4C, 0x1A, 0xFF]]);
        data[1] = 2;
        assert!(split_commands(&data, &[]).is_err());
        data[1] = 1;
        data.push(0);
        assert!(split_commands(&data, &[]).is_err());
    }

    #[test]
    fn full_message_type_prevents_low_byte_aliases() {
        let data = batch(&[vec![0x2D, 0x1B, 0xFF]]);
        assert!(split_commands(&data, &[CMD_QUEUE]).unwrap().is_empty());
    }
}
