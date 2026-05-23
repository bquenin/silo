//! Walk the command stream and fill in each player's `actual_faction`
//! when they picked Random in the lobby.
//!
//! Strategy: for every player whose `chosen_faction == Random`, find their
//! first command of type `0x2D` (queue) or `0x31` (placedown) — the template
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
/// whose `chosen_faction` is Random.
///
/// Stops walking once every Random player has been resolved (early exit).
pub fn resolve_actual_factions<T: Read>(r: &mut R<'_, T>, players: &mut [Player]) -> Result<()> {
    // Pre-compute which player slots still need resolution.
    let mut pending: std::collections::HashSet<i32> = players
        .iter()
        .filter(|p| matches!(p.chosen_faction, Faction::Random) && !p.is_observer)
        .map(|p| p.slot as i32)
        .collect();
    if pending.is_empty() {
        return Ok(());
    }

    // Walking the chunk stream can hit EOF mid-chunk on a small number of
    // truncated / malformed replays from the corpus. Treat that as "best-
    // effort" rather than failing the whole parse — the metadata is still
    // valid and we just don't get to resolve any Random players from this
    // particular file.
    let res = walk_commands(r, &[0x2D, 0x31], |cmd| {
        if pending.is_empty() {
            return;
        }
        let pid = cmd.player_id;
        if !pending.contains(&pid) {
            return;
        }
        let hash = cmd.queue_template_hash().or_else(|| cmd.placedown_template_hash());
        let Some(h) = hash else { return };
        let Some(f) = resolve_template_to_faction(h) else { return };
        if let Some(p) = players.iter_mut().find(|p| p.slot as i32 == pid) {
            p.actual_faction = f;
            pending.remove(&pid);
        }
    });
    if let Err(super::error::ParseError::Eof { .. }) = res {
        // soft-fail; keep whatever we resolved
    } else {
        res?;
    }
    Ok(())
}
