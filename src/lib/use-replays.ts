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
  // Filter to "real" players (no observers, no commentators) so the chips
  // show the actual matchup. Sort by slot for stable rendering.
  const realPlayers = row.players
    .filter((p) => !p.is_observer && !p.is_commentator)
    .sort((a, b) => a.slot - b.slot);

  const first = realPlayers[0];
  const second = realPlayers.find((_, i) => i > 0); // any second real player

  return {
    id: String(row.id),
    file: row.file_path?.split(/[\\/]/).pop() ?? row.file_hash.slice(0, 12),
    map: row.map_name,
    n_players: realPlayers.length || row.n_players,
    players: realPlayers.map((p) => ({
      slot: p.slot,
      name: p.name,
      chosen: p.chosen_faction,
      actual: p.actual_faction,
    })),
    bro_alias: first?.name,
    bro_actual: first?.actual_faction,
    opponent_name: second?.name,
    opponent_actual: second?.actual_faction,
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
