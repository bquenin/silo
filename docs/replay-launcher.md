# Replay launcher

Tacitus checks a catalogue replay against an existing Kane's Wrath `.SkuDef`
configuration before offering Play. Select the play icon on a replay row to
see its required map revision, availability, and game configuration. The
default Steam English 1.2 configuration is detected when present; use **Choose
configuration** for another installation, language, or version.

The selection is saved beside the catalogue in `launcher.json`. Choosing a
configuration does not modify the game's files. After installing or enabling
a required pack through your usual map manager, use **Check again**.

## Compatibility decisions

| Status | Meaning |
|---|---|
| Ready to launch | Engine version matches, exactly one enabled source contains the exact map asset, and required community scripts are present. |
| Map revision missing | No scanned archive provides the exact `.map` entry. A newer revision or thumbnail does not satisfy it. |
| Installed but disabled | The map is in an archive on disk, but that archive is outside the selected configuration's content chain. |
| Different engine version | The selected `RetailExe/<version>/cnc3ep1.dat` does not match the replay header. |
| Compatibility needs verification | Unsupported/custom map path, community map without a revision suffix, conflicting providers, unreadable active archives/configuration, or missing community scripts. |

The checker follows nested `add-config` directives relative to each containing
file, indexes `add-big` archives, and checks loose assets under explicit
`add-search-path` directories. It also indexes other installed `.big` files
to identify disabled packs. BIGF/BIG4 payloads are not extracted; only bounded
archive directories are read.

Replay `M=` values commonly have a three-hex-digit prefix, such as `283`.
Matching strips that prefix, normalizes case/separators, and preserves the
entire map directory, including `__24g`, `__23z`, etc. Only an exact `.map`
entry counts. `map_id` is not an identity key: real replays commonly contain
`FakeMapID`. Display names can be localized and do not identify revisions.

**Verification limits:** Ready means the supported preflight checks passed,
not that deterministic replay playback has been proven. The original `MC=`
value is shown but not recomputed: many community maps share revision markers
(for example `19`), so it must not be treated as an ordinary unique file CRC.
The checker requires an enabled `102Scripts.big` or `R<revision>Scripts.big`
containing `data/scripts/scripts.lua` for community maps, and names the
identified provider in the report. Stock `Core/Misc.big`, unrelated archives,
and unidentified loose scripts cannot satisfy this check. It does not verify
the scripts' exact revision or all transitive asset dependencies. Custom user maps and older
community maps lacking explicit revision paths remain unknown. Missing
configuration references appear in the detailed report; absent unrelated
packs do not by themselves hide an available map.

## Launch behavior

Play repeats the compatibility check and verifies the replay file's SHA-256
against the catalogue. It starts the selected engine directly with separate
arguments, preserving paths with spaces, Unicode, or `#`:

```text
cnc3ep1.dat -replayGame <absolute replay path> -win -config <absolute SkuDef>
```

The working directory is the game root. An already running KW process blocks
another launch. Tacitus does not kill games or automatically close playback.
The start result reports a process ID, not successful loading of the replay.
Imports store absolute replay paths. Reimport the original folder to update
paths in catalogue entries created by an older version using relative paths.

The game process inherits the user's environment. Tacitus does not terminate
existing game processes or clear their logs.

## CLI

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin tacitus-cli -- check 123 --json
cargo run --manifest-path src-tauri/Cargo.toml --bin tacitus-cli -- play 123 --dry-run --json
cargo run --manifest-path src-tauri/Cargo.toml --bin tacitus-cli -- play 123 --sku "C:\Games\KW\CNC3EP1_english_1.2.SkuDef"
```

`check` and `play --dry-run` emit a compatibility report (including the launch
plan when ready) without starting a process. Non-ready checks exit nonzero.
`--sku` overrides the saved selection for that invocation; `--db` selects the
catalogue as with other commands.

## Verification

Archive/config fixtures in `src-tauri/tests/playback.rs` exercise
compatibility without a game install.
