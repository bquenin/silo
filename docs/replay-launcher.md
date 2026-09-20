# Replay playback

Click **Play** on a replay. Tacitus finds the game, checks installed and cached
content, downloads missing supported content, and launches Kane's Wrath.
There is no separate download action or configuration-file picker. Progress
and cancellation are available during preparation; a failed attempt can be
retried with the same Play button. If detection fails, choose the game folder.

Steam's default and additional libraries and common EA/Origin installation
folders are searched. The selected folder is saved beside the catalogue in
`launcher.json`. Older `sku_path` settings are migrated in memory to their
containing game folder, without rewriting the old settings just by reading them.

## Content resolution

A replay's internal map path identifies the exact asset and revision. The
three-hex-digit replay prefix (such as `283`) is removed and separators/case
are normalized, while the complete directory and revision suffix are retained.
A thumbnail, display name, `FakeMapID`, newer patch, or recorded `MC=` value
cannot substitute for that asset. The recorded CRC is not treated as a unique
file checksum.

Resolution checks the persistent cache and then installed BIG archives,
including packs disabled in the user's current configuration. An installed
community map needs a revision-named script archive containing
`data/scripts/scripts.lua`; a shared installed `102Scripts.big` alone cannot
establish which historical patch it belongs to.

When content is missing, the kaneswrath.com version lists are searched for
the exact R22–R25 revision, across 1v1, 2v2, 4v4, legacy and combined pack pages. Missing
historical downloads fail explicitly; Tacitus never substitutes the latest
version. Download support also depends on the available package format:
direct BIG files in ZIPs and the Unicode, non-solid LZMA NSISBI layout used
by the R24g pack are supported. Other layouts fail without running installers.

The extractor reads the installer as data and selects the revision's map
archives, scripts and community texture archive from its Patch103 payload.
It does not execute installer instructions, plugins, replacement engines or
configuration changes. Extraction has bounds on file counts, offsets, memory,
expanded sizes and output paths. No external extraction program is required.

## Cache and temporary sessions

The default cache is `%LOCALAPPDATA%\tacitus\playback`:

- `packages/pack-*/`: complete extracted replay-content archives and an index
  containing the source URL, exact revision, download SHA-256, archive sizes,
  archive SHA-256 values and internal asset paths.
- `downloads/`: complete, hashed ZIPs retained if extraction fails, avoiding
  another large download on retry. Successful publication removes that copy.
- `staging/`: incomplete extraction/download work, excluded from resolution.
- `sessions/replay-*/`: generated configuration for a particular launch.

Revisions coexist in separate package directories. Before playback, cached
archives are checked against their stored hashes. Incomplete or corrupt
packages are not launch candidates. Hashes detect subsequent corruption;
the original download's provenance is the provider's HTTPS endpoint, not
an independently signed publisher manifest. Once cached, playback does not
contact the provider.

Downloads hold an OS file lock on the cache to prevent overlapping writers.
After a crash, the next download removes only marked abandoned staging
folders while holding that lock. Unmarked folders are left alone.

The temporary configuration mounts the resolved replay content ahead of the
installed base-game, language and audio layers. It does not read the user's
top-level patch selection or write the game's SkuDef, Patch103 configuration
or installed packs. Paths to borrowed archives remain in their original
locations. Unsupported custom maps, unidentified community revisions and
missing engine/base content produce an explanation instead of guessing.

The game is started directly, in its installation directory:

```text
cnc3ep1.dat -replayGame <absolute replay path> -win -config <temporary SkuDef>
```

The replay SHA-256 is checked against the catalogue before preparation.
A running KW process blocks another launch, and that check is repeated after
preparation. Launch arguments remain separate to preserve spaces, Unicode
and special characters.

Tacitus retains the session until the game exits. If Tacitus exits first,
the generated files remain available to the game; the next Play recovers
marked stale sessions once no KW process is running. Cleanup never removes
cached or borrowed content. This isolates content selection, **not** the
game's ordinary preferences, logs or profile writes.

The game process inherits the user's environment. Tacitus does not terminate
existing game processes or clear their logs.

A successful launch reports a process ID. It does not prove that every
supported replay remains deterministic through its entire duration.

## CLI

```powershell
tacitus-cli check 123 --json
tacitus-cli play 123 --dry-run --json
tacitus-cli play 123 --game "C:\Games\KW"
tacitus-cli prepare 123 --offline --json
```

`check` and `play --dry-run` inspect without downloading or launching. A
positive report can mean that Play can download missing content.
`prepare` downloads if needed and retains a temporary launch plan for
inspection without starting the game; `--offline` forbids downloads.
`--cache DIR` overrides the cache. `--db` selects the catalogue as usual.

The advanced `--sku FILE` override retains the former manual compatibility
checker/launcher for diagnostics. It does not use automatic preparation and
cannot be combined with `--game`, `--offline` or `prepare`.

## Implementation references and checks

The [NSISBI project](https://sourceforge.net/projects/nsisbi/), NSIS's
`Source/exehead/fileform.h`, and the
[NSISExtractor format notes](https://github.com/KokerZhou/NSISExtractor/blob/main/docs/nsis-format-notes.md)
inform the bounded installer reader. The historical source is the
[R24 1v1 map pack version list](https://kaneswrath.com/download/r24-1vs1-map-pack/).

Tests cover exact revisions, disabled local packs, offline reuse, concurrent
cached revisions, shared-script ambiguity, corrupt and incomplete caches,
truncated HTTP responses, cancellation during HTTP waits and preparation,
bounded extraction, path containment, settings migration, stale-session
ownership, and the one-action frontend flow. Legacy checker fixtures remain
in `src-tauri/tests/playback.rs`.

On 2026-09-19, the R24g 1v1 ZIP was downloaded from the historical version
list and extracted without executing its installer. Catalogue replay 1153
(`[R24] Abandoned Subway`) prepared again offline and reached active replay
playback with a temporary config outside the game directory. The CLI had
exited while its session remained
available to the game. The test copy was then closed. All 37 game configuration
files matched their pre-test SHA-256 values. This was a brief loading/playback
check, not a full-duration determinism test; further live testing was deferred
at the user's request.
