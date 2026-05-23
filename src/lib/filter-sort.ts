import type { Replay } from './types';

export type SortKey = 'recorded' | 'map' | 'players' | 'length';
export type SortDir = 'asc' | 'desc';

export interface ModeFilter {
  /** null = "All", otherwise the exact n_players to match. */
  n_players: number | null;
}

export interface FilterState {
  search: string;
  mode: ModeFilter;
  /** Empty = no faction filter. Otherwise show only replays where *any*
   *  player's actual_faction is in this set. */
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

export const MODE_OPTIONS: { label: string; n_players: number | null }[] = [
  { label: 'All', n_players: null },
  { label: '1v1', n_players: 2 },
  { label: '2v2', n_players: 4 },
  { label: '3v3', n_players: 6 },
  { label: '4v4', n_players: 8 },
];

export const FACTIONS: string[] = ['GDI','Nod','Sc','BH','MoK','ST','ZCM','R17','T59'];

export function applyFilters(replays: Replay[], state: FilterState): Replay[] {
  const q = state.search.trim().toLowerCase();
  return replays.filter((r) => {
    if (state.mode.n_players != null && r.n_players !== state.mode.n_players) return false;
    if (state.factions.size > 0) {
      const hit = r.players.some((p) => state.factions.has(String(p.actual)));
      if (!hit) return false;
    }
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
        return sign * (tb - ta);
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
        return sign * (lb - la);
      });
      break;
  }
  return sorted;
}
