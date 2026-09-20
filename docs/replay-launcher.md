# Replay playback

Click **Play** on a replay. Tacitus finds the game, checks installed and cached
content, downloads missing supported content, and launches Kane's Wrath.
There is no separate download action or configuration-file picker. Progress
and cancellation are available during preparation; a failed attempt can be
retried with the same Play button. If detection fails, choose the game folder.

Downloads and unpacking show progress bars. Unpacking starts with an
indeterminate bar while the package is opened, then reports progress across
all selected files without resetting between them. For installer payloads,
the percentage tracks compressed bytes processed; for direct ZIP contents,
it tracks extracted bytes. Verification is shown as a separate step.
Decompression remains single-threaded.

Steam's default and additional libraries and common EA/Origin installation
folders are searched. The selected folder is saved beside the catalogue in
`launcher.json`. Older `sku_path` settings are migrated in memory to their
containing game folder, without rewriting the old settings just by reading them.

## Content resolution

A replay's internal map path identifies the exact asset and revision. The
three-hex-digit replay prefix (such as `283`) is removed and separators/case
are normalized, while the complete directory and revision suffix are retained.
A thumbnail, display name, `FakeMapID`, or newer patch cannot substitute for
that asset. Tacitus also compares the replay's hexadecimal `MC=` value with
the exact map's compiled `MapMetaData` entry. Several releases reuse the same
path: an R18 replay with `MC=2B` needs the original R18d package, while `MC=2C`
needs R18e. The release label alone cannot establish compatibility.

The parser reads manifest version 5, including old streams with one asset per
map and recent streams containing a list. Stock maps use the active engine's
metadata, including its bounded RefPack compression. Community MC values are
compatibility codes, not cryptographic file checksums. Archive SHA-256 checks
separately protect cached content from changes after download.

Unversioned `1.02+ edition` maps are selected by exact path and compiled MC.
Installed custom maps in the game's user `Maps` directory require the exact
directory and filename plus the SAGE rotate/add checksum of the complete map
file. A missing or different custom map fails explicitly.

Resolution checks the persistent cache and then installed BIG archives,
including packs disabled in the user's current configuration. An installed
community map needs a revision-named script archive containing
`data/scripts/scripts.lua`; a shared installed `102Scripts.big` alone cannot
establish which historical patch it belongs to. Cached packages retain their
own bundled scripts. Original R2–R7 installers that contain no community script
bundle may use stock scripts only when their source URL and complete package
SHA-256 match an inspected exception in the source catalogue.

When content is missing, Tacitus first uses cached and installed map indexes
to identify the likely pack. An exact map entry takes priority; the same map
in another revision can guide pack selection but cannot satisfy playback.
Otherwise, compatible Command Post registry codes supply category hints, then
the active participant count selects the first category: 1–2
players prefer 1v1, 3–4 prefer 2v2, and 5–8 prefer the large-map pack.
Observers and commentators are excluded; AI players count. FFA and team
games with the same participant count use the same map-capacity hint.
Without a known map match, categories too small for the match are skipped.
Early unversioned packs mixed capacities, so their categories are not excluded
by participant count.

Within each category, verified public links from Command Post's historical
registry are tried first. The kaneswrath.com version lists are fallbacks for
the exact R21–R25 revision, including filenames with underscores or WordPress
numeric duplicate suffixes such as `Map-Pack-1.zip`. The
[source catalogue](map-pack-sources.md) records links, their availability,
exact Command Post version IDs and archive names. Downloading the embedded
public links does not require a Command Post login. Match size is
not guaranteed to equal map capacity, so an unknown map can still require
another candidate if the first pack does not contain it. Missing
historical downloads fail explicitly; Tacitus never substitutes the latest
version. Download support also depends on the available package format:
direct BIG files in ZIPs, ANSI/Unicode solid LZMA NSIS (including R16),
Unicode non-solid DEFLATE NSIS (including R20e), and the Unicode chunked
LZMA NSISBI layout used by R24g are supported.
Other layouts fail without running installers. Finding an older public ZIP
does not by itself establish that its installer layout is supported.

The extractor reads the installer as data and selects the revision's map
archives, scripts and community texture archive from its Patch103 payload,
including the verified `Patch103/ArcadeMapPack` subfolder used by Arcade F03.
Command Post's metadata handles historical archive aliases such as
`R201v1Maps.big` for R20e and `R21g1v1Maps.big` for R21h. The internal map
asset must still match the replay's complete revision-specific path.
The verified Command Post release named `R16 Beta` maps to replay revision
`R16`. Its three installers also contain companion `102plusmaps*A.big`
archives, which supply additional R16 maps. These are extracted with the
main archives and scripts from the same package. The inspected R15 Beta
standard and Predatore installers likewise contain exact `__15` maps. Other beta labels are not
automatically treated as final replay revisions.
The R18f large-map registry link points to an older R18d package. The catalogue
excludes it for R18f and uses the verified R18f ZIP on the same Command Post
CDN. Its map archive matches Command Post's published checksum and contains
the exact `__18f` assets; the older archive is never used as a substitute.
The extractor does not execute installer instructions, plugins, replacement engines or
configuration changes. Extraction has bounds on file counts, offsets, memory,
expanded sizes and output paths. No external extraction program is required.
Solid streams are decoded into a bounded temporary data file, then only
allowlisted content is copied into the cache. The temporary file is removed
on completion, failure, or cancellation. The reader checks dictionary sizes,
stream completion, decoded sizes, file offsets, and both NSIS string encodings.

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

A cache created before companion archives were recognized remains usable
for maps it contains. It cannot suppress a new download for a missing map
until it includes all map archives declared for that source.

