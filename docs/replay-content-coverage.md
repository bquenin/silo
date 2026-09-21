# Replay content coverage

Checked 2026-09-20 against the 1,228-entry working catalogue and the installed
Kane's Wrath 1.2 game. **1,184 replays (96.4%) have matching content.** Full
coverage has not been reached.

The current recovery target excludes custom maps. **35 replays still need
historical community packs**, down from 58 after recovering all R12d packs.

| Result | Replays |
| --- | ---: |
| Community map path, compiled MC, package scripts and archive hashes verified | 868 |
| Stock map path and active engine's compiled MC verified | 311 |
| Installed custom map path and complete file checksum verified | 5 |
| Exact historical map pack still missing | 35 |
| Exact custom map still missing | 8 |
| Replay file changed after import | 1 |
| Total | 1,228 |

All **120 cached packages** passed archive SHA-256 and BIG-index checks.
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
| R13 paths reused by R13c/R13d | `20`, `21` | 7 | Matching compiled builds, not the original R13 release |
| R20c | `3C` (1v1), `38` (2v2) | 4 | Original R20c maps and their scripts |
| Historical 1.03 maps | `81`, `83`, `86`, `8A`, `97`, `A3` | 7 | TEST117/119/122/128/182 and version 236 map packs |

Command Post's registry, public map-pack storage, website versions and known
archive leads were checked. Its public 1.03 storage retains older patch ZIPs,
but the required historical **map packs** were not found there. R20c managed
Drive objects returned missing-file responses. These results do not prove
that no copy survives elsewhere.

