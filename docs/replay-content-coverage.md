# Replay content coverage

Checked 2026-09-20 against the 1,228-entry working catalogue and the installed
Kane's Wrath 1.2 game. **1,160 replays (94.5%) have matching content.** Full
coverage has not been reached.

| Result | Replays |
| --- | ---: |
| Community map path, compiled MC, package scripts and archive hashes verified | 845 |
| Stock map path and active engine's compiled MC verified | 311 |
| Installed custom map path and complete file checksum verified | 4 |
| Exact historical map pack still missing | 58 |
| Exact custom map still missing | 9 |
| Replay file changed after import | 1 |
| Total | 1,228 |

All **117 cached packages** passed archive SHA-256 and BIG-index checks.
Every unchanged replay was checked against its imported SHA-256. The audit
uses the full catalogue; SQL `LIKE` patterns containing unescaped underscores
are not used to decide which replays count.

These are content and preparation checks. Games were not simulated through
their duration. Stock metadata is matched against the installed engine's
content chain; stock game files are not compared with an external publisher
hash manifest. Installed community files outside the verified cache are not
counted by the batch audit.

## Exact requirements still missing

| Family | Recorded MC (hex) | Replays | Requirement |
| --- | --- | ---: | --- |
| Early unversioned 1.02+ maps | `6`, `7` | 17 | Original multiplayer packs; recovered R3/R4 1v1 packs do not contain these maps, and R5/R6 multiplayer packs have MC `9` |
| R12d and its unversioned companion maps | `1A` | 23 | Exact original R12d packs |
| R13 paths reused by R13c/R13d | `20`, `21` | 7 | Matching compiled builds, not the original R13 release |
| R20c | `3C` (1v1), `38` (2v2) | 4 | Original R20c maps and their scripts |
| Historical 1.03 maps | `81`, `83`, `86`, `8A`, `97`, `A3` | 7 | TEST117/119/122/128/182 and version 236 map packs |

Command Post's registry, public map-pack storage, website versions and known
archive leads were checked. Its public 1.03 storage retains older patch ZIPs,
but the required historical **map packs** were not found there. R20c managed
Drive objects returned missing-file responses. These results do not prove
that no copy survives elsewhere.

| Missing custom map directory | File checksum (hex) | Replays |
| --- | --- | ---: |
| `redzone_rampage_1v1` | `6F11AD75` | 4 |
| `the_kracken` | `0FCC7DC4` | 1 |
| `tiberium_garden_6_way_2012` | `D5C7C73C` | 1 |
| `tiberium gardens iii` | `9C00930B` | 1 |
| `island_paradise_2021` | `6BDD2F67` | 1 |
| `!!!!!!infinity_island` | `F58A6252` | 1 |

The [public 2,200-map collection](https://kaneswrath.com/download/mega-mappack/)
provided an exact `[standard] tournament highlands` map with checksum
`D83258F0`; its map and companions were recovered. Installed Alien Tower V2.1
and V1.7x maps matched the other three custom-map replays. Similarly named
Redzone and Infinity Island maps from Command Post had different checksums
and were excluded.

The changed catalogue entry points to `Last Replay.KWReplay`, which was
overwritten after import. Its stock map exists, but the original replay must
be recovered or the current file imported as a separate entry before that
replay can pass preparation. No matching local backup was found.

## Recovered coverage and provenance

Recovered packages include early R2–R11 releases, R12 category packs, R15
standard and Predatore bundles, exact R18d/R18e builds, R19e/f/g/j, R20a,
R21b assets distributed under R21c labels, and Arcade F03/R21h. R23f and R24
website fallback downloads now recognize WordPress's numeric filename suffix.
Each candidate still needs the exact full asset path and compiled MC value.

Command Post links remain first choice. The nine recovered R19 packages were
downloaded through its managed flow; transfer checksums and original map BIG
MD5 records agreed. They work from the local cache. Their restricted links
are **not** public automatic downloads on a fresh installation; separately
obtained ZIPs can be imported with `tacitus-cli cache-pack`.

The [source catalogue](map-pack-sources.md) records shareable links, original
version IDs, compatibility codes, verified archive aliases and dependency
exceptions. No session credentials or downloaded map binaries are committed.
The [playback documentation](replay-launcher.md) describes the resolution
rules, import command and validation limits.

## Reproduce

```powershell
python tools/audit_replay_content.py --game "C:\Games\KW" --output scratch/coverage.json
tacitus-cli prepare <replay-id> --offline --json
```

The JSON report includes every replay's exact asset, MC, engine version and
matching cached package, or the unresolved requirement. Keep this output
local: it includes catalogue paths. The CLI then verifies an individual
launch plan without starting the game.
