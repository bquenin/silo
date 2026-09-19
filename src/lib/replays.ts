import type { BackendReplayRow } from './backend';
import type { Replay } from './types';

/** Adapt a `BackendReplayRow` to the Replay shape the UI components expect. */
export function rowToReplay(row: BackendReplayRow): Replay {
  const realPlayers = row.players
    .filter((p) => !p.is_observer && !p.is_commentator)
    .sort((a, b) => a.slot - b.slot);

  const players = realPlayers.map((p) => ({
    slot: p.slot,
    name: p.name,
    chosen: p.chosen_faction,
    actual: p.actual_faction,
    team: p.team,
  }));

  // The backend normalizes the header's -1/no-team value to 0.
  // Players with the same positive team number stand on the same side.
  // For FFA / no-team games, each player is their own "team".
  const teamMap = new Map<number, typeof players>();
  for (const p of players) {
    const t = p.team != null && p.team >= 1 ? p.team : -(p.slot + 100); // FFA → unique synthetic team
    if (!teamMap.has(t)) teamMap.set(t, []);
    teamMap.get(t)!.push(p);
  }
  const teams = [...teamMap.values()].sort((a, b) =>
    (a[0]?.slot ?? 0) - (b[0]?.slot ?? 0)
  );

  // 1v1 convenience fields (used for the cover gradient on row + detail pane)
  let bro_alias: string | undefined;
  let bro_actual: string | undefined;
  let opponent_name: string | undefined;
  let opponent_actual: string | undefined;
  if (teams.length === 2 && teams[0].length === 1 && teams[1].length === 1) {
    bro_alias = teams[0][0].name;
    bro_actual = teams[0][0].actual;
    opponent_name = teams[1][0].name;
    opponent_actual = teams[1][0].actual;
  }

  return {
    id: String(row.id),
    file: row.file_path?.split(/[\\/]/).pop() ?? row.file_hash.slice(0, 12),
    map: row.map_name,
    n_players: realPlayers.length,
    duration_s: row.duration_frames != null && row.duration_frames > 0
      ? Math.floor(row.duration_frames / 15) : undefined,
    players,
    teams,
    bro_alias,
    bro_actual,
    opponent_name,
    opponent_actual,
    recorded_at: row.timestamp ? new Date(row.timestamp * 1000).toISOString() : undefined,
  };
}


export function modeOf(replay: Pick<Replay, 'teams' | 'n_players'>): string {
  const sizes = replay.teams.map((team) => team.length).filter(Boolean).sort((a, b) => b - a);
  if (sizes.length >= 3 && sizes.every((n) => n === 1)) return 'FFA';
  if (sizes.length >= 2) return sizes.join('v');
  return `${sizes.reduce((sum, n) => sum + n, 0)}p`;
}

export function formatDuration(seconds?: number): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return '--:--';
  const whole = Math.floor(seconds);
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
}