The Mega folder linked by a [2023 R20 release guide](https://cncseries.ru/kw-patch-1-02/)
was also listed. It contains R20e-labelled archives and 4K add-ons, with no
R20c archive listed; it does not supply the missing R20c builds.

| Missing custom map directory | File checksum (hex) | Replays |
| --- | --- | ---: |
| `redzone_rampage_1v1` | `6F11AD75` | 4 |
| `the_kracken` | `0FCC7DC4` | 1 |
| `tiberium_garden_6_way_2012` | `D5C7C73C` | 1 |
| `island_paradise_2021` | `6BDD2F67` | 1 |
| `!!!!!!infinity_island` | `F58A6252` | 1 |

The [public 2,200-map collection](https://kaneswrath.com/download/mega-mappack/)
provided an exact `[standard] tournament highlands` map with checksum
`D83258F0`; its map and companions were recovered. Installed Alien Tower V2.1
and V1.7x maps matched the other three custom-map replays. Similarly named
Redzone and Infinity Island maps from Command Post had different checksums
and were excluded.

The original `Tiberium Gardens III` custom map was recovered from
[MaD_Animal's Tiberium Wars collection](https://steamcommunity.com/sharedfiles/filedetails/?id=1503194353),
using its [public archive share](https://madnetwork.direct.quickconnect.to:5001/sharing/oB6Cylj5U).
Its complete 450,767-byte `.map` file matches the replay's `9C00930B` checksum
and has SHA-256 `d10323815862d8e48e869dfdd7c0faebb7595e2a13da2c532fa2575857911c15`.
The original map, preview and two XML companions were installed; offline
preparation passed. Only those four ZIP members were downloaded and checked,
not the entire Tiberium Wars collection. The similarly named Kane's Wrath
website download in the candidate table below is a different file.

The [missing-content manifest](missing-replay-content.json) lists all **32
distinct missing map requirements affecting 43 replays**, with their exact
asset paths, MC values and engine versions. Eight of those replays require
custom maps, which are outside the current recovery target. It also records 21 original
Command Post release records, including available archive and script MD5s,
to identify surviving copies. Those records are recovery leads, not claims
that their downloads still work. The manifest contains no replay filenames,
catalogue IDs, local paths or session credentials.

Further checks covered every `.map` file in eight public collections, including
maps stored under unrelated names:

| Collection | Map files checked | ZIP/RAR SHA-256 | Additional missing-map matches |
| --- | ---: | --- | ---: |
| [KanesWrath.com mega pack](https://kaneswrath.com/download/mega-mappack/) | 2,309 | `2b349bb410ffae12f7dda1aaeebc8ef79ddc7418f409472adcf23a6a36d38fd6` | 0 |
| [ItzTeeJaay's 2020 collection](https://www.moddb.com/games/cc-kanes-wrath/addons/itzteejaays-kanes-wrath-map-collection) | 305 | `1d3885c1693cba4f752c727adbd128c495bd7f7d41107a6d1c7628cbee8f34a8` | 0 |
| [MaD_Animal KW pack A–L](https://madnetwork.direct.quickconnect.to:5001/sharing/wKvFDDIFR) | 477 | `8d2a79d67b90cb3176be2f81cca7064e17963a94bdcd0186a37c5fd833b8e4ca` | 0 |
| [MaD_Animal KW pack M–Z](https://madnetwork.direct.quickconnect.to:5001/sharing/sPbgOrdoH) | 520 | `3577c7f03c200c39c960d823ca1ca1cd210e3193ff07ddb67889807d495c5459` | 0 |
| [MaD_Animal edited maps](https://madnetwork.direct.quickconnect.to:5001/sharing/yVqRoD5O7) | 49 | `d72632c0d734adccac0a6771aa2ca93de14a60039adaee29885dc24b56b8811d` | 0 |
| [MaD_Animal original maps](https://madnetwork.direct.quickconnect.to:5001/sharing/Il3LwuQWe) | 67 | `15aebce0aa9cbba25d4cdb66fff624c35f5bf36354626dcd3933bc6fa6d3a878` | 0 |
| [MaD_Animal TD, AOD and mission maps](https://madnetwork.direct.quickconnect.to:5001/sharing/gAQlZomSz) | 191 | `c70c112ba673eb0d5c19c46cb5653d7b27fb82f4ccb2914db77a4b73b1486b31` | 0 |
| [PurpleGaga27 KW skirmish collection](https://drive.google.com/file/d/1GiMujALCi4sKu4xW3mTacEyHGSvcPJ78/view) | 1,108 | `10f583eaea6ad78dc441f4ca2c1f34f79351ce69faca4c4b8dd96068e3ba0c80` | 0 |

Published MD5s also matched for ItzTeeJaay's collection and
[MaD_Animal's 67 original maps](https://www.moddb.com/games/cc-kanes-wrath/addons/cc-kanes-wrath-67-maps-made-by-mad-animal).
The MaD_Animal shares
are linked from the author's [Kane's Wrath guide](https://steamcommunity.com/sharedfiles/filedetails/?id=1503234098);
PurpleGaga27's share is linked in the author's [collection post](https://forums.cncnz.com/topic/21722-the-ultimate-skirmish-map-packs-for-cc-the-ultimate-collection-and-the-first-decade/).
Counts reflect actual `.map` entries, even where archive titles say 517 or
1,035. These collections overlap, so the 5,026 entries above are not a count
of unique maps. The mega pack
had already supplied the matching Tournament Highlands map noted above.
Its `tiberium garden iii 2012 1.0` folder contains companion images and
strings, but no `.map` file.

All 26 map ZIPs returned by the related Command Post custom-map searches were
also downloaded and checksum-checked, with no additional matches.

Additional candidates also failed the replay checksum check:

| Candidate | Available file checksum | Required checksum |
| --- | --- | --- |
| [Tiberium Gardens III](https://kaneswrath.com/kw-maps/tiberium-gardens-iii/) | `72572686` | `9C00930B` |
| [Redzone Rampage 1v1 from Command Post](https://maps2.s3.amazonaws.com/redzone_rampage_1vs1/redzone_rampage_1vs1.zip) | `68FBA0E8` | `6F11AD75` |
| [Redzone Rampage Tournament Edition from ModDB](https://www.moddb.com/addons/redzone-rampage-1vs1) | `1ED26F0C` | `6F11AD75` |
| [Infinity Island](https://kaneswrath.com/kw-maps/infinity-island/) | `D089CE4D` | `F58A6252` |

The missing Infinity Island replay records the display name
`Infinity Isle [InProgress]`; a published final version is insufficient.
The missing Redzone map records `Tournament redzone` as its display name,
but still requires the `redzone_rampage_1v1` directory and checksum above.

The changed catalogue entry points to `Last Replay.KWReplay`, which was
overwritten after import. Its stock map exists, but the original replay must
be recovered or the current file imported as a separate entry before that
replay can pass preparation. No matching local backup was found.

## Recovery boundary

The renewed Wayback search recovered all three R12d installers from
Shatabrick's original download URLs. The complete captures date from August
2025; older 2019 captures were only about 1 MB and were not used. All three
downloaded payload SHA-1 hashes match their CDX records. Their 81 compiled map
assets have MC `1A`, and all 23 R12d requirements match the full asset path and
MC. The 1v1 installer has different scripts from the 2v2 and large-map
installers; Silo keeps each package's own dependency.

The [source catalogue](map-pack-sources.md#r12d-recovered-from-wayback) records
the three downloadable archive URLs, original URLs, capture dates, package
SHA-256 values and extracted BIG hashes. Automatic preparation from a fresh
large-pack download passed, along with offline preparation from the 1v1,
2v2 and unversioned companion maps. The full metadata audit confirmed the
23 additional covered replays without running the game.

Wayback's CDX index was available during this renewed search even though its
separate availability API still returned HTTP 429. Successful index queries
covered the old `app-direct.net` domain, Command Post and its public storage
hosts, Shatabrick, GameReplays download URLs and release threads, the author's
old Dropbox paths, the missing R20c Drive URLs and the Kane's Wrath website.
The checked indexes supplied no remaining exact historical pack downloads.
Archived GameReplays pages preserved early multiplayer R3/R4 Dropbox
filenames, but the saved captures of those download URLs are 404 responses.
These filenames are leads, not verified matches for MC `6` or `7`.

Command Post's public object-version listings were also checked: 81 objects
under `files/1.02+/` and 306 under `files/1.03/`, with no older object versions
or deletion records exposed. The visible 1.03 map packs do not include the
required TEST117/119/122/128/182 or version236 builds. Internet Archive item
searches and the checked Arquivo.pt text searches did not yield those files.
Common Crawl's checked 2018 and 2023 page indexes had no matching page records;
another query timed out, so Common Crawl is not exhaustively ruled out.

The remaining R13c/R13d Command Post records still point to unavailable
`app-direct.net` downloads. Those URLs returned HTTP 403 on the live recheck;
the earlier catalogue checks recorded HTTP 404. Command Post's managed R20c
1v1 and 2v2 archive requests returned HTTP 404. None of these findings proves
that no copy survives elsewhere.

The two R20-era 4K add-ons in the public Mega folder were inspected as data.
Their installer file tables contain 119 and 156 records respectively, with
no `.map` files, map directories or BIG archives. They cannot replace the
missing map packs. Neither installer was executed.

Further recovery needs an original installer/ZIP, an installed copy of the
required map BIG and matching scripts, or a surviving download source for
one of the remaining historical requirements in the missing-content manifest.
A new candidate can be checked against the saved asset paths, MC values and original Command
Post archive hashes before installation. Renaming a later release or changing
the recorded compatibility value would not recover the required content.

## Recovered coverage and provenance

Recovered packages include early R2–R11 releases, R12 and R12d category packs, R15
standard and Predatore bundles, exact R18d/R18e builds, R19e/f/g/j, R20a,
R21b assets distributed under R21c labels, and Arcade F03/R21h. R23f and R24
website fallback downloads now recognize WordPress's numeric filename suffix.
Each candidate still needs the exact full asset path and compiled MC value.

Command Post links remain first choice. The nine recovered R19 packages were
downloaded through its managed flow; transfer checksums and original map BIG
MD5 records agreed. They work from the local cache. Their restricted links
are **not** public automatic downloads on a fresh installation; separately
obtained ZIPs can be imported with `silo-cli cache-pack`.

The [source catalogue](map-pack-sources.md) records shareable links, original
version IDs, compatibility codes, verified archive aliases and dependency
exceptions. No session credentials or downloaded map binaries are committed.
The [playback documentation](replay-launcher.md) describes the resolution
rules, import command and validation limits.

## Reproduce

```powershell
python tools/audit_replay_content.py --game "C:\Games\KW" --output scratch/coverage.json
silo-cli prepare <replay-id> --offline --json
```

The JSON report includes every replay's exact asset, MC, engine version and
matching cached package, or the unresolved requirement. Keep this output
local: it includes catalogue paths. The CLI then verifies an individual
launch plan without starting the game.
