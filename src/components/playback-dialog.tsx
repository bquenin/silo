import { useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { Check, FolderOpen, Loader2, Play, RefreshCw, X } from 'lucide-react';
import {
  checkPlayback, launchReplay, playbackSettings, setPlaybackConfig,
  type PlaybackReport, type PlaybackSettings, type PlaybackStatus,
} from '../lib/backend';
import type { Replay } from '../lib/types';

const STATUS_LABEL: Record<PlaybackStatus, string> = {
  ready: 'Ready to launch',
  not_configured: 'Choose your game configuration',
  replay_missing: 'Replay file missing',
  engine_mismatch: 'Different engine version required',
  map_missing: 'Map revision missing',
  map_disabled: 'Map revision installed but disabled',
  unknown: 'Compatibility needs verification',
};

export function PlaybackDialog({ replay, onClose }: { replay: Replay; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [settings, setSettings] = useState<PlaybackSettings | null>(null);
  const [report, setReport] = useState<PlaybackReport | null>(null);
  const [checking, setChecking] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [launched, setLaunched] = useState(false);
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);

  useEffect(() => {
    let active = true;
    setChecking(true);
    setReport(null);
    setError(null);
    setLaunched(false);
    void Promise.all([playbackSettings(), checkPlayback(Number(replay.id))])
      .then(([config, result]) => {
        if (active) { setSettings(config); setReport(result); }
      })
      .catch((reason: unknown) => { if (active) setError(String(reason)); })
      .finally(() => { if (active) setChecking(false); });
    return () => { active = false; };
  }, [replay.id, revision]);

  async function chooseConfiguration() {
    setBusy(true);
    setError(null);
    try {
      const selected = await open({
        title: "Select Kane's Wrath game configuration",
        filters: [{ name: 'Game configuration', extensions: ['SkuDef', 'skudef'] }],
        multiple: false,
        defaultPath: settings?.sku_path ?? undefined,
      });
      if (typeof selected !== 'string') return;
      setSettings(await setPlaybackConfig(selected));
      setReport(null);
      setRevision((value) => value + 1);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function play() {
    setBusy(true);
    setError(null);
    try {
      // The backend repeats preflight and verifies the replay's hash at launch.
      await launchReplay(Number(replay.id));
      setLaunched(true);
    } catch (reason) {
      setReport(null);
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <dialog ref={dialog} onCancel={onClose}
      className="m-auto w-[min(640px,calc(100vw-32px))] max-h-[85vh] overflow-y-auto rounded-xl border border-bg-border bg-bg-surface text-fg p-6 shadow-2xl backdrop:bg-black/70"
      aria-labelledby="playback-title">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h2 id="playback-title" className="text-xl font-semibold">Play replay</h2>
          <p className="text-sm text-fg-muted mt-1 break-words">{replay.map}</p>
          <p className="text-xs text-fg-dim mt-1 break-all">{replay.file}</p>
        </div>
        <button onClick={onClose} aria-label="Close playback" className="p-1 text-fg-muted hover:text-fg"><X size={20} /></button>
      </div>

      <div className="my-5 rounded-lg border border-bg-border p-4" aria-live="polite">
        {checking ? (
          <p className="flex gap-2 items-center text-sm"><Loader2 size={16} className="animate-spin" /> Checking the required map revision…</p>
        ) : launched ? (
          <p className="flex gap-2 items-center text-accent"><Check size={18} /> Kane's Wrath has been started.</p>
        ) : report ? (
          <>
            <p className={`font-medium ${report.status === 'ready' ? 'text-accent' : 'text-fg'}`}>{STATUS_LABEL[report.status]}</p>
            {report.required_revision && <p className="text-sm mt-1">Required map revision: <strong>{report.required_revision}</strong></p>}
            <p className="text-sm text-fg-muted mt-2">{report.message}</p>
          </>
        ) : <p className="text-sm text-fg-muted">Check compatibility again before playing.</p>}
        {error && <p role="alert" className="mt-2 text-sm text-red-300 break-words">{error}</p>}
      </div>

      <div className="flex items-start justify-between gap-4 mb-4">
        <div className="min-w-0 text-sm">
          <p className="text-fg-muted">Game configuration</p>
          <p className="text-xs mt-1 break-all">{settings?.sku_path?.split(/[\\/]/).pop() ?? 'Not selected'}</p>
        </div>
        <button onClick={() => void chooseConfiguration()} disabled={busy || checking}
          className="flex items-center gap-2 shrink-0 rounded border border-bg-border px-3 py-2 text-xs hover:bg-bg-elevated disabled:opacity-40">
          <FolderOpen size={14} /> Choose configuration
        </button>
      </div>
      {!settings?.sku_path && <p className="text-xs text-fg-muted mb-4">Select the .SkuDef file for your installed game version and language in the Kane's Wrath installation folder.</p>}

      {report && (
        <details className="text-xs text-fg-muted mb-5">
          <summary className="cursor-pointer hover:text-fg">Map and configuration details</summary>
          <dl className="mt-3 space-y-2">
            <div><dt>Required map path</dt><dd className="font-mono break-all text-fg">{report.map_path}</dd></div>
            <div><dt>Recorded map CRC</dt><dd className="font-mono text-fg">{report.recorded_crc || 'Unavailable'}</dd></div>
            <div><dt>Configuration</dt><dd className="break-all text-fg">{report.sku_path ?? 'Not selected'}</dd></div>
          </dl>
          {report.providers.map((provider) => (
            <p key={provider.path} className="mt-3 break-all">{provider.enabled ? 'Enabled' : 'Installed, disabled'}: {provider.path}</p>
          ))}
          {report.warnings.length > 0 && <ul className="mt-3 list-disc pl-4 space-y-1">{report.warnings.map((warning, i) => <li key={i}>{warning}</li>)}</ul>}
        </details>
      )}
      <div className="flex justify-end items-center gap-3">
        <button onClick={() => setRevision((value) => value + 1)} disabled={checking || busy}
          className="flex items-center gap-2 text-sm px-3 py-2 rounded hover:bg-bg-elevated disabled:opacity-40">
          <RefreshCw size={14} /> Check again
        </button>
        <button onClick={() => void play()} disabled={checking || busy || launched || report?.status !== 'ready'}
          className="flex items-center gap-2 text-sm font-semibold bg-accent text-bg px-4 py-2 rounded hover:bg-accent-dim disabled:opacity-40">
          {busy ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} fill="currentColor" />} Play
        </button>
      </div>
    </dialog>
  );
}
