import raw from './mock-data-raw.json';
import type { Replay } from './types';

// Synthesize realistic-looking durations + dates since the binary parser hasn't
// extracted them yet. Just enough variety for the prototype UI.
function deterministicHash(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

const seedReplays: Replay[] = (raw as unknown as Replay[]).map((r) => {
  const h = deterministicHash(r.id);
  const duration_s = 240 + (h % 1200);
  const days_ago = h % 1800;
  const recorded_at = new Date(Date.now() - days_ago * 86400000).toISOString();
  // Synthesize a simple teams array from players[] for mock display.
  const teams = r.players?.length === 2
    ? [[r.players[0]], [r.players[1]]]
    : r.players?.map((p) => [p]) ?? [];
  return { ...r, duration_s, recorded_at, teams };
});

// Duplicate the sample into ~600 entries so we can validate infinite scroll.
export const REPLAYS: Replay[] = [];
for (let i = 0; i < 10; i++) {
  for (const r of seedReplays) {
    REPLAYS.push({ ...r, id: `${r.id}-${i}` });
  }
}

export const TOTAL_REPLAYS = REPLAYS.length;

export function formatRelative(iso?: string) {
  if (!iso) return '';
  const then = new Date(iso).getTime();
  const days = Math.floor((Date.now() - then) / 86400000);
  if (days < 1) return 'today';
  if (days < 7) return `${days}d ago`;
  if (days < 30) return `${Math.floor(days/7)}w ago`;
  if (days < 365) return `${Math.floor(days/30)}mo ago`;
  return `${Math.floor(days/365)}y ago`;
}

export function formatDate(iso?: string) {
  if (!iso) return '—';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return '—';
  return d.toISOString().slice(0, 10); // YYYY-MM-DD
}
