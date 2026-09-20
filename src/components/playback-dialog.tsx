import { useCallback, useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { Check, FolderOpen, Loader2, Play, X } from 'lucide-react';
import {
  cancelPlayback, checkPlayback, launchReplay, playbackSettings, setGameInstallation,
  type PlaybackReport, type PlaybackSettings, type PlaybackProgress,
} from '../lib/backend';
import type { Replay } from '../lib/types';

function size(bytes: number) {
  return bytes >= 1024 ** 3 ? `${(bytes / 1024 ** 3).toFixed(1)} GB` : `${Math.round(bytes / 1024 ** 2)} MB`;
}

export function PlaybackDialog({ replay, onClose }: { replay: Replay; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const mounted = useRef(false);
  const request = useRef<string | null>(null);
  const started = useRef(false);
  const [settings, setSettings] = useState<PlaybackSettings | null>(null);
  const [report, setReport] = useState<PlaybackReport | null>(null);
  const [checking, setChecking] = useState(true);
  const [choosing, setChoosing] = useState(false);
  const [progress, setProgress] = useState<PlaybackProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cancelled, setCancelled] = useState(false);
  const [launched, setLaunched] = useState(false);

  const play = useCallback(async () => {
    if (request.current) return;
    const id = crypto.randomUUID();
    request.current = id;
    setError(null);
    setCancelled(false);
    setProgress({ phase: 'checking', message: 'Preparing replay…', completed: 0, total: null });
    try {
      await launchReplay(Number(replay.id), id, (event) => {
        if (mounted.current && request.current === id) setProgress(event);
      });
      if (mounted.current) setLaunched(true);
    } catch (reason) {
      if (mounted.current) {
        if (String(reason).includes('Replay preparation cancelled.')) setCancelled(true);
        else setError(String(reason));
      }
    } finally {
      if (request.current === id) request.current = null;
      if (mounted.current) setProgress(null);
    }
  }, [replay.id]);

  useEffect(() => {
    mounted.current = true;
    const element = dialog.current;
    element?.showModal();
    return () => {
      mounted.current = false;
      if (request.current) void cancelPlayback(request.current).catch(() => {});
      element?.close();
    };
  }, []);

  useEffect(() => {
    let active = true;
    void Promise.all([playbackSettings(), checkPlayback(Number(replay.id))])
      .then(([installation, result]) => {
        if (!active) return;
        setSettings(installation);
        setReport(result);
        setChecking(false);
        // Clicking Play in the library is the intent to launch; preparation
        // starts here without another confirmation inside the dialog.
        if (result.can_play && !started.current) {
          started.current = true;
          void play();
        }
      })
      .catch((reason: unknown) => { if (active) setError(String(reason)); })
      .finally(() => { if (active) setChecking(false); });
    return () => { active = false; };
  }, [replay.id, play]);

  async function chooseGame() {
    setChoosing(true);
    setError(null);
    try {
      const selected = await open({
        title: "Choose your Kane's Wrath game folder",
        directory: true, multiple: false,
        defaultPath: settings?.game_path ?? undefined,
      });
      if (!mounted.current || typeof selected !== 'string') return;
      const installation = await setGameInstallation(selected);
      if (!mounted.current) return;
      setSettings(installation);
      const result = await checkPlayback(Number(replay.id));
      if (!mounted.current) return;
      setReport(result);
      if (result.can_play) { started.current = true; void play(); }
    } catch (reason) {
      if (mounted.current) setError(String(reason));
    } finally {
      if (mounted.current) setChoosing(false);
    }
  }

  async function cancel() {
    if (!request.current) return;
    try {
      const accepted = await cancelPlayback(request.current);
      if (accepted) setProgress((value) => value && { ...value, message: 'Cancelling…' });
    } catch (reason) {
      setError(String(reason));
    }
  }

  const busy = progress !== null;
  const percent = progress?.total ? Math.min(100, Math.max(0, progress.completed / progress.total * 100)) : undefined;

  return (
    <dialog ref={dialog} onCancel={onClose}
      className="m-auto w-[min(560px,calc(100vw-32px))] max-h-[85vh] overflow-y-auto rounded-xl border border-bg-border bg-bg-surface text-fg p-6 shadow-2xl backdrop:bg-black/70"
      aria-labelledby="playback-title">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h2 id="playback-title" className="text-xl font-semibold">Play replay</h2>
          <p className="text-sm text-fg-muted mt-1 break-words">{replay.map}</p>
          {report?.required_revision && <p className="text-xs text-fg-dim mt-1">Map version {report.required_revision}</p>}
        </div>
        <button onClick={onClose} aria-label="Close playback" className="p-1 text-fg-muted hover:text-fg"><X size={20} /></button>
      </div>

      <div className="my-5 rounded-lg border border-bg-border p-4" aria-live="polite">
        {checking || progress ? (
          <>
            <p className="flex gap-2 items-center text-sm"><Loader2 size={16} className="animate-spin shrink-0" />
              {progress?.message ?? 'Finding your game…'}</p>
            {(progress?.phase === 'downloading' || progress?.phase === 'extracting') && (
              <div className="mt-3">
                <progress aria-label={progress.phase === 'downloading' ? 'Map download progress' : 'Map unpacking progress'} max={100} value={percent}
                  className="w-full h-2 accent-accent" />
                {progress.phase === 'downloading' ? (
                  <p className="text-xs text-fg-dim mt-1">
                    {size(progress.completed)}{progress.total != null ? ` of ${size(progress.total)}` : ''}
                  </p>
                ) : percent !== undefined && (
                  <p className="text-xs text-fg-dim mt-1 tabular-nums">{Math.floor(percent)}%</p>
                )}
              </div>
            )}
          </>
        ) : launched ? (
          <p className="flex gap-2 items-center text-accent"><Check size={18} /> Kane's Wrath has started.</p>
        ) : cancelled ? (
          <p className="text-sm text-fg-muted">Replay preparation cancelled.</p>
        ) : !error && report ? (
          <p className="text-sm text-fg-muted">{report.message}</p>
        ) : null}
        {error && <p role="alert" className="text-sm text-red-300 break-words">{error}</p>}
      </div>

      {!checking && !busy && !launched && !settings?.game_path && (
        <button onClick={() => void chooseGame()} disabled={choosing}
          className="flex items-center gap-2 rounded border border-bg-border px-3 py-2 text-sm hover:bg-bg-elevated disabled:opacity-40">
          <FolderOpen size={14} /> {choosing ? 'Finding game…' : 'Choose game folder'}
        </button>
      )}

      {settings?.game_path && (
        <details className="text-xs text-fg-muted mb-5">
          <summary className="cursor-pointer hover:text-fg">Troubleshooting</summary>
          <p className="mt-3 break-all">Game folder: {settings.game_path}</p>
          <p className="mt-2 break-all">Replay: {replay.file}</p>
          {report?.details.map((detail, i) => <p key={i} className="mt-2 break-all">{detail}</p>)}
          <button onClick={() => void chooseGame()} disabled={busy || choosing || checking || launched}
            className="mt-3 underline disabled:opacity-40">Change game folder</button>
        </details>
      )}

      <div className="flex justify-end items-center gap-3">
        {busy && progress?.phase !== 'launching' && (
          <button onClick={() => void cancel()} className="text-sm px-3 py-2 rounded hover:bg-bg-elevated">Cancel</button>
        )}
        {launched ? (
          <button onClick={onClose} className="text-sm px-4 py-2 rounded bg-accent text-bg">Done</button>
        ) : !busy && !checking && settings?.game_path ? (
          <button onClick={() => void play()} disabled={choosing || report?.can_play === false}
            className="flex items-center gap-2 text-sm font-semibold bg-accent text-bg px-4 py-2 rounded hover:bg-accent-dim disabled:opacity-40">
            <Play size={16} fill="currentColor" /> Play
          </button>
        ) : null}
      </div>
    </dialog>
  );
}
