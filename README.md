# Tacitus

A replay-corpus manager for **Command & Conquer 3: Kane's Wrath**.

In the C&C lore, the Tacitus is the alien artifact that holds *all the
knowledge*. This is the same idea, scoped to KW replays — a single
desktop app that catalogues every `.kwreplay` file you own, extracts
its metadata (players, factions, map, length, version, …), and lets
you search, filter, and re-watch them via the live engine.

## Status

**Pre-alpha.** UI prototype works against mock data; the binary
parser, SQLite-backed catalogue, and playback integration land next.

## Why exist

`gamereplays.org` hosts thousands of public KW replays, and a number
of tools touch the format (KWReplayAutoSaver, ReplayTool, the
closed-source *Command Post*). What's missing is an **open-source**
manager with both a polished UI **and** a command-line interface
with structured JSON output.

## What it will do

- **Ingest** `.kwreplay` files from any folder; auto-extract metadata
- **Resolve Random** — when a player picked Random in the lobby, the
  actual faction is decoded from the first build command (the template
  hash carries a faction prefix)
- **Search/filter** by player, faction (including resolved Random),
  map, opponent, year, duration, tag
- **Map-pack awareness** — each replay's required map → required map
  pack → which packs are installed locally → "needs R23z, not installed"
- **Playback** — launch replays in the game
- **CLI** with `--json` output for importing and searching replays

## Stack

- **[Tauri 2](https://tauri.app/)** for the desktop shell — single-file
  binary, native window, WebView2 on Windows
- **Rust** for the backend — binary parsing, SQLite, the CLI
- **React + TypeScript + Tailwind** for the UI
- **SQLite** as the catalogue store

## Dev

Requires:
- Rust toolchain (`rustup` ≥ 1.80)
- Node.js LTS

```sh
npm install
npm run tauri dev      # opens native window
```

For frontend-only iteration (no Rust rebuild loop):

```sh
npm run dev            # Vite at localhost:1420 — preview only
```

## License

MIT (planned). See [LICENSE](./LICENSE) once added.

## Acknowledgements

- [`forcecore/KWReplayAutoSaver`](https://github.com/forcecore/KWReplayAutoSaver)
  — Python reference parser for the `.KWReplay` binary format.
- [`Aquatech5/ReplayTool`](https://github.com/Aquatech5/ReplayTool) —
  fork with extended metadata extraction.
- [Command Post](https://cgf-uploads.net/cp) — closed-source but a
  reference for feature scope.
