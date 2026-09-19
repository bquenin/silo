import { expect, it, vi } from 'vitest';
import { listReplays } from './backend';
import { loadCatalogue } from './catalogue';
import { applyFilters } from './filter-sort';
import { row } from '../test/fixtures';

vi.mock('./backend', () => ({ listReplays: vi.fn() }));

it('loads and searches beyond 2000 rows with tied import timestamps', async () => {
  const rows = Array.from({ length: 2505 }, (_, i) => row([0, 0], 2505 - i));
  vi.mocked(listReplays).mockImplementation(async (limit = 1000, before) =>
    rows.filter((r) => !before || r.id < before.id).slice(0, limit));
  const replays = await loadCatalogue();
  expect(replays).toHaveLength(2505);
  expect(new Set(replays.map((r) => r.id)).size).toBe(2505);
  expect(listReplays).toHaveBeenCalledTimes(3);
  expect(applyFilters(replays, { search: 'replay-1.KWReplay', mode: null, factions: new Set() }).map((r) => r.id)).toEqual(['1']);
});

it('rejects a partial catalogue when a later page fails', async () => {
  vi.mocked(listReplays).mockResolvedValueOnce(Array.from({ length: 1000 }, (_, i) => row([0, 0], 1000 - i)))
    .mockRejectedValueOnce(new Error('database unavailable'));
  await expect(loadCatalogue()).rejects.toThrow('database unavailable');
});
