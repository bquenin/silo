import { useEffect, useMemo, useRef, useState } from 'react';
import { observeElementRect, useVirtualizer } from '@tanstack/react-virtual';
import {
  ArrowDown, ArrowUp, Check, ChevronDown, FolderInput, Loader2, Play, Search, X,
  FileCheck2,
} from 'lucide-react';
import { FactionMonogram, TeamNames } from '../components/player-names';
import { PlaybackDialog } from '../components/playback-dialog';
import { ReplayDetail } from '../components/replay-detail';
import {
  associateReplayFiles, replayFileAssociation,
  type IngestReport, type ReplayFileAssociation,
} from '../lib/backend';
import { formatDate, formatRelative } from '../lib/mock-data';
import { displayMapName, formatDuration, modeOf } from '../lib/replays';
import { useReplays } from '../lib/use-replays';
import {
  applyFilters, applySort, FACTION_GROUPS, MODE_OPTIONS,
  type FilterState, type SortKey, type SortState,
} from '../lib/filter-sort';
import { FACTION_LABEL } from '../lib/types';
import type { Replay } from '../lib/types';

// One source of truth for the table grid — header and rows must match exactly.
const GRID = 'grid-cols-[32px_minmax(0,2.4fr)_minmax(0,1.2fr)_64px_104px_64px]';
const ROW_H = 44;

// jsdom (and any render before layout) measures the scroll container as 0×0,
// and a zero-height viewport makes the virtualizer render no rows at all.
// Fall back to a nominal viewport in that case.
const FALLBACK_RECT = { width: 1000, height: 800 };

const SORT_COLUMNS: { key: SortKey; label: string; right?: boolean }[] = [
  { key: 'map', label: 'Map' },
  { key: 'players', label: 'Mode' },
  { key: 'recorded', label: 'Date' },
  { key: 'length', label: 'Length', right: true },
];

