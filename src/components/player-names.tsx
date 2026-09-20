import { FACTION_COLOR, FACTION_LABEL } from '../lib/types';
import type { Player } from '../lib/types';

/** Short-hand for "did this player pick Random and get assigned a faction". */
export function pickedRandom(p: Pick<Player, 'chosen' | 'actual'>): boolean {
  return p.chosen === 'Rnd' && p.actual !== 'Rnd';
}

export function factionLabel(faction: string): string {
  return FACTION_LABEL[faction] ?? faction;
}

const FACTION_MONOGRAM: Record<string, string> = {
  GDI: 'GDI', Nod: 'NOD', Sc: 'SCRIN',
  ST: 'ST', T59: 'T59', R17: 'R17',
  MoK: 'MoK', BH: 'BH', ZCM: 'ZOCOM',
  Rnd: 'RND', Obs: 'OBS',
};

export function FactionMonogram({ faction, chosen, className = '', decorative = false }: {
  faction: string; chosen?: string; className?: string; decorative?: boolean;
}) {
  const color = FACTION_COLOR[faction] ?? FACTION_COLOR.Rnd;
  const random = pickedRandom({ chosen: chosen ?? faction, actual: faction });
  const label = `${factionLabel(faction)}${random ? ' (picked Random)' : ''}`;
  return (
    <span
      aria-hidden={decorative || undefined}
      title={decorative ? undefined : label}
      className={`inline-flex shrink-0 items-center rounded border px-1 py-0.5 align-middle font-mono text-[10px] font-semibold leading-none ${className}`}
      style={{
        color: `color-mix(in srgb, ${color} 70%, white)`,
        backgroundColor: `${color}14`,
        borderColor: `${color}40`,
      }}
    >
      <span aria-hidden="true">{FACTION_MONOGRAM[faction] ?? faction}</span>
      {!decorative && <span className="sr-only">{label}</span>}
    </span>
  );
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

/** "A, B vs C, D" with a faction monogram before each name. */
export function TeamNames({ teams, className = '' }: { teams: Player[][]; className?: string }) {
  return (
    <span className={`min-w-0 truncate ${className}`}>
      {teams.map((team, t) => (
        <span key={t}>
          {t > 0 && <span className="text-fg-dim mx-1.5">vs</span>}
          {team.map((p, i) => (
            <span key={p.slot} className="whitespace-nowrap">
              {i > 0 && <span className="text-fg-dim">, </span>}
              <FactionMonogram faction={p.actual} chosen={p.chosen} className="mr-1.5 -mt-0.5" />
              {p.name}
            </span>
          ))}
        </span>
      ))}
    </span>
  );
}
