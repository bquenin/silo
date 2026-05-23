import { useCallback, useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';

import {
  BackendReplayRow,
  IngestReport,
  countReplays,
  ingestPath,
  isTauri,
  listReplays,
} from './backend';
import { REPLAYS as MOCK_REPLAYS } from './mock-data';
import { Replay } from './types';

/** Adapt a `BackendReplayRow` to the Replay shape the UI components expect. */
function rowToReplay(row: BackendReplayRow): Replay {
  const realPlayers = row.players
    .filter((p) => !p.is_observer && !p.is_commentator)
    .sort((a, b) => a.slot - b.slot);

  const players = realPlayers.map((p) => ({
    slot: p.slot,
    name: p.name,
    chosen: p.chosen_faction,
    actual: p.actual_faction,
    team: p.team,
  }));

  // Group by team. Team values come from the replay header where team=-1 means
  // "no team" (FFA). Players with the same team number stand on the same side.
  // For FFA / no-team games, each player is their own "team".
  const teamMap = new Map<number, typeof players>();
  for (const p of players) {
    const t = p.team != null && p.team >= 1 ? p.team : -(p.slot + 100); // FFA → unique synthetic team
    if (!teamMap.has(t)) teamMap.set(t, []);
    teamMap.get(t)!.push(p);
  }
  const teams = [...teamMap.values()].sort((a, b) =>
    (a[0]?.slot ?? 0) - (b[0]?.slot ?? 0)
  );

  // 1v1 convenience fields (used for the cover gradient on row + detail pane)
  let bro_alias: string | undefined;
  let bro_actual: string | undefined;
  let opponent_name: string | undefined;
  let opponent_actual: string | undefined;
  if (teams.length === 2 && teams[0].length === 1 && teams[1].length === 1) {
    bro_alias = teams[0][0].name;
    bro_actual = teams[0][0].actual;
    opponent_name = teams[1][0].name;
    opponent_actual = teams[1][0].actual;
  }

  return {
    id: String(row.id),
    file: row.file_path?.split(/[\\/]/).pop() ?? row.file_hash.slice(0, 12),
    map: row.map_name,
    n_players: realPlayers.length || row.n_players,
    players,
    teams,
    bro_alias,
    bro_actual,
    opponent_name,
    opponent_actual,
    recorded_at: row.timestamp ? new Date(row.timestamp * 1000).toISOString() : undefined,
  };
}

export interface ReplaysHookValue {
  replays: Replay[];
  total: number;
  loading: boolean;
  importFolder: () => Promise<IngestReport | null>;
  refresh: () => Promise<void>;
  /** True when the hook is reading from the live backend (vs mock data). */
  live: boolean;
}

export function useReplays(): ReplaysHookValue {
  const live = isTauri();
  const [replays, setReplays] = useState<Replay[]>(live ? [] : MOCK_REPLAYS);
  const [total, setTotal] = useState<number>(live ? 0 : MOCK_REPLAYS.length);
  const [loading, setLoading] = useState<boolean>(live);

  const refresh = useCallback(async () => {
    if (!live) return;
    setLoading(true);
    try {
      const [rows, n] = await Promise.all([listReplays(2000), countReplays()]);
      setReplays(rows.map(rowToReplay));
      setTotal(n);
    } finally {
      setLoading(false);
    }
  }, [live]);

  useEffect(() => {
    if (live) {
      void refresh();
    }
  }, [live, refresh]);

  const importFolder = useCallback(async (): Promise<IngestReport | null> => {
    if (!live) return null;
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return null;
    setLoading(true);
    try {
      const report = await ingestPath(selected);
      await refresh();
      return report;
    } finally {
      setLoading(false);
    }
  }, [live, refresh]);

  return { replays, total, loading, importFolder, refresh, live };
}
