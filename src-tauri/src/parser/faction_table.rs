//! Template hash → faction mapping for buildable game objects.
//!
//! The mapping is purely by template-name prefix (BlackHand* → BlackHand,
//! NOD* → Nod, etc.). This is sufficient because the engine uses
//! faction-specific copies of every buildable structure / unit, so the
//! first hash a player emits unambiguously identifies their actual side.

use super::types::Faction;

pub fn resolve_template_to_faction(hash: u32) -> Option<Faction> {
    TABLE.binary_search_by_key(&hash, |&(h, _)| h).ok().map(|i| TABLE[i].1)
}

const TABLE: &[(u32, Faction)] = include!("faction_table.in");