export function LibraryLayout() {
  const { replays, total, loading, error, openedReport, importFolder, refresh, live } = useReplays();
  const [filter, setFilter] = useState<FilterState>({ search: '', mode: null, factions: new Set() });
  const [sort, setSort] = useState<SortState>({ key: 'recorded', dir: 'desc' });
  const [report, setReport] = useState<IngestReport | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [playbackReplay, setPlaybackReplay] = useState<Replay | null>(null);
  const [association, setAssociation] = useState<ReplayFileAssociation | null>(null);
  const [associationBusy, setAssociationBusy] = useState(false);
  const [associationError, setAssociationError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const filtered = useMemo(
    () => applySort(applyFilters(replays, filter), sort),
    [replays, filter, sort],
  );
  const selected = filtered.find((r) => r.id === selectedId) ?? null;
  const hasFilters = filter.mode != null || filter.factions.size > 0 || filter.search !== '';

  // Escape closes the detail panel — unless the playback dialog owns the key.
  useEffect(() => {
    if (playbackReplay) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setSelectedId(null);
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [playbackReplay]);

  useEffect(() => {
    if (!live) return;
    void replayFileAssociation()
      .then(setAssociation)
      .catch((reason) => setAssociationError(`Could not check the file association: ${String(reason)}`));
  }, [live]);

  useEffect(() => {
    if (openedReport) setReport(openedReport);
  }, [openedReport]);

  function clearFilters() {
    setFilter({ search: '', mode: null, factions: new Set() });
  }

  function toggleSort(key: SortKey) {
    setSort((prev) =>
      prev.key === key
        ? { key, dir: prev.dir === 'asc' ? 'desc' : 'asc' }
        : { key, dir: key === 'map' ? 'asc' : 'desc' });
  }

  async function handleImport() {
    setReport(null);
    const result = await importFolder();
    if (result) setReport(result);
  }

  async function handleAssociation() {
    setAssociationBusy(true);
    setAssociationError(null);
    try {
      const result = await associateReplayFiles();
      setAssociation(result);
      if (!result.associated) {
        setAssociationError('Tacitus is registered, but Windows has another default app. Right-click a .kwreplay file, choose Open with, select Tacitus and choose Always.');
      }
    } catch (reason) {
      setAssociationError(`Could not associate .kwreplay files: ${String(reason)}`);
    } finally {
      setAssociationBusy(false);
    }
  }

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_H,
    overscan: 12,
    initialRect: FALLBACK_RECT,
    observeElementRect: (instance, cb) =>
      observeElementRect(instance, (rect) =>
        cb(rect.width === 0 && rect.height === 0 ? FALLBACK_RECT : rect)),
  });

  const showEmptyLibrary = live && replays.length === 0 && !error && !loading;
  const showNoMatch = !showEmptyLibrary && filtered.length === 0 && !loading && !error;

  return (
    <div className="h-full w-full flex flex-col text-fg">
      {/* toolbar */}
      <div className="h-12 shrink-0 flex items-center px-4 gap-3 border-b border-bg-border bg-bg-subtle">
        <span className="text-accent font-semibold tracking-wide">tacitus</span>

        <div className="relative w-64">
          <Search size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-muted" />
          <input
            className="w-full bg-bg-surface border border-bg-border rounded pl-7 pr-2 py-1.5 text-sm focus:outline-none focus:border-accent-dim"
            placeholder="Search players, maps, files"
            value={filter.search}
            onChange={(e) => setFilter((p) => ({ ...p, search: e.target.value }))}
          />
        </div>

        <div className="flex items-center bg-bg-elevated rounded-md p-0.5">
          {MODE_OPTIONS.map((o) => (
            <button
              key={o.label}
              onClick={() => setFilter((p) => ({ ...p, mode: o.value }))}
              className={`px-2.5 py-1 rounded text-xs transition-colors ${
                filter.mode === o.value ? 'bg-bg-border text-fg' : 'text-fg-muted hover:text-fg'
              }`}
            >
              {o.label}
            </button>
          ))}
        </div>

        <FactionFilter
          selected={filter.factions}
          onToggle={(f) => setFilter((p) => {
            const next = new Set(p.factions);
            if (next.has(f)) next.delete(f); else next.add(f);
            return { ...p, factions: next };
          })}
        />

        {hasFilters && (
          <button
            onClick={clearFilters}
            className="flex items-center gap-1 text-xs text-fg-muted hover:text-fg px-2 py-1 rounded hover:bg-bg-surface"
          >
            <X size={12} /> Clear
          </button>
        )}

        <div className="flex-1" />

        {!live && (
          <span className="text-[11px] px-2 py-0.5 rounded-full text-fg-muted border border-bg-border">Demo data</span>
        )}

        {live && association?.supported !== false && (
          <button
            onClick={() => void handleAssociation()}
            disabled={associationBusy || association?.associated}
            title={association?.associated
              ? '.kwreplay files open with this copy of Tacitus'
              : 'Open .kwreplay files with this copy of Tacitus'}
            className="flex items-center gap-1.5 border border-bg-border text-fg-muted rounded px-3 py-1.5 text-sm hover:text-fg hover:bg-bg-surface disabled:opacity-60"
          >
            {associationBusy ? <Loader2 size={14} className="animate-spin" /> : <FileCheck2 size={14} />}
            {association?.associated ? '.kwreplay associated' : 'Associate .kwreplay'}
          </button>
        )}

        <button
          onClick={handleImport}
          disabled={!live || loading}
          title={live ? 'Import a folder of .kwreplay files' : 'Run via `npm run tauri dev` to import'}
          className="flex items-center gap-1.5 bg-accent text-bg font-medium rounded px-3 py-1.5 text-sm hover:bg-accent-dim disabled:opacity-40"
        >
          {loading ? <Loader2 size={14} className="animate-spin" /> : <FolderInput size={14} />}
          Import folder
        </button>
      </div>

      {/* banners */}
      {error && (
        <div role="alert" className="shrink-0 mx-4 mt-3 rounded border border-red-400/30 bg-red-400/5 p-3 text-sm text-red-300">
          <p>{error}</p>
          <button onClick={() => void refresh()} disabled={loading} className="mt-2 underline disabled:opacity-40">
            Retry loading catalogue
          </button>
        </div>
      )}
      {associationError && (
        <div role="alert" className="shrink-0 mx-4 mt-3 rounded border border-red-400/30 bg-red-400/5 p-3 text-sm text-red-300 flex items-start gap-3">
          <p className="flex-1">{associationError}</p>
          <button onClick={() => setAssociationError(null)} aria-label="Dismiss" className="text-fg-muted hover:text-fg p-0.5">
            <X size={14} />
          </button>
        </div>
      )}
      {report && (
        <div role="status" className="shrink-0 mx-4 mt-3 rounded border border-bg-border bg-bg-surface p-3 text-sm flex items-start gap-3">
          <div className="flex-1 min-w-0">
            <p>
              Imported {report.inserted} new · {report.duplicates} duplicate{report.duplicates === 1 ? '' : 's'}
              {' '}· {report.errors.length} error{report.errors.length === 1 ? '' : 's'}
            </p>
            {report.errors.length > 0 && (
              <details className="mt-2 text-xs text-fg-muted">
                <summary className="cursor-pointer">Show errors</summary>
                <ul className="mt-1 space-y-0.5">
                  {report.errors.map((e, i) => <li key={i} className="break-all">{e.path}: {e.message}</li>)}
                </ul>
              </details>
            )}
          </div>
          <button onClick={() => setReport(null)} aria-label="Dismiss" className="text-fg-muted hover:text-fg p-0.5">
            <X size={14} />
          </button>
        </div>
      )}

      {/* body */}
      <div className="flex-1 flex min-h-0">
        <div className="flex-1 min-w-0 flex flex-col">
          <div ref={scrollRef} role="grid" aria-label="Replays" aria-rowcount={filtered.length} className="flex-1 overflow-auto">
            <div role="row" className={`sticky top-0 z-10 grid ${GRID} gap-3 px-3 h-8 items-center bg-bg border-b border-bg-border text-[11px] uppercase tracking-wider text-fg-muted`}>
              <span role="columnheader">#</span>
              <span role="columnheader">Match</span>
              {SORT_COLUMNS.map((c) => (
                <span key={c.key} role="columnheader" className={c.right ? 'text-right' : ''}
                  aria-sort={sort.key === c.key ? (sort.dir === 'asc' ? 'ascending' : 'descending') : 'none'}>
                  <button
                    onClick={() => toggleSort(c.key)}
                    className={`inline-flex items-center gap-1 uppercase tracking-wider hover:text-fg ${
                      sort.key === c.key ? 'text-fg' : ''
                    }`}
                  >
                    {c.label}
                    {sort.key === c.key && (sort.dir === 'asc' ? <ArrowUp size={11} /> : <ArrowDown size={11} />)}
                  </button>
                </span>
              ))}
            </div>

            {loading && replays.length === 0 && (
              <div className="flex items-center justify-center gap-2 text-fg-muted py-16">
                <Loader2 size={16} className="animate-spin" /> Loading catalogue…
              </div>
            )}

            {showEmptyLibrary && (
              <div className="text-center text-fg-muted py-16">
                <p className="mb-3">No replays in the catalogue yet.</p>
                <button
                  onClick={handleImport}
                  className="inline-flex items-center gap-2 rounded bg-accent hover:bg-accent-dim text-bg font-medium px-4 py-2"
                >
                  <FolderInput size={14} /> Import a folder
                </button>
              </div>
            )}

            {showNoMatch && (
              <div className="text-center text-fg-muted py-16">
                <p className="mb-2">No replays match the filter.</p>
                <button onClick={clearFilters} className="text-accent hover:underline text-sm">Clear filters</button>
              </div>
            )}

            <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
              {virtualizer.getVirtualItems().map((v) => {
                const replay = filtered[v.index];
                if (!replay) return null;
                return (
                  <Row
                    key={replay.id}
                    replay={replay}
                    index={v.index + 1}
                    start={v.start}
                    live={live}
                    selected={replay.id === selectedId}
                    onSelect={() => setSelectedId(replay.id)}
                    onPlay={() => setPlaybackReplay(replay)}
                  />
                );
              })}
            </div>
          </div>

          <div className="shrink-0 h-8 flex items-center px-4 text-xs text-fg-dim border-t border-bg-border">
            {filtered.length === total ? `${total} replays` : `${filtered.length} of ${total} replays`}
          </div>
        </div>

        {selected && (
          <ReplayDetail
            replay={selected}
            live={live}
            onClose={() => setSelectedId(null)}
            onPlay={() => setPlaybackReplay(selected)}
          />
        )}
      </div>

      {playbackReplay && (
        <PlaybackDialog key={playbackReplay.id} replay={playbackReplay} onClose={() => setPlaybackReplay(null)} />
      )}
    </div>
  );
}

