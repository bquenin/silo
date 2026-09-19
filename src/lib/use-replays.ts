import { useCallback, useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';

import { type IngestReport, ingestPath, isTauri } from './backend';
import { loadCatalogue } from './catalogue';
import { REPLAYS as MOCK_REPLAYS } from './mock-data';
import type { Replay } from './types';

export interface ReplaysHookValue {
  replays: Replay[];
  total: number;
  loading: boolean;
  error: string | null;
  importFolder: () => Promise<IngestReport | null>;
  refresh: () => Promise<void>;
  live: boolean;
}

export function useReplays(): ReplaysHookValue {
  const live = isTauri();
  const [replays, setReplays] = useState<Replay[]>(live ? [] : MOCK_REPLAYS);
  const [loading, setLoading] = useState(live);
  const [error, setError] = useState<string | null>(null);
  const request = useRef(0);

  const refresh = useCallback(async () => {
    if (!live) return;
    const current = ++request.current;
    setLoading(true);
    setError(null);
    try {
      const rows = await loadCatalogue();
      if (current === request.current) setReplays(rows);
    } catch (reason) {
      if (current === request.current) setError(`Could not load the catalogue: ${String(reason)}`);
    } finally {
      if (current === request.current) setLoading(false);
    }
  }, [live]);

  useEffect(() => {
    void refresh();
    return () => { request.current += 1; };
  }, [refresh]);

  const importFolder = useCallback(async (): Promise<IngestReport | null> => {
    if (!live) return null;
    setLoading(true);
    setError(null);
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || Array.isArray(selected)) return null;
      const report = await ingestPath(selected);
      await refresh();
      return report;
    } catch (reason) {
      setError(`Could not import replays: ${String(reason)}`);
      return null;
    } finally {
      setLoading(false);
    }
  }, [live, refresh]);

  return { replays, total: replays.length, loading, error, importFolder, refresh, live };
}
