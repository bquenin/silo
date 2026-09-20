import { StrictMode } from 'react';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { open } from '@tauri-apps/plugin-dialog';
import { cancelPlayback, checkPlayback, launchReplay, playbackSettings, setGameInstallation, type PlaybackProgress } from '../lib/backend';
import { rowToReplay } from '../lib/replays';
import { row } from '../test/fixtures';
import { PlaybackDialog } from './playback-dialog';

vi.mock('../lib/backend', () => ({ cancelPlayback: vi.fn(), checkPlayback: vi.fn(), launchReplay: vi.fn(), playbackSettings: vi.fn(), setGameInstallation: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

const replay = rowToReplay(row([0, 0], 42));
const ready = { can_play: true, message: 'Content will be prepared when you press Play.', required_revision: 'R24g', game_path: 'C:/Games/KW', details: [] };

beforeEach(() => {
  vi.resetAllMocks();
  HTMLDialogElement.prototype.showModal = function () { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function () { this.removeAttribute('open'); };
  vi.mocked(playbackSettings).mockResolvedValue({ game_path: 'C:/Games/KW' });
  vi.mocked(checkPlayback).mockResolvedValue(ready);
  vi.mocked(cancelPlayback).mockResolvedValue(true);
  vi.mocked(launchReplay).mockResolvedValue({ pid: 123 });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); });

it('starts preparation once from the original Play action, including in StrictMode', async () => {
  render(<StrictMode><PlaybackDialog replay={replay} onClose={vi.fn()} /></StrictMode>);
  await screen.findByText("Kane's Wrath has started.");
  expect(launchReplay).toHaveBeenCalledOnce();
  expect(launchReplay).toHaveBeenCalledWith(42, expect.any(String), expect.any(Function));
  expect(screen.queryByText(/Download & Play|SkuDef|Choose configuration/i)).toBeNull();
});

it('shows download progress and cancels the matching request', async () => {
  let update!: (progress: PlaybackProgress) => void;
  let reject!: (reason: string) => void;
  vi.mocked(launchReplay).mockImplementation((_id, _request, handler) => {
    update = handler;
    return new Promise((_resolve, fail) => { reject = fail; });
  });
  render(<PlaybackDialog replay={replay} onClose={vi.fn()} />);
  await waitFor(() => expect(launchReplay).toHaveBeenCalledOnce());
  act(() => update({ phase: 'downloading', message: 'Downloading R24g replay content…', completed: 50 * 1024 ** 2, total: 100 * 1024 ** 2 }));
  expect(screen.getByRole('progressbar').getAttribute('value')).toBe('50');
  expect(screen.getByText('50 MB of 100 MB')).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  await waitFor(() => expect(cancelPlayback).toHaveBeenCalledWith(vi.mocked(launchReplay).mock.calls[0][1]));
  await act(async () => reject('Replay preparation cancelled.'));
  expect(screen.getByRole('button', { name: 'Play' })).toBeTruthy();
});

it('asks only for the game folder when detection fails and continues after selection', async () => {
  vi.mocked(playbackSettings).mockResolvedValue({ game_path: null });
  vi.mocked(checkPlayback).mockResolvedValueOnce({ ...ready, can_play: false, game_path: null, message: 'Choose your game folder.' });
  vi.mocked(open).mockResolvedValue('D:/Games/KW');
  vi.mocked(setGameInstallation).mockResolvedValue({ game_path: 'D:/Games/KW' });
  render(<PlaybackDialog replay={replay} onClose={vi.fn()} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Choose game folder' }));
  await screen.findByText("Kane's Wrath has started.");
  expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: true, multiple: false }));
  expect(setGameInstallation).toHaveBeenCalledWith('D:/Games/KW');
  expect(launchReplay).toHaveBeenCalledOnce();
});

it('shows unpacking progress after the download and clears it for verification', async () => {
  let update!: (progress: PlaybackProgress) => void;
  let finish!: (value: { pid: number }) => void;
  vi.mocked(launchReplay).mockImplementation((_id, _request, handler) => {
    update = handler;
    return new Promise((resolve) => { finish = resolve; });
  });
  render(<PlaybackDialog replay={replay} onClose={vi.fn()} />);
  await waitFor(() => expect(launchReplay).toHaveBeenCalledOnce());
  act(() => update({ phase: 'downloading', message: 'Downloading replay content…', completed: 100, total: 100 }));
  expect(screen.getByRole('progressbar', { name: 'Map download progress' }).getAttribute('value')).toBe('100');
  act(() => update({ phase: 'extracting', message: 'Unpacking replay content…', completed: 0, total: null }));
  expect(screen.getByRole('progressbar', { name: 'Map unpacking progress' }).hasAttribute('value')).toBe(false);
  expect(screen.queryByText('100%')).toBeNull();
  act(() => update({ phase: 'extracting', message: 'Unpacking replay content…', completed: 45, total: 100 }));
  expect(screen.getByRole('progressbar', { name: 'Map unpacking progress' }).getAttribute('value')).toBe('45');
  expect(screen.getByText('45%')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
  act(() => update({ phase: 'extracting', message: 'Unpacking replay content…', completed: 999, total: 1000 }));
  expect(screen.getByText('99%')).toBeTruthy();
  act(() => update({ phase: 'extracting', message: 'Unpacking replay content…', completed: 1000, total: 1000 }));
  expect(screen.getByText('100%')).toBeTruthy();
  act(() => update({ phase: 'verifying', message: 'Verifying downloaded map files…', completed: 0, total: null }));
  expect(screen.queryByRole('progressbar')).toBeNull();
  await act(async () => finish({ pid: 123 }));
  expect(screen.getByText("Kane's Wrath has started.")).toBeTruthy();
});

it('offers the same Play action after a failed download and cancels on close', async () => {
  vi.mocked(launchReplay).mockRejectedValueOnce('Download interrupted. Press Play to retry.');
  const { unmount } = render(<PlaybackDialog replay={replay} onClose={vi.fn()} />);
  expect((await screen.findByRole('alert')).textContent).toContain('Download interrupted');
  let finish!: (value: { pid: number }) => void;
  vi.mocked(launchReplay).mockImplementationOnce(() => new Promise((resolve) => { finish = resolve; }));
  fireEvent.click(screen.getByRole('button', { name: 'Play' }));
  await waitFor(() => expect(launchReplay).toHaveBeenCalledTimes(2));
  const id = vi.mocked(launchReplay).mock.calls[1][1];
  unmount();
  expect(cancelPlayback).toHaveBeenCalledWith(id);
  await act(async () => finish({ pid: 123 }));
});

it('does not start playback when a folder selection completes after closing', async () => {
  vi.mocked(playbackSettings).mockResolvedValue({ game_path: null });
  vi.mocked(checkPlayback).mockResolvedValue({ ...ready, can_play: false, game_path: null });
  let select!: (path: string) => void;
  vi.mocked(open).mockImplementationOnce(() => new Promise((resolve) => { select = resolve; }));
  const { unmount } = render(<PlaybackDialog replay={replay} onClose={vi.fn()} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Choose game folder' }));
  unmount();
  await act(async () => select('D:/Games/KW'));
  expect(setGameInstallation).not.toHaveBeenCalled();
  expect(launchReplay).not.toHaveBeenCalled();
});
