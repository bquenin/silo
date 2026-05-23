import { useState, useMemo } from 'react';
import { Library, Tag, Search, Play, Filter, MoreHorizontal, ArrowUpDown, Plus, FolderInput, Loader2 } from 'lucide-react';
import { FactionChip } from '../components/faction-chip';
import { formatDuration, formatRelative, modeOf } from '../lib/mock-data';
import { useReplays } from '../lib/use-replays';
import type { Replay } from '../lib/types';

export function SpotifyLayout() {
  const { replays, total, loading, importFolder, live } = useReplays();
  const [query, setQuery] = useState('');
  const [importStatus, setImportStatus] = useState<string | null>(null);
  const filtered = useMemo(
    () => replays.filter(
      (r) =>
        r.bro_alias?.toLowerCase().includes(query.toLowerCase()) ||
        r.opponent_name?.toLowerCase().includes(query.toLowerCase()) ||
        r.map.toLowerCase().includes(query.toLowerCase())
    ),
    [query, replays]
  );

  async function handleImport() {
    setImportStatus(null);
    const report = await importFolder();
    if (!report) return;
    setImportStatus(
      `Imported ${report.inserted} new (${report.duplicates} dupes, ${report.errors.length} errors)`
    );
    setTimeout(() => setImportStatus(null), 5000);
  }

  return (
    <div className="flex h-full w-full text-fg">
      {/* sidebar */}
      <aside className="w-64 shrink-0 bg-bg-subtle flex flex-col">
        <div className="px-4 py-3 mb-1">
          <div className="flex items-center justify-between mb-3">
            <span className="text-lg font-semibold tracking-wide text-accent">tacitus</span>
            <button
              onClick={handleImport}
              title={live ? 'Import a folder of .kwreplay files' : 'Run via `npm run tauri dev` to import'}
              disabled={!live || loading}
              className="text-fg-dim hover:text-fg p-1 disabled:opacity-40"
            >
              {loading ? <Loader2 size={16} className="animate-spin" /> : <Plus size={16} />}
            </button>
          </div>
          <button className="w-full flex items-center gap-2 px-2 py-2 rounded bg-bg-elevated text-fg text-sm hover:bg-bg-border transition-colors">
            <Library size={15} /> <span className="flex-1 text-left">Library</span>
            <span className="text-xs text-fg-dim font-mono">{total}</span>
          </button>
          {live && total === 0 && (
            <button
              onClick={handleImport}
              className="mt-2 w-full flex items-center gap-2 px-2 py-2 rounded border border-bg-border text-sm text-fg-muted hover:text-fg hover:border-accent-dim transition-colors"
            >
              <FolderInput size={14} /> Import folder…
            </button>
          )}
          {importStatus && (
            <div className="mt-2 text-xs text-accent-dim px-1">{importStatus}</div>
          )}
        </div>

        {/* search-in-library + sort */}
        <div className="px-4 py-2 flex items-center gap-2">
          <div className="relative flex-1">
            <Search size={12} className="absolute left-2 top-1/2 -translate-y-1/2 text-fg-dim" />
            <input
              className="bg-bg-surface border border-transparent focus:border-bg-border rounded pl-7 pr-2 py-1.5 text-xs w-full focus:outline-none"
              placeholder="Filter library..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
          <button className="text-xs text-fg-dim flex items-center gap-1 hover:text-fg">
            Recents <ArrowUpDown size={11} />
          </button>
        </div>

        {/* playlists / tags */}
        <div className="flex-1 overflow-y-auto px-2">
          <PlaylistItem label="All Replays" subtitle={`Library · ${total} items`} emoji="📚" active />
        </div>
      </aside>

      {/* main */}
      <main className="flex-1 flex flex-col min-w-0 bg-gradient-to-b from-bg-elevated/40 to-bg">
        {/* top bar */}
        <div className="h-14 flex items-center px-6 gap-3">
          <div className="flex items-center gap-1.5">
            <button className="w-7 h-7 rounded-full bg-bg-elevated text-fg-muted hover:text-fg flex items-center justify-center">‹</button>
            <button className="w-7 h-7 rounded-full bg-bg-elevated text-fg-muted hover:text-fg flex items-center justify-center">›</button>
          </div>
          <div className="flex-1" />
          <button className="text-xs text-fg-muted hover:text-fg px-2 py-1 rounded">Stats</button>
          <button className="text-xs text-fg-muted hover:text-fg px-2 py-1 rounded">Maps</button>
        </div>

        {/* hero */}
        <div className="px-6 pb-4 flex items-end gap-5">
          <div
            className="w-44 h-44 rounded shadow-xl flex items-center justify-center text-5xl font-bold tracking-tighter"
            style={{ background: 'linear-gradient(135deg, #7be03e 0%, #4f9c25 60%, #2a5314 100%)' }}
          >
            <span className="text-bg/80">T</span>
          </div>
          <div className="pb-2">
            <div className="text-xs uppercase tracking-wider text-fg-muted font-medium">Library</div>
            <div className="text-6xl font-bold mt-1 tracking-tight">All Replays</div>
            <div className="text-sm text-fg-muted mt-3">
              <span className="font-semibold text-fg">{live ? 'Live' : 'Demo'}</span> · {total} replays{live ? '' : ' · mock data'}
            </div>
          </div>
        </div>

        {/* action bar */}
        <div className="px-6 py-3 flex items-center gap-4">
          <button className="w-12 h-12 rounded-full bg-accent hover:bg-accent-dim flex items-center justify-center text-bg shadow-lg">
            <Play size={20} className="ml-0.5" fill="currentColor" />
          </button>
          <button className="text-fg-muted hover:text-fg p-2">
            <Filter size={20} />
          </button>
          <button className="text-fg-muted hover:text-fg p-2">
            <MoreHorizontal size={20} />
          </button>
          <div className="flex-1" />
          <div className="text-xs text-fg-dim flex items-center gap-1">
            <ArrowUpDown size={11} /> Date added
          </div>
        </div>

        {/* table */}
        <div className="flex-1 overflow-y-auto px-6 pb-6">
          <div className="grid grid-cols-[24px_minmax(0,2fr)_minmax(0,1fr)_minmax(240px,1.5fr)_80px_70px] gap-4 px-3 py-2 text-[10px] uppercase tracking-wider text-fg-dim border-b border-bg-border">
            <span>#</span>
            <span>Title / Players</span>
            <span>Map</span>
            <span>Matchup</span>
            <span>Added</span>
            <span className="text-right">Length</span>
          </div>
          {filtered.map((r, i) => (
            <Row key={r.id} replay={r} index={i + 1} />
          ))}
          {!loading && filtered.length === 0 && (
            <div className="text-center text-fg-muted py-12">
              {live ? (
                <>
                  <p className="mb-3">No replays in the catalogue yet.</p>
                  <button
                    onClick={handleImport}
                    className="inline-flex items-center gap-2 px-4 py-2 rounded bg-accent hover:bg-accent-dim text-bg font-medium transition-colors"
                  >
                    <FolderInput size={14} /> Import a folder
                  </button>
                </>
              ) : (
                <p>No replays match the filter.</p>
              )}
            </div>
          )}
          <div className="text-center text-fg-dim text-xs mt-6 pb-4">
            {filtered.length} of {total} replays
          </div>
        </div>
      </main>
    </div>
  );
}

