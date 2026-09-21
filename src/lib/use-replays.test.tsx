import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { open } from '@tauri-apps/plugin-dialog';
import { ingestPath, takePendingReplay } from './backend';
import { loadCatalogue } from './catalogue';
import { useReplays } from './use-replays';

vi.mock('./backend', () => ({ isTauri: () => true, ingestPath: vi.fn(), takePendingReplay: vi.fn() }));
vi.mock('./catalogue', () => ({ loadCatalogue: vi.fn() }));
vi.mock('./mock-data', () => ({ REPLAYS: [] }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(vi.fn()) }));
afterEach(cleanup);
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(loadCatalogue).mockResolvedValue([]);
  vi.mocked(takePendingReplay).mockResolvedValue(null);
});

it('reports a load failure and clears it after a successful retry', async () => {
  vi.mocked(loadCatalogue).mockRejectedValueOnce(new Error('database unavailable'));
  const { result } = renderHook(useReplays);
  await waitFor(() => expect(result.current.loading).toBe(false));
  expect(result.current.error).toContain('database unavailable');
  await act(() => result.current.refresh());
  expect(result.current.error).toBeNull();
});

it('reports both dialog and backend import failures without rejected promises', async () => {
  const { result } = renderHook(useReplays);
  await waitFor(() => expect(result.current.loading).toBe(false));
  vi.mocked(open).mockRejectedValueOnce(new Error('dialog failed'));
  await act(() => result.current.importFolder());
  expect(result.current.error).toContain('dialog failed');
  vi.mocked(open).mockResolvedValueOnce('C:/replays');
  vi.mocked(ingestPath).mockRejectedValueOnce(new Error('directory missing'));
  await act(() => result.current.importFolder());
  expect(result.current.error).toContain('directory missing');
  expect(result.current.loading).toBe(false);
});

it('imports a replay received from Explorer and reports the result', async () => {
  vi.mocked(takePendingReplay)
    .mockResolvedValueOnce('C:/replays/match.kwreplay')
    .mockResolvedValue(null);
  vi.mocked(ingestPath).mockResolvedValue({ scanned: 1, inserted: 1, duplicates: 0, errors: [] });

  const { result } = renderHook(useReplays);
  await waitFor(() => expect(result.current.openedReport?.inserted).toBe(1));

  expect(ingestPath).toHaveBeenCalledWith('C:/replays/match.kwreplay');
  expect(loadCatalogue).toHaveBeenCalledTimes(2);
});
