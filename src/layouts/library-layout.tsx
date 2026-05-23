import { useState, useMemo } from 'react';
import { Library, Star, Tag, Map, Search, Play, ChevronRight } from 'lucide-react';
import { FactionChip } from '../components/faction-chip';
import { REPLAYS, formatDuration, formatRelative, modeOf } from '../lib/mock-data';
import type { Replay } from '../lib/types';

export function LibraryLayout() {
  const [selected, setSelected] = useState<Replay | null>(REPLAYS[0]);
  const [query, setQuery] = useState('');
  const filtered = useMemo(
    () => REPLAYS.filter(
      (r) =>
        r.bro_alias?.toLowerCase().includes(query.toLowerCase()) ||
        r.opponent_name?.toLowerCase().includes(query.toLowerCase()) ||
        r.map.toLowerCase().includes(query.toLowerCase())
    ),
    [query]
  );

  return (
    <div className="flex h-full w-full text-fg">
      {/* sidebar */}
      <aside className="w-56 shrink-0 bg-bg-subtle border-r border-bg-border flex flex-col">
        <div className="h-12 flex items-center px-4 border-b border-bg-border">
          <span className="font-semibold tracking-wide text-accent">tacitus</span>
          <span className="ml-1 text-fg-dim text-xs">v0.1</span>
        </div>
        <nav className="p-3 space-y-1 text-sm">
          <SidebarItem icon={<Library size={15} />} label="All Replays" count={REPLAYS.length} active />
          <SidebarItem icon={<Star size={15} />} label="Starred" count={12} />
          <SidebarItem icon={<Tag size={15} />} label="BRO" count={593} />
          <SidebarItem icon={<Tag size={15} />} label="1v1" count={458} />
          <SidebarItem icon={<Tag size={15} />} label="2v2" count={110} />
          <SidebarItem icon={<Tag size={15} />} label="Tournament" count={89} />
          <div className="pt-3 mt-3 border-t border-bg-border">
            <SidebarSection label="MAPS" />
            <SidebarItem icon={<Map size={15} />} label="Tournament Rift" count={83} />
            <SidebarItem icon={<Map size={15} />} label="Tournament Dustbowl" count={47} />
            <SidebarItem icon={<Map size={15} />} label="Tournament Decision" count={41} />
            <SidebarItem icon={<Map size={15} />} label="Smashed Town" count={17} />
            <SidebarItem icon={<Map size={15} />} label="More..." count={null} muted />
          </div>
        </nav>
        <div className="mt-auto p-3 text-xs text-fg-dim border-t border-bg-border">
          <div>tsug303</div>
          <div className="text-fg-dim mt-1">{REPLAYS.length} replays · 412 MB</div>
        </div>
      </aside>

      {/* main */}
      <main className="flex-1 flex min-w-0">
        {/* grid pane */}
        <section className="flex-1 flex flex-col min-w-0">
          <div className="h-12 flex items-center px-5 border-b border-bg-border gap-3">
            <h1 className="text-lg font-semibold flex-1">All Replays</h1>
            <div className="relative">
              <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-dim" />
              <input
                className="bg-bg-surface border border-bg-border rounded pl-8 pr-3 py-1.5 text-sm w-64 focus:outline-none focus:border-accent-dim"
                placeholder="Search replays..."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
            </div>
          </div>
          {/* infinite-scroll grid */}
          <div className="flex-1 overflow-y-auto p-5">
            <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
              {filtered.map((r) => (
                <ReplayCard
                  key={r.id}
                  replay={r}
                  selected={selected?.id === r.id}
                  onClick={() => setSelected(r)}
                />
              ))}
            </div>
            <div className="text-center text-fg-dim text-xs mt-8 pb-4">
              {filtered.length} of {REPLAYS.length} replays
            </div>
          </div>
        </section>

        {/* detail pane */}
        {selected && (
          <aside className="w-80 shrink-0 bg-bg-subtle border-l border-bg-border overflow-y-auto">
            <DetailPane replay={selected} />
          </aside>
        )}
      </main>
    </div>
  );
}

function SidebarItem({
  icon, label, count, active, muted,
}: { icon?: React.ReactNode; label: string; count?: number | null; active?: boolean; muted?: boolean }) {
  return (
    <div
      className={`flex items-center gap-2 px-2 py-1.5 rounded text-sm cursor-pointer transition-colors ${
        active
          ? 'bg-bg-elevated text-fg'
          : muted
          ? 'text-fg-dim hover:text-fg hover:bg-bg-surface'
          : 'text-fg-muted hover:text-fg hover:bg-bg-surface'
      }`}
    >
      {icon}
      <span className="flex-1 truncate">{label}</span>
      {count !== null && count !== undefined && (
        <span className="text-xs text-fg-dim font-mono">{count}</span>
      )}
    </div>
  );
}

function SidebarSection({ label }: { label: string }) {
  return (
    <div className="px-2 py-1.5 text-[10px] font-semibold tracking-wider text-fg-dim uppercase">
      {label}
    </div>
  );
}

