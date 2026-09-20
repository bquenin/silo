import { FACTION_COLOR, FACTION_LABEL } from '../lib/types';
import type { Player } from '../lib/types';

/** Short-hand for "did this player pick Random and get assigned a faction". */
export function pickedRandom(p: Pick<Player, 'chosen' | 'actual'>): boolean {
  return p.chosen === 'Rnd' && p.actual !== 'Rnd';
}

export function factionLabel(faction: string): string {
  return FACTION_LABEL[faction] ?? faction;
}

/** Small coloured dot naming the faction on hover. Ringed when the faction
 *  was assigned from Random so the distinction survives without text.
 *  Pass `decorative` when the faction name is already written next to it. */
export function FactionDot({ faction, chosen, className = '', decorative = false }: {
  faction: string; chosen?: string; className?: string; decorative?: boolean;
}) {
  const color = FACTION_COLOR[faction] ?? FACTION_COLOR.Obs;
  const random = pickedRandom({ chosen: chosen ?? faction, actual: faction });
  return (
    <span
      role={decorative ? undefined : 'img'}
      aria-hidden={decorative || undefined}
      aria-label={decorative ? undefined : factionLabel(faction)}
      title={decorative ? undefined : factionLabel(faction)}
      className={`inline-block w-2 h-2 rounded-full shrink-0 ${className}`}
      style={{ backgroundColor: color, boxShadow: random ? `0 0 0 1.5px ${color}66` : undefined }}
    />
  );
}

/** "A, B vs C, D" with a faction dot before each name. */
export function TeamNames({ teams, className = '' }: { teams: Player[][]; className?: string }) {
  return (
    <span className={`min-w-0 truncate ${className}`}>
      {teams.map((team, t) => (
        <span key={t}>
          {t > 0 && <span className="text-fg-dim mx-1.5">vs</span>}
          {team.map((p, i) => (
            <span key={p.slot} className="whitespace-nowrap">
              {i > 0 && <span className="text-fg-dim">, </span>}
              <FactionDot faction={p.actual} chosen={p.chosen} className="mr-1.5 align-middle -mt-0.5" />
              {p.name}
            </span>
          ))}
        </span>
      ))}
    </span>
  );
}
