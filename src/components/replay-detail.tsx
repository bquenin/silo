import { Play, X } from 'lucide-react';
import { FactionDot, factionLabel, pickedRandom } from './player-names';
import { displayMapName, formatDuration, modeOf } from '../lib/replays';
import type { Replay } from '../lib/types';

function formatFullDate(iso?: string): string {
  if (!iso) return 'Unknown';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return 'Unknown';
  return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
}

export function ReplayDetail({ replay, live, onClose, onPlay }: {
  replay: Replay; live: boolean; onClose: () => void; onPlay: () => void;
}) {
  const map = displayMapName(replay.map);
  const teams = replay.teams ?? [];
  const ffa = modeOf(replay) === 'FFA';

  return (
    <aside aria-label="Replay details" className="w-80 shrink-0 border-l border-bg-border bg-bg-subtle flex flex-col min-h-0">
      <div className="flex items-start justify-between gap-3 px-4 pt-4 pb-3">
        <div className="min-w-0">
          <div className="text-[11px] uppercase tracking-wider text-fg-muted">Replay</div>
          <h2 className="text-lg font-semibold leading-tight mt-0.5 break-words">{map}</h2>
          {map !== replay.map && <div className="text-xs text-fg-dim mt-0.5 break-words">{replay.map}</div>}
        </div>
        <button onClick={onClose} aria-label="Close details" className="p-1 -mr-1 text-fg-muted hover:text-fg rounded">
          <X size={18} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-4 pb-4">
        <dl className="grid grid-cols-2 gap-x-3 gap-y-2 text-sm">
          <dt className="text-fg-muted">Mode</dt><dd>{modeOf(replay)}</dd>
          <dt className="text-fg-muted">Players</dt><dd>{replay.n_players}</dd>
          <dt className="text-fg-muted">Length</dt><dd className="tabular-nums">{formatDuration(replay.duration_s)}</dd>
          <dt className="text-fg-muted">Recorded</dt><dd>{formatFullDate(replay.recorded_at)}</dd>
        </dl>

        <div className="mt-5 space-y-4">
          {teams.map((team, t) => (
            <div key={t}>
              {!ffa && <div className="text-[11px] uppercase tracking-wider text-fg-muted mb-1.5">Team {t + 1}</div>}
              <ul className="space-y-1.5">
                {team.map((p) => (
                  <li key={p.slot} className="flex items-center gap-2 text-sm min-w-0">
                    <FactionDot faction={p.actual} chosen={p.chosen} decorative />
                    <span className="truncate">{p.name}</span>
                    <span className="ml-auto text-fg-muted whitespace-nowrap">
                      {factionLabel(p.actual)}
                      {pickedRandom(p) && <span className="text-fg-dim"> · random</span>}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-5">
          <div className="text-[11px] uppercase tracking-wider text-fg-muted mb-1">File</div>
          <div className="text-xs text-fg-muted break-all">{replay.file}</div>
        </div>
      </div>

      <div className="px-4 py-3 border-t border-bg-border">
        <button onClick={onPlay} disabled={!live}
          title={live ? 'Check map compatibility and launch the game' : 'Playback is available in the desktop app'}
          className="w-full flex items-center justify-center gap-2 rounded bg-accent hover:bg-accent-dim text-bg font-semibold text-sm px-4 py-2 disabled:opacity-40 disabled:hover:bg-accent">
          <Play size={15} fill="currentColor" /> Play
        </button>
      </div>
    </aside>
  );
}
