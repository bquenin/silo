import { FACTION_COLOR, FACTION_LABEL } from '../lib/types';

interface Props {
  faction: string;
  chosen?: string;          // when actual differs from chosen, mark it
  size?: 'sm' | 'md';
  showLabel?: boolean;
}

export function FactionChip({ faction, chosen, size = 'sm', showLabel = true }: Props) {
  const color = FACTION_COLOR[faction] ?? '#56565c';
  const label = FACTION_LABEL[faction] ?? faction;
  const wasRandom = chosen === 'Rnd' && faction !== 'Rnd';
  const px = size === 'sm' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-xs';
  return (
    <span
      className={`inline-flex items-center gap-1 rounded font-medium uppercase tracking-wider ${px}`}
      style={{
        backgroundColor: `${color}1a`,
        color,
        border: `1px solid ${color}33`,
      }}
      title={wasRandom ? `${label} (assigned from Random)` : label}
    >
      {showLabel ? label : faction}
      {wasRandom && <span className="opacity-60">?</span>}
    </span>
  );
}
