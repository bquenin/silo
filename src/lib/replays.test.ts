import { describe, expect, it } from 'vitest';
import { applyFilters, applySort } from './filter-sort';
import { formatDuration, modeOf, rowToReplay } from './replays';
import { row } from '../test/fixtures';


describe('replay presentation', () => {
  it('preserves real durations at 15 ticks per second and sorts by them', () => {
    const long = rowToReplay(row([0, 0], 1));
    const short = rowToReplay({ ...row([0, 0], 2), duration_frames: 9000 });
    expect(formatDuration(long.duration_s)).toBe('20:00');
    expect(applySort([long, short], { key: 'length', dir: 'asc' }).map((r) => r.id)).toEqual(['2', '1']);
    expect(rowToReplay({ ...row([]), duration_frames: null }).duration_s).toBeUndefined();
  });

  it.each([
    [[0, 0], '1v1'], [[1, 1, 2, 2], '2v2'], [[0, 0, 0, 0], 'FFA'],
    [[1, 2, 2], '2v1'], [[1, 1, 2, 2, 3, 3, 4, 4], '2v2v2v2'],
  ] as const)('labels teams %j as %s', (teams, expected) => {
    expect(modeOf(rowToReplay(row([...teams])))).toBe(expected);
  });

  it('excludes spectators and rejects FFA or uneven teams from balanced filters', () => {
    const data = row([0, 0, 1]);
    data.players[2].is_commentator = true;
    expect(rowToReplay(data).n_players).toBe(2);
    expect(modeOf(rowToReplay(data))).toBe('1v1');
    const candidates = [[0, 0, 0, 0], [1, 1, 1, 2], [1, 1, 2, 2]].map((t, i) => rowToReplay(row(t, i)));
    expect(applyFilters(candidates, { search: '', factions: new Set(), mode: '2v2' }).map((r) => r.id)).toEqual(['2']);
  });
});
