import { useCallback, useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

import { type IngestReport, ingestPath, isTauri, takePendingReplay } from './backend';
import { loadCatalogue } from './catalogue';
import { REPLAYS as MOCK_REPLAYS } from './mock-data';
import type { Replay } from './types';

export interface ReplaysHookValue {
  replays: Replay[];
  total: number;
  loading: boolean;
  error: string | null;
  openedReport: IngestReport | null;
  importFolder: () => Promise<IngestReport | null>;
  refresh: () => Promise<void>;
  live: boolean;
}

export function useReplays(): ReplaysHookValue {
  const live = isTauri();
  const [replays, setReplays] = useState<Replay[]>(live ? [] : MOCK_REPLAYS);
  const [loading, setLoading] = useState(live);
  const [error, setError] = useState<string | null>(null);
  const [openedReport, setOpenedReport] = useState<IngestReport | null>(null);
  const request = useRef(0);
  const pendingTask = useRef<Promise<void>>(Promise.resolve());

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

  const processPendingReplays = useCallback(() => {
    if (!live) return;
    pendingTask.current = pendingTask.current.then(async () => {
      let combined: IngestReport | null = null;
      let importing = false;
      try {
        for (;;) {
          const path = await takePendingReplay();
          if (!path) break;
          importing = true;
          setLoading(true);
          const report = await ingestPath(path);
          combined ??= { scanned: 0, inserted: 0, duplicates: 0, errors: [] };
          combined.scanned += report.scanned;
          combined.inserted += report.inserted;
          combined.duplicates += report.duplicates;
          combined.errors.push(...report.errors);
        }
        if (combined) {
          await refresh();
          setOpenedReport(combined);
        }
      } catch (reason) {
        setError(`Could not open replay: ${String(reason)}`);
      } finally {
        if (importing) setLoading(false);
      }
    });
  }, [live, refresh]);

  useEffect(() => {
    if (!live) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen('replay-open-requested', processPendingReplays).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
        processPendingReplays();
      }
    }).catch((reason) => setError(`Could not listen for replay files: ${String(reason)}`));
    return () => { disposed = true; unlisten?.(); };
  }, [live, processPendingReplays]);

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

  return { replays, total: replays.length, loading, error, openedReport, importFolder, refresh, live };
}
