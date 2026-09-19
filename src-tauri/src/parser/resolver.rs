//! Walk the command stream and fill in each player's `actual_faction`
//! when they picked Random in the lobby.
//!
//! Strategy: for every player whose `chosen_faction == Random`, find their
//! first command of type `0x22D` (queue) or `0x231` (placedown) — the template
//! hash in the payload uniquely identifies their sub-faction via the
//! generated `faction_table`.

use std::io::Read;

use super::commands::walk_commands;
use super::error::Result;
use super::faction_table::resolve_template_to_faction;
use super::reader::R;
use super::types::{Faction, Player};

/// Walk the command stream starting at the reader's current position and
/// mutate `players` in place to fill in `actual_faction` for any player
/// whose `chosen_faction` is Random. Returns the highest `time_code`
/// observed (= the replay's total simulation-tick count), which the caller
/// can convert to seconds via `max_tc / TICKS_PER_SECOND` (15 ticks per second).
///
/// **Always walks the whole stream** even when every Random player has
/// already been resolved — we need the final `time_code` for duration.
pub fn resolve_actual_factions<T: Read>(r: &mut R<'_, T>, players: &mut [Player]) -> Result<u32> {
    // Pre-compute which player slots still need faction resolution.
    let mut pending: std::collections::HashSet<i32> = players
        .iter()
        .filter(|p| matches!(p.chosen_faction, Faction::Random) && !p.is_observer)
        .map(|p| p.slot as i32)
        .collect();

    // Walking the chunk stream can hit EOF mid-chunk on a small number of
    // truncated / malformed replays from the corpus. Treat that as "best-
    // effort" rather than failing the whole parse — the metadata is still
    // valid and any factions resolved before the truncation remain useful,
    // but the duration is unknown.
    let res = walk_commands(r, &[0x22D, 0x231], |cmd| {
        if pending.is_empty() {
            return;
        }
        let pid = cmd.player_id;
        if !pending.contains(&pid) {
            return;
        }
        let hash = cmd
            .queue_template_hash()
            .or_else(|| cmd.placedown_template_hash());
        let Some(h) = hash else { return };
        let Some(f) = resolve_template_to_faction(h) else {
            return;
        };
        if let Some(p) = players.iter_mut().find(|p| p.slot as i32 == pid) {
            p.actual_faction = f;
            pending.remove(&pid);
        }
    });
    match res {
        Ok(max_tc) => Ok(max_tc),
        Err(super::error::ParseError::Eof { .. }) => Ok(0), // truncated body, no reliable duration
        Err(e) => Err(e),
    }
}
