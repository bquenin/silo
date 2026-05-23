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
  const duration_s = 240 + (h % 1200);   // 4-24 minutes
  const days_ago = h % 1800;              // 0-5 years ago
  const recorded_at = new Date(Date.now() - days_ago * 86400000).toISOString();
  return { ...r, duration_s, recorded_at };
});

// Duplicate the sample into ~600 entries so we can validate infinite scroll.
export const REPLAYS: Replay[] = [];
for (let i = 0; i < 10; i++) {
  for (const r of seedReplays) {
    REPLAYS.push({ ...r, id: `${r.id}-${i}` });
  }
}

export const TOTAL_REPLAYS = REPLAYS.length;

export function formatDuration(s?: number) {
  if (!s) return '--:--';
  const m = Math.floor(s / 60);
  const sec = s % 60;
  return `${m}:${String(sec).padStart(2, '0')}`;
}

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

export function modeOf(n_players: number): string {
  switch (n_players) {
    case 2: return '1v1';
    case 4: return '2v2';
    case 6: return '3v3';
    case 8: return '4v4';
    case 3: return 'FFA';
    default: return `${n_players}p`;
  }
}
