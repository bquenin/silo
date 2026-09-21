import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { associateReplayFiles } from '../lib/backend';
import { useReplays } from '../lib/use-replays';
import { rowToReplay } from '../lib/replays';
import { row } from '../test/fixtures';
import { LibraryLayout } from './library-layout';

vi.mock('../lib/use-replays');
vi.mock('../lib/backend', async (loadOriginal) => ({
  ...await loadOriginal<typeof import('../lib/backend')>(),
  replayFileAssociation: vi.fn().mockResolvedValue({ supported: true, associated: false }),
  associateReplayFiles: vi.fn().mockResolvedValue({ supported: true, associated: true }),
}));
afterEach(cleanup);
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useReplays).mockReturnValue({
    replays: [rowToReplay(row([1, 1, 2, 2, 3, 3, 4, 4]))], total: 1,
    loading: false, error: null, openedReport: null, live: true, refresh: vi.fn(), importFolder: vi.fn(),
  });
});

it('renders all eight participants and their factions in a four-team match', () => {
  render(<LibraryLayout />);
  const replayRow = screen.getByRole('button', { name: 'Play replay-1.KWReplay' }).parentElement!;
  for (let i = 0; i < 8; i++) expect(replayRow.textContent).toContain(`Player${i}`);
  expect(replayRow.querySelectorAll('[title="GDI"]')).toHaveLength(8);
  expect(replayRow.textContent).toContain('2v2v2v2');
  expect(replayRow.textContent).toContain('20:00');
});

it('shows backend errors with retry instead of the empty-library prompt', () => {
  const refresh = vi.fn();
  vi.mocked(useReplays).mockReturnValue({ ...useReplays(), replays: [], total: 0, error: 'Database unavailable', refresh });
  render(<LibraryLayout />);
  expect(screen.getByRole('alert').textContent).toContain('Database unavailable');
  expect(screen.queryByText('No replays in the catalogue yet.')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Retry loading catalogue' }));
  expect(refresh).toHaveBeenCalledOnce();
});

it('associates replay files only after the user chooses the toolbar action', async () => {
  render(<LibraryLayout />);
  expect(associateReplayFiles).not.toHaveBeenCalled();

  fireEvent.click(screen.getByRole('button', { name: 'Associate .kwreplay' }));

  await waitFor(() => expect(
    screen.getByRole('button', { name: '.kwreplay associated' }).hasAttribute('disabled'),
  ).toBe(true));
  expect(associateReplayFiles).toHaveBeenCalledOnce();
});