function FactionFilter({ selected, onToggle }: { selected: Set<string>; onToggle: (f: string) => void }) {
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDown(e: MouseEvent) {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false);
    }
    document.addEventListener('mousedown', onDown);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDown);
      document.removeEventListener('keydown', onKey);
    };
  }, [open]);

  const label = selected.size === 0
    ? 'Faction'
    : selected.size === 1
      ? <FactionMonogram faction={[...selected][0]} />
      : `${selected.size} factions`;

  return (
    <div ref={wrapRef} className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        aria-haspopup="listbox"
        aria-expanded={open}
        className={`flex items-center gap-1 text-xs px-2 py-1 rounded border border-bg-border hover:text-fg ${
          selected.size > 0 ? 'text-fg' : 'text-fg-muted'
        }`}
      >
        {label} <ChevronDown size={12} />
      </button>
      {open && (
        <div
          role="listbox"
          aria-multiselectable
          aria-label="Filter by faction"
          className="absolute left-0 top-full mt-1 z-20 bg-bg-surface border border-bg-border rounded-md shadow-lg min-w-[224px]"
        >
          {FACTION_GROUPS.map((group) => (
            <div key={group.label} role="group" aria-label={group.label}
              className="py-1 border-t border-bg-border first:border-t-0">
              {group.factions.map((f) => {
                const active = selected.has(f);
                return (
                  <button
                    key={f}
                    role="option"
                    aria-selected={active}
                    onClick={() => onToggle(f)}
                    className="w-full flex items-center gap-2 px-3 py-1.5 text-left text-xs text-fg-muted hover:bg-bg-elevated hover:text-fg"
                  >
                    <FactionMonogram faction={f} decorative className="w-12 justify-center" />
                    <span className="flex-1">{FACTION_LABEL[f] ?? f}</span>
                    <Check size={12} aria-hidden="true" className={`shrink-0 text-accent ${active ? '' : 'invisible'}`} />
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function Row({ replay, index, start, live, selected, onSelect, onPlay }: {
  replay: Replay; index: number; start: number; live: boolean;
  selected: boolean; onSelect: () => void; onPlay: () => void;
}) {
  return (
    <div
      role="row"
      aria-rowindex={index}
      aria-selected={selected}
      onClick={onSelect}
      style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: ROW_H, transform: `translateY(${start}px)` }}
      className={`group grid ${GRID} gap-3 px-3 items-center cursor-pointer hover:bg-bg-surface ${
        selected ? 'bg-bg-elevated' : ''
      }`}
    >
      <button
        onClick={(e) => { e.stopPropagation(); onPlay(); }}
        disabled={!live}
        aria-label={`Play ${replay.file}`}
        title={live ? 'Check map compatibility and play' : 'Playback is available in the desktop app'}
        className="text-accent-dim flex items-center justify-center w-7 h-7 rounded hover:bg-bg-elevated disabled:opacity-40 focus-visible:outline focus-visible:outline-accent"
      >
        <span className="font-mono text-xs text-fg-dim group-hover:hidden group-focus-within:hidden">{index}</span>
        <Play size={12} fill="currentColor" className="hidden group-hover:block group-focus-within:block" />
      </button>

      <TeamNames teams={replay.teams ?? []} className="text-sm" />

      <span className="text-sm text-fg-muted truncate" title={replay.map}>{displayMapName(replay.map)}</span>

      <span className="text-xs text-fg-muted">{modeOf(replay)}</span>

      <span className="text-xs text-fg-muted" title={formatDate(replay.recorded_at)}>
        {formatRelative(replay.recorded_at)}
      </span>

      <span className="text-xs text-fg-muted font-mono text-right">{formatDuration(replay.duration_s)}</span>
    </div>
  );
}