Downloads hold an OS file lock on the cache to prevent overlapping writers.
After a crash, the next download removes only marked abandoned staging
folders while holding that lock. Unmarked folders are left alone.

The temporary configuration mounts the resolved replay content ahead of the
installed base-game, language and audio layers. It does not read the user's
top-level patch selection or write the game's SkuDef, Patch103 configuration
or installed packs. Paths to borrowed archives remain in their original
locations. Unsupported map namespaces, absent exact custom maps, unavailable
compatibility values and missing engine/base content produce an explanation
instead of guessing.

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
tacitus-cli cache-pack 123 C:\Downloads\historical-pack.zip --source https://example.org/pack.zip --sha256 <verified-SHA256>
```

`check` and `play --dry-run` inspect without downloading or launching. A
positive report can mean that Play can download missing content.
`prepare` downloads if needed and retains a temporary launch plan for
inspection without starting the game; `--offline` forbids downloads.
`--cache DIR` overrides the cache. `--db` selects the catalogue as usual.

`cache-pack` imports a separately obtained ZIP or supported standalone NSIS
installer for a replay with a revision
suffix. It verifies the supplied SHA-256, exact map asset, compiled MC and
script dependency before publishing the cache. The shareable source URL must
exclude credentials. The supplied package is preserved, and no installer or game
is executed. This supports packs obtained through Command Post's managed
download flow without storing its session credentials in Tacitus.

The original R12d installers also have verified Internet Archive fallback
links. Automatic downloads must match their pinned SHA-256 before extraction.
Tacitus reads their NSIS data records directly and retains each package's own
maps and script dependency. It does not execute the downloaded program.

To audit every catalogue entry without downloads or game simulation:

```powershell
python tools/audit_replay_content.py --game "C:\Games\KW" --output scratch/coverage.json
```

The audit hashes each cached archive once, checks replay hashes, reads actual
BIG indexes and compiled map metadata, validates script provenance, and checks
installed custom-map checksums. `--db`, `--cache` and `--maps` override the
default locations. The [coverage report](replay-content-coverage.md) records
the latest measured result and outstanding exact requirements.

The advanced `--sku FILE` override retains the former manual compatibility
checker/launcher for diagnostics. It does not use automatic preparation and
cannot be combined with `--game`, `--offline` or `prepare`.

## Implementation references and checks

Compiled map layouts follow WrathEd2012's
[MetaDataCommon.xml](https://github.com/Qibbi/WrathEd2012/blob/master/SAGE/Games/Kane%27s%20Wrath/Includes/MetaDataCommon.xml),
[MapMetaData.xml](https://github.com/Qibbi/WrathEd2012/blob/master/SAGE/Games/Kane%27s%20Wrath/Includes/MapMetaData.xml)
and `SAGE.Stream` manifest definitions. RefPack command formats are documented
by [OpenSAGE](https://github.com/OpenSAGE/OpenSAGE/blob/master/src/OpenSage.FileFormats.RefPack/RefPackStream.cs).
The custom-map checksum follows EA's
[CRC implementation](https://github.com/electronicarts/CnC_Generals_Zero_Hour/blob/main/GeneralsMD/Code/GameEngine/Source/Common/crc.cpp)
and `GameClient/MapUtil.cpp`, confirmed against the installed Alien Tower maps.

The [NSISBI project](https://sourceforge.net/projects/nsisbi/), NSIS's
[fileform.h](https://github.com/kichik/nsis/blob/master/Source/exehead/fileform.h), and the
[NSISExtractor format notes](https://github.com/KokerZhou/NSISExtractor/blob/main/docs/nsis-format-notes.md)
inform the bounded installer reader. The historical source is the
[R24 1v1 map pack version list](https://kaneswrath.com/download/r24-1vs1-map-pack/).

Tests cover exact revisions, disabled local packs, offline reuse, concurrent
cached revisions, shared-script ambiguity, corrupt and incomplete caches,
truncated HTTP responses, cancellation during HTTP waits and preparation,
bounded extraction, path containment, settings migration, stale-session
ownership, and the one-action frontend flow. Legacy checker fixtures remain
in `src-tauri/tests/playback.rs`.

The historical-source tests also cover public-link filtering, exact archive
aliases, underscore filenames, the older NSIS file records, bounded DEFLATE
decoding, stream termination, and cancellation. R20e 1v1 and 2v2 packages
from Command Post's public Drive links were fully downloaded, extracted and
used to prepare replays 405 and 443 on 2026-09-19. The 2v2 check exercised
Tacitus's complete network download path; the 1v1 check reused the verified
research download. These checks did not start the game.

R16 checks on 2026-09-19 prepared Tournament Highlands (388), Redzone
Rampage (397), Spacegarden (385), and companion maps Smashed Decision (453),
Forgotten Forest (635), and Tiberian Dunes (499) from Command Post's three
public CDN ZIPs. Tests cover the explicit `R16 Beta` mapping, ANSI solid
LZMA extraction, truncation, trailing data, dictionary bounds, cancellation,
and upgrading a cache that lacks companion maps. No installer or game was
executed during these preparation checks.

On 2026-09-19, the R24g 1v1 ZIP was downloaded from the historical version
list and extracted without executing its installer. Catalogue replay 1153
(`[R24] Abandoned Subway`) prepared again offline and reached active replay
playback with a temporary config outside the game directory. The CLI had
exited while its session remained
available to the game. The test copy was then closed. All 37 game configuration
files matched their pre-test SHA-256 values. This was a brief loading/playback
check, not a full-duration determinism test; further live testing was deferred
at the user's request.
