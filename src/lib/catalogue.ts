import { listReplays, type ReplayCursor } from './backend';
import { rowToReplay } from './replays';
import type { Replay } from './types';

// Read every page before publishing the new snapshot, so filters and sorting
// always cover the full library. Keyset pagination is stable across new imports.
export async function loadCatalogue(): Promise<Replay[]> {
  const replays: Replay[] = [];
  let before: ReplayCursor | undefined;
  const pageSize = 1000;
  for (;;) {
    const rows = await listReplays(pageSize, before);
    if (rows.length === 0) break;
    replays.push(...rows.map(rowToReplay));
    if (rows.length < pageSize) break;
    const last = rows[rows.length - 1];
    if (before && (last.imported_at > before.imported_at
      || (last.imported_at === before.imported_at && last.id >= before.id))) {
      throw new Error('Catalogue pagination did not advance');
    }
    before = { imported_at: last.imported_at, id: last.id };
  }
  return replays;
}
