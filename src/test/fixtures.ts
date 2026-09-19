import type { BackendReplayRow } from '../lib/backend';

export function row(teams: number[], id = 1): BackendReplayRow {
  return {
    id, file_path: `replay-${id}.KWReplay`, file_hash: String(id),
    map_name: `Map ${id}`, n_players: teams.length, timestamp: 1700000000,
    imported_at: 1, duration_frames: 18000,
    players: teams.map((team, slot) => ({
      slot, team, name: `Player${slot}`, clan: '', chosen_faction: 'GDI',
      actual_faction: 'GDI', is_ai: false, is_observer: false, is_commentator: false,
    })),
  };
}