function ReplayCard({ replay, selected, onClick }: { replay: Replay; selected: boolean; onClick: () => void }) {
  const map = replay.map.replace(/^\[[^\]]+\]\s*/, '').replace(/\s+\d+\.\d+\+.*$/, '');
  return (
    <div
      onClick={onClick}
      className={`group bg-bg-surface border rounded-lg overflow-hidden cursor-pointer transition-all hover:border-accent-dim/50 hover:shadow-lg hover:-translate-y-0.5 ${
        selected ? 'border-accent-dim ring-1 ring-accent-dim/40' : 'border-bg-border'
      }`}
    >
      {/* "cover" area: map name placeholder w/ gradient */}
      <div
        className="aspect-[16/10] flex items-end p-3 relative overflow-hidden"
        style={{
          background: `linear-gradient(135deg, ${factionColor(replay.bro_actual)}33, ${factionColor(replay.opponent_actual)}33)`,
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent" />
        <div className="relative z-10">
          <div className="text-[10px] uppercase tracking-wider text-fg-dim font-semibold">
            {modeOf(replay.n_players)} · {formatDuration(replay.duration_s)}
          </div>
          <div className="text-sm font-semibold text-fg truncate">{map}</div>
        </div>
        <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button className="w-7 h-7 rounded-full bg-accent/90 text-bg flex items-center justify-center shadow">
            <Play size={13} className="ml-0.5" fill="currentColor" />
          </button>
        </div>
      </div>
      <div className="p-3 space-y-2">
        <div className="flex items-center gap-1.5 min-w-0">
          <FactionChip faction={replay.bro_actual ?? 'Rnd'} chosen={replay.players[0]?.chosen} size="sm" showLabel={false} />
          <span className="text-fg-dim text-xs truncate">{replay.bro_alias}</span>
        </div>
        {replay.opponent_actual && (
          <div className="flex items-center gap-1.5 min-w-0">
            <FactionChip faction={replay.opponent_actual} chosen={replay.players[1]?.chosen} size="sm" showLabel={false} />
            <span className="text-fg-dim text-xs truncate">{replay.opponent_name}</span>
          </div>
        )}
        <div className="text-[10px] text-fg-dim pt-1">{formatRelative(replay.recorded_at)}</div>
      </div>
    </div>
  );
}

function factionColor(f?: string): string {
  switch (f) {
    case 'GDI': case 'ZCM': case 'ST': return '#e6c34a';
    case 'Nod': case 'BH': case 'MoK': return '#d44848';
    case 'Sc': case 'R17': case 'T59': return '#4ad4cf';
    default: return '#7be03e';
  }
}

function DetailPane({ replay }: { replay: Replay }) {
  return (
    <div>
      <div
        className="aspect-[16/10] flex items-end p-4 relative"
        style={{
          background: `linear-gradient(135deg, ${factionColor(replay.bro_actual)}44, ${factionColor(replay.opponent_actual)}44)`,
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-t from-black/70 to-transparent" />
        <div className="relative z-10 space-y-1">
          <div className="text-[10px] uppercase tracking-wider text-fg-dim font-semibold">
            {modeOf(replay.n_players)} · {formatDuration(replay.duration_s)} · {formatRelative(replay.recorded_at)}
          </div>
          <div className="text-base font-semibold">{replay.map}</div>
        </div>
      </div>
      <div className="p-4 space-y-4">
        <button className="w-full flex items-center justify-center gap-2 bg-accent hover:bg-accent-dim text-bg font-medium py-2 px-3 rounded transition-colors">
          <Play size={14} fill="currentColor" /> Watch via DLL
        </button>
        <div>
          <div className="text-xs font-semibold text-fg-dim uppercase tracking-wider mb-2">Players</div>
          <div className="space-y-2">
            {replay.players.map((p) => (
              <div key={p.slot} className="flex items-center gap-2 text-sm">
                <FactionChip faction={p.actual} chosen={p.chosen} size="md" />
                <span className="flex-1 truncate text-fg">{p.name}</span>
                <span className="text-xs text-fg-dim">slot {p.slot}</span>
              </div>
            ))}
          </div>
        </div>
        <div>
          <div className="text-xs font-semibold text-fg-dim uppercase tracking-wider mb-2">File</div>
          <div className="text-xs text-fg-muted break-all font-mono leading-relaxed">{replay.file}</div>
          <div className="text-xs text-fg-dim mt-1">id {replay.id}</div>
        </div>
        <div>
          <div className="text-xs font-semibold text-fg-dim uppercase tracking-wider mb-2">Map pack</div>
          <div className="flex items-center justify-between p-2 bg-bg-surface rounded border border-bg-border text-xs">
            <span className="text-fg-muted">R24j map pack</span>
            <span className="text-accent-dim">installed</span>
          </div>
        </div>
        <div className="pt-2">
          <button className="text-xs text-fg-dim hover:text-fg flex items-center gap-1">
            More details <ChevronRight size={12} />
          </button>
        </div>
      </div>
    </div>
  );
}
