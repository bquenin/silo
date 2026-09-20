// Tauri command bridge. All types match the Rust serde shapes in
// src-tauri/src/db/mod.rs and src-tauri/src/ingest.rs.

import { Channel, invoke } from '@tauri-apps/api/core';

export interface BackendPlayerSummary {
  slot: number;
  name: string;
  clan: string;
  chosen_faction: string;
  actual_faction: string;
  team: number;
  is_ai: boolean;
  is_observer: boolean;
  is_commentator: boolean;
}

export interface BackendReplayRow {
  id: number;
  file_hash: string;
  file_path: string | null;
  map_name: string;
  n_players: number;
  timestamp: number;
  imported_at: number;
  duration_frames: number | null;
  players: BackendPlayerSummary[];
}

export interface ReplayCursor {
  imported_at: number;
  id: number;
}

export interface IngestReport {
  scanned: number;
  inserted: number;
  duplicates: number;
  errors: { path: string; message: string }[];
}

/** True when we're running inside the Tauri shell (vs `npm run dev` in a browser). */
export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function listReplays(limit = 1000, before?: ReplayCursor): Promise<BackendReplayRow[]> {
  return invoke('list_replays', { limit, before: before ?? null });
}

export async function countReplays(): Promise<number> {
  return invoke('count_replays');
}

export async function ingestPath(path: string): Promise<IngestReport> {
  return invoke('ingest', { path });
}

export interface PlaybackSettings {
  game_path: string | null;
}

export interface PlaybackReport {
  can_play: boolean;
  message: string;
  required_revision: string | null;
  game_path: string | null;
  details: string[];
}

export interface PlaybackProgress {
  phase: 'checking' | 'locating' | 'downloading' | 'extracting' | 'verifying' | 'preparing' | 'launching';
  message: string;
  completed: number;
  total: number | null;
}

export async function playbackSettings(): Promise<PlaybackSettings> {
  return invoke('playback_settings');
}

export async function setGameInstallation(path: string): Promise<PlaybackSettings> {
  return invoke('set_game_installation', { path });
}

export async function checkPlayback(replayId: number): Promise<PlaybackReport> {
  return invoke('check_playback', { replayId });
}

export async function launchReplay(replayId: number, requestId: string,
  onProgress: (progress: PlaybackProgress) => void): Promise<{ pid: number }> {
  const channel = new Channel<PlaybackProgress>();
  channel.onmessage = onProgress;
  return invoke('launch_replay', { replayId, requestId, onProgress: channel });
}

export async function cancelPlayback(requestId: string): Promise<boolean> {
  return invoke('cancel_playback', { requestId });
}
