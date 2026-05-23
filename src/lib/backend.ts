// Tauri command bridge. All types match the Rust serde shapes in
// src-tauri/src/db/mod.rs and src-tauri/src/ingest.rs.

import { invoke } from '@tauri-apps/api/core';

export interface BackendReplayRow {
  id: number;
  file_hash: string;
  file_path: string | null;
  map_name: string;
  n_players: number;
  timestamp: number;
  imported_at: number;
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

export async function listReplays(limit = 1000): Promise<BackendReplayRow[]> {
  return invoke('list_replays', { limit });
}

export async function countReplays(): Promise<number> {
  return invoke('count_replays');
}

export async function ingestPath(path: string): Promise<IngestReport> {
  return invoke('ingest', { path });
}