function PlaylistItem({
  label, subtitle, emoji, tint, active,
}: { label: string; subtitle: string; emoji?: string; tint?: string; active?: boolean }) {
  return (
    <div
      className={`flex items-center gap-3 px-2 py-2 rounded cursor-pointer transition-colors ${
        active ? 'bg-bg-elevated' : 'hover:bg-bg-surface'
      }`}
    >
      <div
        className="w-10 h-10 rounded shrink-0 flex items-center justify-center text-base"
        style={{
          background: tint
            ? `linear-gradient(135deg, ${tint}cc, ${tint}66)`
            : 'linear-gradient(135deg, #2a2a32, #1c1c22)',
        }}
      >
        {emoji ?? <Tag size={14} className="text-bg" />}
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm text-fg truncate">{label}</div>
        <div className="text-[11px] text-fg-dim truncate">{subtitle}</div>
      </div>
    </div>
  );
}

function Row({ replay, index }: { replay: Replay; index: number }) {
  const map = replay.map.replace(/^\[[^\]]+\]\s*/, '').replace(/\s+\d+\.\d+\+.*$/, '');
  const teams = replay.teams ?? [];
  const isOneV = teams.length === 2 && teams[0]?.length === 1 && teams[1]?.length === 1;
  const isTeamGame = teams.length === 2 && (teams[0]?.length > 1 || teams[1]?.length > 1);
  const isFfa = !isOneV && !isTeamGame;

  // Cover gradient: pick two representative factions
  const lhsFaction = teams[0]?.[0]?.actual ?? 'Rnd';
  const rhsFaction = teams[teams.length - 1]?.[0]?.actual ?? 'Rnd';

  return (
    <div className="group grid grid-cols-[24px_minmax(0,3fr)_minmax(0,2fr)_140px_120px_80px] gap-4 px-3 py-2 rounded items-center hover:bg-bg-surface cursor-pointer transition-colors">
      <span className="text-sm text-fg-dim font-mono group-hover:hidden">{index}</span>
      <span className="hidden group-hover:flex text-accent-dim items-center">
        <Play size={12} fill="currentColor" />
      </span>

      <div className="min-w-0 flex items-center gap-3">
        <div
          className="w-9 h-9 rounded shrink-0"
          style={{
            background: `linear-gradient(135deg, ${factionColor(lhsFaction)}99, ${factionColor(rhsFaction)}99)`,
          }}
        />
        <div className="min-w-0">
          <div className="text-sm text-fg truncate">
            {isOneV && (
              <>
                {teams[0][0].name}
                <span className="text-fg-muted"> vs {teams[1][0].name}</span>
              </>
            )}
            {isTeamGame && (
              <>
                <span>{teams[0].map((p) => p.name).join(' · ')}</span>
                <span className="text-fg-muted"> vs </span>
                <span>{teams[1].map((p) => p.name).join(' · ')}</span>
              </>
            )}
            {isFfa && (
              <span>{teams.map((t) => t[0]?.name).filter(Boolean).join(' · ')}</span>
            )}
          </div>
          <div className="text-xs text-fg-dim truncate">
            {modeOf(replay.n_players)} · {replay.n_players} players
          </div>
        </div>
      </div>

      <div className="text-sm text-fg-muted truncate">{map}</div>

      <div className="flex items-center gap-1 min-w-0 overflow-hidden">
        {isOneV && (
          <>
            <FactionChip faction={teams[0][0].actual} chosen={teams[0][0].chosen} size="sm" showLabel={false} />
            <span className="text-xs text-fg-dim">vs</span>
            <FactionChip faction={teams[1][0].actual} chosen={teams[1][0].chosen} size="sm" showLabel={false} />
          </>
        )}
        {isTeamGame && (
          <>
            <div className="flex items-center gap-0.5 flex-wrap">
              {teams[0].map((p, i) => (
                <FactionChip key={i} faction={p.actual} chosen={p.chosen} size="sm" showLabel={false} />
              ))}
            </div>
            <span className="text-xs text-fg-dim mx-1 shrink-0">vs</span>
            <div className="flex items-center gap-0.5 flex-wrap">
              {teams[1].map((p, i) => (
                <FactionChip key={i} faction={p.actual} chosen={p.chosen} size="sm" showLabel={false} />
              ))}
            </div>
          </>
        )}
        {isFfa && (
          <div className="flex items-center gap-0.5 flex-wrap">
            {teams.map((t, i) => (
              <FactionChip key={i} faction={t[0]?.actual ?? 'Rnd'} chosen={t[0]?.chosen} size="sm" showLabel={false} />
            ))}
          </div>
        )}
      </div>

      <span className="text-xs text-fg-muted">{formatRelative(replay.recorded_at)}</span>
      <span className="text-xs text-fg-muted text-right font-mono">{formatDuration(replay.duration_s)}</span>
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
