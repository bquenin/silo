import type { Replay } from './types';
import { modeOf } from './replays';

export type SortKey = 'recorded' | 'map' | 'players' | 'length';
export type SortDir = 'asc' | 'desc';

export interface FilterState {
  search: string;
  mode: string | null;
  /** Empty = no faction filter. Otherwise every selected faction must
   *  appear as at least one player's actual faction. */
  factions: Set<string>;
}

export interface SortState {
  key: SortKey;
  dir: SortDir;
}

export const SORT_LABELS: Record<SortKey, string> = {
  recorded: 'Date',
  map: 'Map name',
  players: 'Player count',
  length: 'Length',
};

export const MODE_OPTIONS: { label: string; value: string | null }[] = [
  { label: 'All', value: null },
  { label: '1v1', value: '1v1' },
  { label: '2v2', value: '2v2' },
  { label: '3v3', value: '3v3' },
  { label: '4v4', value: '4v4' },
  { label: 'FFA', value: 'FFA' },
];

export const FACTION_GROUPS = [
  { label: 'GDI', factions: ['GDI', 'ST', 'ZCM'] },
  { label: 'NOD', factions: ['Nod', 'MoK', 'BH'] },
  { label: 'SCRIN', factions: ['Sc', 'T59', 'R17'] },
];

export function applyFilters(replays: Replay[], state: FilterState): Replay[] {
  const q = state.search.trim().toLowerCase();
  const factions = [...state.factions];
  return replays.filter((r) => {
    if (state.mode != null && modeOf(r) !== state.mode) return false;
    if (!factions.every((faction) => r.players.some((p) => p.actual === faction))) return false;
    if (q) {
      const inMap = r.map.toLowerCase().includes(q);
      const inPlayers = r.players.some((p) => p.name.toLowerCase().includes(q));
      const inFile = r.file?.toLowerCase().includes(q);
      if (!inMap && !inPlayers && !inFile) return false;
    }
    return true;
  });
}

export function applySort(replays: Replay[], sort: SortState): Replay[] {
  const sign = sort.dir === 'asc' ? 1 : -1;
  const sorted = [...replays];
  switch (sort.key) {
    case 'recorded':
      sorted.sort((a, b) => {
        const ta = a.recorded_at ? Date.parse(a.recorded_at) : 0;
        const tb = b.recorded_at ? Date.parse(b.recorded_at) : 0;
        return sign * (ta - tb);
      });
      break;
    case 'map':
      sorted.sort((a, b) => sign * a.map.localeCompare(b.map, 'en', { sensitivity: 'base' }));
      break;
    case 'players':
      sorted.sort((a, b) => sign * (a.n_players - b.n_players));
      break;
    case 'length':
      sorted.sort((a, b) => {
        const la = a.duration_s ?? 0;
        const lb = b.duration_s ?? 0;
        return sign * (la - lb);
      });
      break;
  }
  return sorted;
}
