import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useReplays } from '../lib/use-replays';
import { rowToReplay } from '../lib/replays';
import { row } from '../test/fixtures';
import { SpotifyLayout } from './spotify-layout';

vi.mock('../lib/use-replays');
afterEach(cleanup);
beforeEach(() => vi.mocked(useReplays).mockReturnValue({
  replays: [rowToReplay(row([1, 1, 2, 2, 3, 3, 4, 4]))], total: 1,
  loading: false, error: null, live: true, refresh: vi.fn(), importFolder: vi.fn(),
}));

it('renders all eight participants and their factions in a four-team match', () => {
  render(<SpotifyLayout />);
  const replayRow = screen.getByRole('button', { name: 'Play replay-1.KWReplay' }).parentElement!;
  for (let i = 0; i < 8; i++) expect(replayRow.textContent).toContain(`Player${i}`);
  expect(replayRow.querySelectorAll('[title="GDI"]')).toHaveLength(8);
  expect(replayRow.textContent).toContain('2v2v2v2');
  expect(replayRow.textContent).toContain('20:00');
});

it('shows backend errors with retry instead of the empty-library prompt', () => {
  const refresh = vi.fn();
  vi.mocked(useReplays).mockReturnValue({ ...useReplays(), replays: [], total: 0, error: 'Database unavailable', refresh });
  render(<SpotifyLayout />);
  expect(screen.getByRole('alert').textContent).toContain('Database unavailable');
  expect(screen.queryByText('No replays in the catalogue yet.')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Retry loading catalogue' }));
  expect(refresh).toHaveBeenCalledOnce();
});
