# Historical map pack sources

Checked 2026-09-20. **462 distinct source links**, including **382 public ZIP downloads whose headers were verified**. The catalogue records 403 pack versions and verified additions.

A header check confirms that a public URL returned ZIP bytes; it does not validate the whole archive or prove replay compatibility. Unavailable links remain in the catalogue as research leads. Login pages and transient failures are not proof that a pack no longer exists.

[Machine-readable catalogue](../src-tauri/resources/map-pack-sources.json). This file is generated with `python tools/render_map_sources.py`.

## Source order in Tacitus

Tacitus checks installed and cached content first. For each likely map category, it tries verified public links from Command Post before the exact-version list on kaneswrath.com. A failed download or extraction advances to the fallback. It never accepts a candidate only when its full internal map path and compiled compatibility value match the replay. Provider labels can differ from internal suffixes. Unverified test releases and pack families remain excluded.

The catalogue embeds only shareable URLs. Automatic candidates must be public. Some R19 packs were recovered through Command Post managed downloads and verified against its archive checksums; their restricted links remain excluded from automatic downloads. Supplied ZIPs can be imported using `tacitus-cli cache-pack`. Credentials and session-bound download URLs are excluded.

Metadata provenance: [Command Post public metadata ZIP](https://corefiles1.s3.eu-central-1.amazonaws.com/metadata.zip) and the Command Post `fetch_files.php` registry, queried by exact `metapack_name` and `meta_version_id`. The version identifier also supplies the map archive name: for example R20e uses `R201v1Maps.big`, while R21h uses `R21g1v1Maps.big`. The `compatibility_code` selects candidates, then Tacitus checks the actual compiled MapMetaData value against replay `MC`. Original R2–R7 packages without separate scripts use stock scripts only with an inspected, SHA-256-pinned exception. [Measured catalogue coverage](replay-content-coverage.md) separates verified content from remaining missing requirements.

Installer support covers ZIPs containing BIG files, ANSI/Unicode solid LZMA NSIS, Unicode non-solid DEFLATE NSIS, and Unicode chunked LZMA NSISBI. Other installer layouts fail without being executed. Not every historical pack listed here has been fully extracted or replay-tested.

## Recovered original packs

R15 standard and Predatore bundles were recovered from Command Post public storage; their beta labels contain the exact `__15` assets. R20 registry records contain `__20a`, and R21c packages contain `__21b`; these mappings are recorded explicitly. R18d and R18e both use `__18` paths, distinguished by MC values `2B` and `2C`. Early `1.02+ edition` maps are likewise distinguished by compiled MC, never by display name alone. The website Arcade F03 source contains exact `__r21h` assets with MC `5` and its own scripts.

## R16

Command Post labels this release **R16 Beta**. Its actual map assets use the `__16` suffix recorded by R16 replays. The three verified standard pack records explicitly map to R16; other beta labels remain excluded until their assets are verified. The installers use an ANSI NSIS header and solid LZMA compression.

Each installer contains its main `102plusmaps*.big` archive, a companion `102plusmaps*A.big` archive, and matching scripts. Tacitus includes both map archives from that exact package. This covers companion maps such as Smashed Decision, Forgotten Forest and Tiberian Dunes. A previously cached pack missing its companion archive no longer suppresses the download of a missing map.

| Pack | Command Post link | Public ZIP size |
| --- | --- | ---: |
| 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps_R16.zip) | 321,025,614 bytes |
| 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps2_R16.zip) | 170,328,550 bytes |
| 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps3_R16.zip) | 169,458,219 bytes |

## R18f 4v4

Command Post registry link 809, registered as R18f, points to an R18d ZIP whose map assets use `__18`. Tacitus excludes that link for R18f and uses the [correct R18f ZIP on the same Command Post CDN](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18f/KWCommunityPatch102PlusMaps3_R18f.zip). The corrected path follows the installer filename in Command Post version `61f8aa314ec59`.

The 251,672,113-byte ZIP was fully downloaded and extracted. Its `102plusmaps3_18.big` contains 40 exact `__18f` map assets and matches the MD5 recorded in Command Post metadata: `1f63b5e1ac0a3859df1a2c2d222a1117`. Matching scripts come from the same installer. Tacitus successfully prepared `4_vs_4-367542da1019e4a6.KWReplay` (Tiberian Gardens VIII) with these assets. The pack covers all seven R18f large-map replays in the checked catalogue. These checks prepare content without starting the game.

ZIP SHA-256: `7cff6da9368c02c0d85ff497b7c850f7c5444d879fd355a2bd53d23d013985b8`.

## R20e

| Pack | Command Post link | Public ZIP size |
| --- | --- | ---: |
| 1v1 | [Download](https://drive.google.com/file/d/1gXufNs20MMYavQlQ-SLZq6lzkoZD8mfg/view?usp=sharing) | 961,792,588 bytes |
| 2v2 | [Download](https://drive.google.com/file/d/1gfGhhzRLlZZ4DoW0kq7r2Nt33BMrlmac/view?usp=drive_link) | 573,634,650 bytes |
| 4v4 | [Download](https://drive.google.com/file/d/1h-0Cj71wA1H4g3EgH0MCAz-X8ZA93IOV/view?usp=drive_link) | 575,464,421 bytes |

The 1v1 and 2v2 packages were fully downloaded and extracted as data. Tacitus prepared catalogue replays 405 (Tournament Highlands) and 443 (Redzone Rampage) using their exact R20e assets and matching scripts. The packs contain 69 and 49 map assets respectively. These checks prepared launch configurations without starting the game.

The 1v1 ZIP SHA-256 is `7c0ab9ddfd58cd5b44b134a19eea01ff6b3fa90d2e232dab033d46edd1a6147b`.

## Command Post links

| Revision | Pack | Link | Check |
| --- | --- | --- | --- |
| R22j | Legacy | [Download](https://drive.google.com/file/d/1mXQKjXGffQZQYw7RZaAcGcawSn37rAaB/view?usp=drive_link) | ZIP header verified |
| R23 | Legacy | [Download](https://drive.google.com/file/d/1yZuHaD60sPkLlW0fdE6OGpzMUj_Fy_1J/view?usp=sharing) | HTTP 404 |
| R23a | Legacy | [Download](https://drive.google.com/file/d/1rk38oiukzcVAGI9Zn-Z4_GVhc2yvCkC_/view?usp=drive_link) | ZIP header verified |
| R23b | Legacy | [Download](https://drive.google.com/file/d/1-6CrJP5FCUj7Rxbse5RCRSG3AP9Vh7Cn/view?usp=drive_link) | ZIP header verified |
| R23c (test) | Legacy | [Download](https://drive.google.com/file/d/189CX9Y2cPWMsWScfo1pqpVa9pO7I9oJv/view?usp=drive_link) | ZIP header verified |
| R23d | Legacy | [Download](https://drive.google.com/file/d/1PtLszq1XTkpNIJp4U-QDldUhFTs5TAKz/view?usp=drive_link) | ZIP header verified |
| R23h | Legacy | [Download](https://drive.google.com/file/d/1pZs-j50HPR8Z3xA1RY4yTZrxEJn7dMuV/view?usp=drive_link) | ZIP header verified |
| R23x | Legacy | [Download](https://drive.google.com/file/d/1nxjnmdAhCENBN6yO93iMwq-gmMpjACDA/view?usp=drive_link) | ZIP header verified |
| R23z | Legacy | [Download](https://drive.google.com/file/d/1FCNdQIM6B8nQdF4ydPma7WXHovN8237r/view?usp=drive_link) | ZIP header verified |
| R24b | Legacy | [Download](https://drive.google.com/file/d/1ZFBd6Zg--K8V7rJOaWmdk4LSVZGzux6b/view?usp=drive_link) | HTTP 404 |
| R24c | Legacy | [Download](https://drive.google.com/file/d/11jWNGNRVBsiTydQS7tDbx9bxd2peLAks/view?usp=drive_link) | HTTP 404 |
| R24d (test) | Legacy | [Download](https://drive.google.com/file/d/1qo4a4fqEaXhJyWkmf7K_qBx1E8wQmIBL/view?usp=drive_link) | HTTP 404 |
| R24e (test) | Legacy | [Download](https://drive.google.com/file/d/1AZeTKnl_qN4hoFADGAqwq_7S6hwRYJB0/view?usp=drive_link) | HTTP 404 |
| R24f | Legacy | [Download](https://drive.google.com/file/d/1A0wYlDZQfU4NhNsKu6j6WKOu3e4UFfj_/view?usp=drive_link) | HTTP 404 |
| R24g | Legacy | [Download](https://drive.google.com/file/d/1Dz2Rzz5FPobkrrRTy7LCBg7gCol4bdEz/view?usp=drive_link) | HTTP 404 |
| R24h | Legacy | [Download](https://drive.google.com/file/d/1JtGiJBk55Yz_KgXNJMekwJKOhfparXk3/view?usp=drive_link) | ZIP header verified |
| R24i | Legacy | [Download](https://drive.google.com/file/d/1DXZFjXTifZmsc0UyFq4s99XqhFk7iuxM/view?usp=drive_link) | ZIP header verified |
| R24j | Legacy | [Download](https://drive.google.com/file/d/1R5oX2-ubN5q7BFwtHb4LvFvwTYMicTw3/view?usp=drive_link) | ZIP header verified |
| R24k | Legacy | [Download](https://drive.google.com/file/d/1kQrTId-7VpYyNY1Vm8qnlS584oXS6NrT/view?usp=drive_link) | ZIP header verified |
| R24l (test) | Legacy | [Download](https://drive.google.com/file/d/1g1JZx8yuYNBkXYeXuvDFOa5KY42Jbtfq/view?usp=drive_link) | HTTP 404 |
| R24m | Legacy | [Download](https://drive.google.com/file/d/12sP5TJ1BAXyFI3_pJNObBFT8nfjZTlrz/view?usp=drive_link) | ZIP header verified |
| R24o | Legacy | [Download](https://drive.google.com/file/d/1cufr9IMFBdK9_UlSyBp82v4eMCUUKzE1/view?usp=drive_link) | ZIP header verified |
| R24p | Legacy | [Download](https://drive.google.com/file/d/1tdYX1JwFjrNTL15NYKjNwNwfmDTQvvXG/view?usp=drive_link) | ZIP header verified |
| R24q | Legacy | [Download](https://drive.google.com/file/d/1sFbd3gNhVmwEH_MB8_IyfUFyo20AVeY4/view?usp=drive_link) | ZIP header verified |
| R2 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R2/KWCommunityPatch102PlusMaps_R2.zip) | ZIP header verified |
| R3 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R3/KWCommunityPatch102PlusMaps_R3.zip) | ZIP header verified |
| R4 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R4/KWCommunityPatch102PlusMaps_R4.zip) | ZIP header verified |
| R5 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R5/KWCommunityPatch102PlusMaps_R5.zip) | ZIP header verified |
| R6 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R6/KWCommunityPatch102PlusMaps_R6.zip) | ZIP header verified |
| R7 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R7/KWCommunityPatch102PlusMaps_R7.zip) | ZIP header verified |
| R8 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R8/KWCommunityPatch102PlusMaps_R8.zip) | ZIP header verified |
| R9 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R9/KWCommunityPatch102PlusMaps_R9.zip) | ZIP header verified |
| R10 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R10/KWCommunityPatch102PlusMaps_R10C.zip) | ZIP header verified |
| R11 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R11/KWCommunityPatch102PlusMaps_R11.zip) | ZIP header verified |
| R12 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12/KWCommunityPatch102PlusMaps_R12.zip) | ZIP header verified |
| R12b | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12b/KWCommunityPatch102PlusMaps_R12b.zip) | ZIP header verified |
| R12c | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12c/KWCommunityPatch102PlusMaps_R12c.zip) | ZIP header verified |
| R12d | 1v1 | [Download](http://app-direct.net/production/public/files/1.02+/R12d/KWCommunityPatch102PlusMaps_R12d.zip) | HTTP 404 |
| R13 | 1v1 | [Download](http://app-direct.net/production/public/files/1.02+/R13/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13b | 1v1 | [Download](http://app-direct.net/production/public/files/1.02+/R13b/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13c | 1v1 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13d | 1v1 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R14 | 1v1 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14.zip) | HTTP 404 |
| R14 Beta 2 (test) | 1v1 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14_TEST.zip) | HTTP 404 |
| R15 Beta | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMaps_R15.zip) | ZIP header verified |
| R16 Beta | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps_R16.zip) | ZIP header verified |
| R18 | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18/KWCommunityPatch102PlusMaps_R18.zip) | ZIP header verified |
| R18d | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18d/KWCommunityPatch102PlusMaps_R18d.zip) | ZIP header verified |
| R18e | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18e/KWCommunityPatch102PlusMaps_R18e.zip) | ZIP header verified |
| R18f | 1v1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18f/KWCommunityPatch102PlusMaps_R18f.zip) | ZIP header verified |
| R18f | 1v1 | [Download](https://drive.google.com/file/d/1pmKt1idq8sKgxhv_52CmSdkadk1H25lg/view?usp=sharing) | HTTP 404 |
| R19b (test) | 1v1 | [Download](https://drive.google.com/file/d/1odkeOFRgMes4AVqI9pavitzR5VoI2fIc/view?usp=sharing) | No ZIP returned |
| R19c (test) | 1v1 | [Download](https://drive.google.com/file/d/18Es2UpZjRnbxDB2h-7h9FihuKyQVqj2c/view?usp=sharing) | HTTP 404 |
| R19d (test) | 1v1 | [Download](https://drive.google.com/file/d/1rv3Rz7cx_jKOtF2aR6ayOp54MD2Qsenh/view?usp=sharing) | No ZIP returned |
| R19e | 1v1 | [Download](https://drive.google.com/file/d/1SgQYqZJNkPXUT3GT9Zis0yb_w8NKj46t/view?usp=sharing) | Full managed download verified; login required |
| R19f | 1v1 | [Download](https://drive.google.com/file/d/1252O2D6zc7s1If7uHXAFN8V3PhRSpoXK/view?usp=sharing) | Full managed download verified; login required |
| R19g | 1v1 | [Download](https://drive.google.com/file/d/1nF1XPm1YT9L8kJ7ZyYxCpTrQExBGm7_B/view?usp=sharing) | Full managed download verified; login required |
| R19h | 1v1 | [Download](https://mega.nz/file/hf0ijBhA#Y4_XMwwxfvSV543T_BjUWN5OHEIRkMkvBN3OxNBNXOo) | No ZIP returned |
| R19i (test) | 1v1 | [Download](https://drive.google.com/file/d/1ZGiD7op1l2uRyyVtDaboLwrj6h_locxi/view?usp=share_link) | HTTP 404 |
| R19j | 1v1 | [Download](https://drive.google.com/file/d/1Z0ymHPsGdzrPmXmEKh2zjZqVgF11UF3F/view?usp=sharing) | Full managed download verified; login required |
| R20 | 1v1 | [Download](https://drive.google.com/file/d/1lMGI4WpL1OuLqclMofAm4MBmhKjN1R_O/view?usp=sharing) | ZIP header verified |
| R20b | 1v1 | [Download](https://drive.google.com/file/d/1yFUohrA-sJ8TjVX3qj2qwYGTCe5sVuuo/view?usp=sharing) | ZIP header verified |
| R20c | 1v1 | [Download](https://drive.google.com/file/d/1b3U0_coU00xUkly6oJBrhVzGNk6YjweW/view?usp=sharing) | HTTP 404 |
| R20d | 1v1 | [Download](https://drive.google.com/file/d/13FIdWEqWO0FZUxDODgd-t6Llix7BwcFw/view?usp=sharing) | ZIP header verified |
| R20e | 1v1 | [Download](https://drive.google.com/file/d/1gXufNs20MMYavQlQ-SLZq6lzkoZD8mfg/view?usp=sharing) | ZIP header verified |
| R21 | 1v1 | [Download](https://drive.google.com/file/d/1FGEVrdlzkY5ppjUClLU0a6ka8jkWZBIZ/view?usp=share_link) | HTTP 404 |
| R21b | 1v1 | [Download](https://drive.google.com/file/d/1t6F7b1oU9EhfpnIr8GUGaWM07abQnMHb/view?usp=drive_link) | ZIP header verified |
| R21c | 1v1 | [Download](https://drive.google.com/file/d/1lCgrcM6EF7U1rTgAajc1e-6d1tEDUzX-/view?usp=drive_link) | HTTP 404 |
| R21c | 1v1 | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13890) | ZIP header verified |
| R21d | 1v1 | [Download](https://drive.google.com/file/d/1SKGzPzpz3J4J75-ftsdormejQ-6TYl2k/view?usp=drive_link) | ZIP header verified |
| R21e | 1v1 | [Download](https://drive.google.com/file/d/1P6nyRxV5rYmjLn9Jdao5WGLVyyNvjcfv/view?usp=drive_link) | ZIP header verified |
| R21f (Hotfix) | 1v1 | [Download](https://drive.google.com/file/d/1aRKFjzcxrnnIkKIEvJ8JrMMy9ywq4cbr/view?usp=drive_link) | HTTP 404 |
| R21g | 1v1 | [Download](https://drive.google.com/file/d/1WXkgDEAL24NgVJfwr972suW_Mp6XYN--/view?usp=drive_link) | ZIP header verified |
| R21h | 1v1 | [Download](https://drive.google.com/file/d/1RamYnKr_sKtWrFbZCg16vHGC6Xg4TB-v/view?usp=drive_link) | ZIP header verified |
| R21i | 1v1 | [Download](https://drive.google.com/drive/folders/1-l81GjEUE0w342zjULmeS01DURxgIJF5?usp=drive_link) | No ZIP returned |
| R21j | 1v1 | [Download](https://drive.google.com/file/d/19SJcTCZzJlUB1iyYhHIqrff1eAuj0yHh/view?usp=drive_link) | ZIP header verified |
| R22 | 1v1 | [Download](https://drive.google.com/file/d/1aWbJ3qmBSdcBlRXWuvT7bNs5Y94aX4TO/view?usp=drive_link) | HTTP 404 |
| R22b | 1v1 | [Download](https://drive.google.com/file/d/12SfuF8swOLI-9TtOW1w5yqyZs0VpZZlv/view?usp=drive_link) | ZIP header verified |
| R22c | 1v1 | [Download](https://drive.google.com/file/d/1v5pdz5GOnK5nXX4WlP7GaDqPchnWPfDN/view?usp=drive_link) | ZIP header verified |
| R22d | 1v1 | [Download](https://drive.google.com/file/d/16NfmoHnIhFKS1aO0mSSdiWbdpQtCbIgL/view?usp=drive_link) | ZIP header verified |
| R22e | 1v1 | [Download](https://drive.google.com/file/d/18ddlX_6N5I_Jbc7gwNLBZIPSGOZg2D94/view?usp=drive_link) | ZIP header verified |
| R22f | 1v1 | [Download](https://drive.google.com/file/d/1j-70T7nnNu1tISqwktFfSS5s-0NnHB3G/view?usp=drive_link) | ZIP header verified |
| R22g | 1v1 | [Download](https://drive.google.com/file/d/1TPEY89LMaOrjQn8B488I7B7SVukt3j5L/view?usp=drive_link) | ZIP header verified |
| R22h | 1v1 | [Download](https://drive.google.com/file/d/1MUpXpGXcBaOiIYPc7GxFN39EWQSWCzvs/view?usp=drive_link) | ZIP header verified |
| R22j | 1v1 | [Download](https://drive.google.com/file/d/1MVMOrdf9qfHBXLMvJm26o-Ankdy5wKNX/view?usp=drive_link) | ZIP header verified |
| R23 | 1v1 | [Download](https://drive.google.com/file/d/1qHwbN8GotuJV3fZvA3dzHn0JN7cF3eGj/view?usp=drive_link) | ZIP header verified |
| R23a | 1v1 | [Download](https://drive.google.com/file/d/1IGKXx0TttdhJnJ4RJy5S_p_OcRtdGWRU/view?usp=drive_link) | ZIP header verified |
| R23b | 1v1 | [Download](https://drive.google.com/file/d/1Wnr69nh4Hzqow5uQNFuihoIrS-gnXSKV/view?usp=drive_link) | ZIP header verified |
| R23c (test) | 1v1 | [Download](https://drive.google.com/file/d/1xn0T1HzBrwSQ5tH4W2YWGhF_60CqBvcE/view?usp=drive_link) | ZIP header verified |
| R23d | 1v1 | [Download](https://drive.google.com/file/d/1gjCINVBcWL_Jj7Af4boK5Ta1lMbILnbb/view?usp=drive_link) | ZIP header verified |
| R23e | 1v1 | [Download](https://drive.google.com/file/d/14XBpWvPxoD9LWpXKLMpJmEmr_jlhGZ6x/view?usp=drive_link) | ZIP header verified |
| R23f | 1v1 | [Download](https://drive.google.com/file/d/17IE1OH3xjeKmPTTbzguZcRPTjfbK-wZf/view?usp=drive_link) | ZIP header verified |
| R23h | 1v1 | [Download](https://drive.google.com/file/d/1oVT7KOluRxGtvT6obVvb16n28sIdHPvn/view?usp=drive_link) | ZIP header verified |
| R23x | 1v1 | [Download](https://drive.google.com/file/d/1D7unwclIE2Kx82XuJS8tkdS-WUBYCtqg/view?usp=drive_link) | ZIP header verified |
| R23z | 1v1 | [Download](https://drive.google.com/file/d/1DAfBPVDs4GdJYHADzvhmg4e0bRiAoatV/view?usp=drive_link) | ZIP header verified |
| R24d (test) | 1v1 | [Download](https://drive.google.com/file/d/1kD1NaeS0CPNNpnO3Yk7Onkbfl3NcrypX/view?usp=drive_link) | HTTP 404 |
| R24e (test) | 1v1 | [Download](https://drive.google.com/file/d/1amgpPaitWkUu9ajhuur2QAMzST_9jwAN/view?usp=drive_link) | HTTP 404 |
| R24f | 1v1 | [Download](https://drive.google.com/file/d/1SEe5xBoirrpSiGxFz8iqoqg6UEco-83f/view?usp=drive_link) | HTTP 404 |
| R24g | 1v1 | [Download](https://drive.google.com/file/d/1pq-9VeGmFBbw6zhQpOr4iesrUAqB2Z0y/view?usp=drive_link) | HTTP 404 |
| R24h | 1v1 | [Download](https://drive.google.com/file/d/1dR0wVqWKb8fAUD1Z-HnObGnodlKttzYE/view?usp=drive_link) | ZIP header verified |
| R24i | 1v1 | [Download](https://drive.google.com/file/d/1UNuj2zk5ARF72hR2RJvWzv0ZAtfpn-qL/view?usp=drive_link) | ZIP header verified |
| R24j | 1v1 | [Download](https://drive.google.com/file/d/1nnEPn9waKL03wER2eP7Z3pFVyvEKnYXH/view?usp=drive_link) | ZIP header verified |
| R24k | 1v1 | [Download](https://drive.google.com/file/d/14AEyq9v4_l9DdbLXGgAVCKBeQpz_Kz1q/view?usp=drive_link) | ZIP header verified |
| R24l (test) | 1v1 | [Download](https://drive.google.com/file/d/1OltczRlFqSmN5dMEvnNkb25yqnCtEuiS/view?usp=drive_link) | ZIP header verified |
| R24m | 1v1 | [Download](https://drive.google.com/file/d/1H-1WZhDd4TrX_J2HeTd_KqJ_YpeLLiS3/view?usp=drive_link) | ZIP header verified |
| R24o | 1v1 | [Download](https://drive.google.com/file/d/1wMJYmo5i35oZHtJOYIZ5H8tC3Ik25RAJ/view?usp=drive_link) | ZIP header verified |
| R24p | 1v1 | [Download](https://drive.google.com/file/d/1yCHGH3tbKx_lKcwoHAN6RPWPNvmTuH5H/view?usp=drive_link) | ZIP header verified |
| R24q | 1v1 | [Download](https://drive.google.com/file/d/1g6QV65r4j_VASp-ol7xAEqSoqNWGTDDp/view?usp=drive_link) | ZIP header verified |
| R24u | 1v1 | [Download](https://drive.google.com/file/d/1TbhB64OKrv0EsF1pmQaZa7V0UproL3qv/view?usp=drive_link) | ZIP header verified |
| R25 | All-in-one | [Download](https://drive.google.com/file/d/1O1K3ewRvRkbeP9we-g2GaxOUf_6HvRRt/view?usp=drive_link) | HTTP 404 |
| R25b | All-in-one | [Download](https://drive.google.com/file/d/1kPgl1UDHpz1tvWvPwhPzZVNK2F8pjHYP/view?usp=drive_link) | HTTP 404 |
| R25d | All-in-one | [Download](https://drive.google.com/file/d/1U9wyRUjwXqMB5drewMLIR268mKvr8MhQ/view?usp=drive_link) | ZIP header verified |
| R25e | All-in-one | [Download](https://drive.google.com/file/d/18pOWrX0v-cpd4VPowDmgjD7ANLkdMbKK/view?usp=drive_link) | ZIP header verified |
| R25g | All-in-one | [Download](https://drive.google.com/file/d/1AAjA583BUmpihV8w0iSSL2uMjSnOa9CL/view?usp=drive_link) | ZIP header verified |
| R25h | All-in-one | [Download](https://drive.google.com/file/d/18hRZNLQLW0wNDGSyOt0SMstnIa7cKUep/view?usp=drive_link) | ZIP header verified |
| R25i | All-in-one | [Download](https://drive.google.com/file/d/127NuCSHqUbUNIobFzUzo5i7S9mNGmf63/view?usp=drive_link) | ZIP header verified |
| R5 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R5/KWCommunityPatch102PlusMaps2_R5.zip) | ZIP header verified |
| R6 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R6/KWCommunityPatch102PlusMaps2_R6.zip) | ZIP header verified |
| R7 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R7/KWCommunityPatch102PlusMaps2_R7.zip) | ZIP header verified |
| R8 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R8/KWCommunityPatch102PlusMaps2_R8.zip) | ZIP header verified |
| R9 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R9/KWCommunityPatch102PlusMaps2_R9.zip) | ZIP header verified |
| R10 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R10/KWCommunityPatch102PlusMaps2_R10C.zip) | ZIP header verified |
| R11 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R11/KWCommunityPatch102PlusMaps2_R11.zip) | ZIP header verified |
| R12 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12/KWCommunityPatch102PlusMaps2_R12.zip) | ZIP header verified |
| R12b | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12b/KWCommunityPatch102PlusMaps2_R12b.zip) | ZIP header verified |
| R12c | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12c/KWCommunityPatch102PlusMaps2_R12c.zip) | ZIP header verified |
| R12d | 2v2 | [Download](http://app-direct.net/production/public/files/1.02+/R12d/KWCommunityPatch102PlusMaps2_R12d.zip) | HTTP 404 |
| R13 | 2v2 | [Download](http://app-direct.net/production/public/files/1.02+/R13/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13b | 2v2 | [Download](http://app-direct.net/production/public/files/1.02+/R13b/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13c | 2v2 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13d | 2v2 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R14 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14.zip) | ZIP header verified |
| R14 Beta 2 (test) | 2v2 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14_TEST.zip) | HTTP 404 |
| R15 Beta | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMaps_R15.zip) | ZIP header verified |
| R16 Beta | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps2_R16.zip) | ZIP header verified |
| R18 | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18/KWCommunityPatch102PlusMaps2_R18.zip) | ZIP header verified |
| R18c (test) | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18c/KWCommunityPatch102PlusMaps2_R18c.zip) | ZIP header verified |
| R18d | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18d/KWCommunityPatch102PlusMaps2_R18d.zip) | ZIP header verified |
| R18e | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18e/KWCommunityPatch102PlusMaps2_R18e.zip) | ZIP header verified |
| R18f | 2v2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18f/KWCommunityPatch102PlusMaps2_R18f.zip) | ZIP header verified |
| R19b (test) | 2v2 | [Download](https://drive.google.com/file/d/1u4Ib1ck1cm2d60ottzzrtnZcUEFRgGZ_/view?usp=sharing) | No ZIP returned |
| R19d (test) | 2v2 | [Download](https://drive.google.com/file/d/1m01SeK_kA7-b1qNf7D03GCk8N_hxfzdQ/view?usp=sharing) | No ZIP returned |
| R19e | 2v2 | [Download](https://drive.google.com/file/d/1E7TZhCr01zt3JIhMAAxUAUHXPWcNv2al/view?usp=sharing) | No ZIP returned |
| R19f | 2v2 | [Download](https://drive.google.com/file/d/14K7RIeS1czYfB_Mg7-Ac0HeSQ_RtRFaQ/view?usp=sharing) | No ZIP returned |
| R19g | 2v2 | [Download](https://drive.google.com/file/d/1G79_X5d7ZvUtsydXY6evlT3DVc4Q2HBO/view?usp=sharing) | Full managed download verified; login required |
| R19i (test) | 2v2 | [Download](https://drive.google.com/file/d/1VfIrLdXfzNRKZqvPLfpinzLYYeUPGKfG/view?usp=share_link) | HTTP 404 |
| R19j | 2v2 | [Download](https://drive.google.com/file/d/1kAe3Y1aEccWXaC6mmpFtl950oViLnqU7/view?usp=share_link) | Full managed download verified; login required |
| R20 | 2v2 | [Download](https://drive.google.com/file/d/1HYP5rwX1LqMhgZral82qOt15WPrtK-Ko/view?usp=share_link) | ZIP header verified |
| R20b | 2v2 | [Download](https://drive.google.com/file/d/1nP2LTE4B8O5Jg_p37jQEi4NG9IKgEusL/view?usp=share_link) | ZIP header verified |
| R20c | 2v2 | [Download](https://drive.google.com/file/d/10ZlCBpOO3mbdHNoHE9iQhwhJzefmqQkV/view?usp=sharing) | HTTP 404 |
| R20d | 2v2 | [Download](https://drive.google.com/file/d/1qPnHqz-CV8KN4XRtnTFx97sqPbksY4Oy/view?usp=drive_link) | ZIP header verified |
| R20e | 2v2 | [Download](https://drive.google.com/file/d/1gfGhhzRLlZZ4DoW0kq7r2Nt33BMrlmac/view?usp=drive_link) | ZIP header verified |
| R21 | 2v2 | [Download](https://drive.google.com/file/d/1qGD77hGpf5CbORj1UrvG6aFMYTlQHdL3/view?usp=share_link) | HTTP 404 |
| R21b | 2v2 | [Download](https://drive.google.com/file/d/174Tkx0NuNtDiJKK8h3HgvozVHRh5bYGV/view?usp=share_link) | ZIP header verified |
| R21c | 2v2 | [Download](https://drive.google.com/file/d/1oVuKct4lma6H3CFNfeRnzTOJb1dxrtpo/view?usp=drive_link) | HTTP 404 |
| R21c | 2v2 | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13898) | ZIP header verified |
| R21d | 2v2 | [Download](https://drive.google.com/file/d/1GvMnZRY0HYu0rdvgFYjKK6W4skfwtwuz/view?usp=drive_link) | ZIP header verified |
| R21e | 2v2 | [Download](https://drive.google.com/file/d/1pwT8UBaOhgRgN-7KSKYUIDgSHjNDwIhh/view?usp=drive_link) | ZIP header verified |
| R21f (Hotfix) | 2v2 | [Download](https://drive.google.com/file/d/1684t2iodKDkGkbSwqlvey76QtnIb2XiS/view?usp=drive_link) | ZIP header verified |
| R21g | 2v2 | [Download](https://drive.google.com/file/d/10JGAhykihXFwhlRkHDsSnJfifqVsIkxF/view?usp=drive_link) | ZIP header verified |
| R21h | 2v2 | [Download](https://drive.google.com/file/d/1KyuBEU9BsePHdxVzlciJHfxVIF0feRHE/view?usp=drive_link) | ZIP header verified |
| R21i | 2v2 | [Download](https://drive.google.com/file/d/1OqmytcKeW3kj045F0ZRDc-tgdVZVG4qF/view?usp=drive_link) | ZIP header verified |
| R21j | 2v2 | [Download](https://drive.google.com/file/d/1cMSXgNGr_qsbv1gvRNsPf1bCq5mLsd7X/view?usp=drive_link) | ZIP header verified |
| R22 | 2v2 | [Download](https://drive.google.com/file/d/1Hj-6u7Oe-Dwcg3KHmZYpMLfARZLS3Apd/view?usp=drive_link) | HTTP 404 |
| R22b | 2v2 | [Download](https://drive.google.com/file/d/1EBd5evEICofdsE2Vnmi25gm8gXygEVhM/view?usp=drive_link) | ZIP header verified |
| R22c | 2v2 | [Download](https://drive.google.com/file/d/1QV-dE6F-wfKdexfIFzsto5gHv64pTZ5-/view?usp=drive_link) | ZIP header verified |
| R22d | 2v2 | [Download](https://drive.google.com/file/d/15quhrKRo8zoyzsv_evWi7xgCFiJGYVym/view?usp=drive_link) | ZIP header verified |
| R22e | 2v2 | [Download](https://drive.google.com/file/d/1fwp-LUt4g3OFbidtc-XKVCppuJAEHXMW/view?usp=drive_link) | ZIP header verified |
| R22f | 2v2 | [Download](https://drive.google.com/file/d/1ZBra9Em49iNtW1Iv1MN8rz8n4_if3eOe/view?usp=drive_link) | ZIP header verified |
| R22g | 2v2 | [Download](https://drive.google.com/file/d/1WvOg8IGcvsgHHHhIpdfiXR_wqb6falSe/view?usp=drive_link) | ZIP header verified |
| R22h | 2v2 | [Download](https://drive.google.com/file/d/1XnmDbZrgt3nZXGlWMDYYE-seHPws6dDx/view?usp=drive_link) | ZIP header verified |
| R22j | 2v2 | [Download](https://drive.google.com/file/d/1kO7kbTx924nxfaVLbRZc88ksxMzv24la/view?usp=drive_link) | ZIP header verified |
| R23 | 2v2 | [Download](https://drive.google.com/file/d/1uzl36vomOg2KYDeEj6uDmJP6CLdipR8P/view?usp=drive_link) | ZIP header verified |
| R23a | 2v2 | [Download](https://drive.google.com/file/d/1PyAjGHtT1MaPVOe90cUuFapnJavBssW_/view?usp=drive_link) | ZIP header verified |
| R23b | 2v2 | [Download](https://drive.google.com/file/d/1ugvPV1ZpZOb5kk_JlLrvQPrBg_sMZjTe/view?usp=drive_link) | ZIP header verified |
| R23c (test) | 2v2 | [Download](https://drive.google.com/file/d/1uf58VYrjLDOKAwzruqL0TUml6H79Ssf6/view?usp=drive_link) | ZIP header verified |
| R23d | 2v2 | [Download](https://drive.google.com/file/d/1HzM-Y4XhlfJzdov-ZqhKCrYc5E-UhZi_/view?usp=sharing) | ZIP header verified |
| R23e | 2v2 | [Download](https://drive.google.com/file/d/1Jm12mewHRrk2L8F07DOW3wRiiOCTiLol/view?usp=drive_link) | ZIP header verified |
| R23f | 2v2 | [Download](https://drive.google.com/file/d/1a4FUCAKhABSGoNYxOUCyK-amRdL3gwWh/view?usp=drive_link) | ZIP header verified |
| R23h | 2v2 | [Download](https://drive.google.com/file/d/1obYjQBBN3QEdidRQjUPNWpORjmPqePXh/view?usp=drive_link) | ZIP header verified |
| R23x | 2v2 | [Download](https://drive.google.com/file/d/1_B9JbzPewb3hsnGxWcUlpj9QcIBtnE8W/view?usp=drive_link) | ZIP header verified |
| R23z | 2v2 | [Download](https://drive.google.com/file/d/1zLjQQfuy0xpntLqGU-a0R-RW4SNxLQyP/view?usp=drive_link) | ZIP header verified |
| R24d (test) | 2v2 | [Download](https://drive.google.com/file/d/14lNi19935COgN9uQt9wjLhhwaBK5stEJ/view?usp=sharing) | HTTP 404 |
| R24e (test) | 2v2 | [Download](https://drive.google.com/file/d/1-jUL1BqI8NXaLl1E7Fiu1yL2cedCrzcz/view?usp=sharing) | HTTP 404 |
| R24f | 2v2 | [Download](https://drive.google.com/file/d/1jwOF6pjEWUFogmmMBGbIL_dt6Zp9z3_7/view?usp=drive_link) | HTTP 404 |
| R24g | 2v2 | [Download](https://drive.google.com/file/d/1b9eDl3ODi5ZZhdIUincIuByo7dfW79Rt/view?usp=drive_link) | HTTP 404 |
| R24h | 2v2 | [Download](https://drive.google.com/file/d/1Yz4TiR4GDWUwJuzCGmVHQSBw-kjV37cG/view?usp=drive_link) | ZIP header verified |
| R24i | 2v2 | [Download](https://drive.google.com/file/d/1w6U45k1Fr9wrKFZgtjZsRkvwwZN3JGm-/view?usp=drive_link) | ZIP header verified |
| R24j | 2v2 | [Download](https://drive.google.com/file/d/1L9j2-YvDcptCLd0LPdNa-gYU4M7Yk9qZ/view?usp=drive_link) | ZIP header verified |
| R24k | 2v2 | [Download](https://drive.google.com/file/d/1O2roa9RwqW11WCcoMZHwzWN5bnqHApUl/view?usp=drive_link) | ZIP header verified |
| R24l (test) | 2v2 | [Download](https://drive.google.com/file/d/1WmWlDNo7n6ctBhLAutua2yPeh7aZ92S-/view?usp=drive_link) | ZIP header verified |
| R24m | 2v2 | [Download](https://drive.google.com/file/d/1OGdVVs8vdM3SlYOCxZPtXd6m1v3F92ni/view?usp=drive_link) | ZIP header verified |
| R24o | 2v2 | [Download](https://drive.google.com/file/d/1PCqruEBW9DVRAvgo1aza3BsomboMvf-o/view?usp=drive_link) | ZIP header verified |
| R24p | 2v2 | [Download](https://drive.google.com/file/d/1lJPkRptDyxWddYynz546H0Pu91_15SRA/view?usp=drive_link) | ZIP header verified |
| R24q | 2v2 | [Download](https://drive.google.com/file/d/1AT8aSDzBh98qwS07VV2bDfhkjdoMeYFC/view?usp=drive_link) | ZIP header verified |
| R13c | Predatore 2 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R13d | Predatore 2 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R14 | Predatore 2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R14/KWCommunityPatch102PlusMapsA_R14.zip) | ZIP header verified |
| R15 Beta | Predatore 2 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMapsA_R15.zip) | ZIP header verified |
| R8 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R8/KWCommunityPatch102PlusMaps3_R8.zip) | ZIP header verified |
| R9 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R9/KWCommunityPatch102PlusMaps3_R9.zip) | ZIP header verified |
| R10 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R10/KWCommunityPatch102PlusMaps3_R10C.zip) | ZIP header verified |
| R11 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R11/KWCommunityPatch102PlusMaps3_R11.zip) | ZIP header verified |
| R12 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12/KWCommunityPatch102PlusMaps3_R12.zip) | ZIP header verified |
| R12b | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12b/KWCommunityPatch102PlusMaps3_R12b.zip) | ZIP header verified |
| R12c | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R12c/KWCommunityPatch102PlusMaps3_R12c.zip) | ZIP header verified |
| R12d | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R12d/KWCommunityPatch102PlusMaps3_R12d.zip) | HTTP 404 |
| R13 | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R13/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13 Beta 2 | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R13/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13b | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R13b/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13c | 4v4 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R13d | 4v4 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMaps_R13.zip) | HTTP 404 |
| R14 | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14.zip) | HTTP 404 |
| R14 Beta 2 (test) | 4v4 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMaps_R14_TEST.zip) | HTTP 404 |
| R15 Beta | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMaps_R15.zip) | ZIP header verified |
| R16 Beta | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R16%20Beta/KWCommunityPatch102PlusMaps3_R16.zip) | ZIP header verified |
| R18 | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18/KWCommunityPatch102PlusMaps3_R18.zip) | ZIP header verified |
| R18c (test) | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18c/KWCommunityPatch102PlusMaps3_R18c.zip) | ZIP header verified |
| R18d (test) | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18d/KWCommunityPatch102PlusMaps3_R18d.zip) | ZIP header verified |
| R18e | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18e/KWCommunityPatch102PlusMaps3_R18e.zip) | ZIP header verified |
| R18f | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18f/KWCommunityPatch102PlusMaps3_R18f.zip) | ZIP header verified |
| R18f | 4v4 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R18d/KWCommunityPatch102PlusMaps3_R18d.zip) | Wrong revision (excluded) |
| R19d (test) | 4v4 | [Download](https://drive.google.com/file/d/1ipODOFADCIHZBG6BxEgL8Fet02_mjEiH/view?usp=sharing) | No ZIP returned |
| R19e | 4v4 | [Download](https://drive.google.com/file/d/1aHd447ffh7K5s_MoiP3WF8iKpYZxby92/view?usp=sharing) | Full managed download verified; login required |
| R19f | 4v4 | [Download](https://drive.google.com/file/d/1dxFL1_03ozpS6nEE8EJgslGJvmTar4i3/view?usp=sharing) | No ZIP returned |
| R19g | 4v4 | [Download](https://drive.google.com/file/d/1_k73Zmg0DuZUAKf8ZrqlKJX5fU-kutBb/view?usp=sharing) | Full managed download verified; login required |
| R19i (test) | 4v4 | [Download](https://drive.google.com/file/d/1ZcyQJUiQiUkrnOSYxOUbDYvvomnp84pc/view?usp=share_link) | HTTP 404 |
| R19j | 4v4 | [Download](https://drive.google.com/file/d/1Qgh3LtM9n4fGVUciSWnOaazOKHezNARM/view?usp=share_link) | Full managed download verified; login required |
| R20 | 4v4 | [Download](https://drive.google.com/file/d/10HO9EdHm4m1myvsK_0M0gOiWoAelrY5G/view?usp=share_link) | ZIP header verified |
| R20b | 4v4 | [Download](https://drive.google.com/file/d/1nnHsQH7PGPjqHHMx2jaiPXw8rH03F8zk/view?usp=share_link) | ZIP header verified |
| R20c | 4v4 | [Download](https://drive.google.com/file/d/1cnD1qdRvlBWm9H0blRF2-U822jFPr0VQ/view?usp=sharing) | HTTP 404 |
| R20e | 4v4 | [Download](https://drive.google.com/file/d/1h-0Cj71wA1H4g3EgH0MCAz-X8ZA93IOV/view?usp=drive_link) | ZIP header verified |
| R21 | 4v4 | [Download](https://drive.google.com/file/d/16vgwbZVmvC_YKomeEQTUG3EmGdoVx83L/view?usp=share_link) | HTTP 404 |
| R21b | 4v4 | [Download](https://drive.google.com/file/d/1AYGhYN0NdvSS3Zbz2EFZXcyyOf5r_vL1/view?usp=share_link) | ZIP header verified |
| R21c | 4v4 | [Download](https://drive.google.com/file/d/1Q_T019XaWaiMhVG_MTADonyNb9rtWox0/view?usp=drive_link) | ZIP header verified |
| R21d | 4v4 | [Download](https://drive.google.com/file/d/1PiaR4Qcu59mAqZP4-gh7JE5ktLARTqen/view?usp=drive_link) | ZIP header verified |
| R21e | 4v4 | [Download](https://drive.google.com/file/d/1OtmbDN9pw5x4n8gwn4x0A-xcnS6xxBFz/view?usp=drive_link) | ZIP header verified |
| R21f (Hotfix) | 4v4 | [Download](https://drive.google.com/file/d/1N8lYnj4ihOeQzmev39lhRJTbnGC2tbQL/view?usp=drive_link) | ZIP header verified |
| R21g | 4v4 | [Download](https://drive.google.com/file/d/1Wr72Cwe6vEJnBHPwmej6Y5UL7J7shIvD/view?usp=drive_link) | ZIP header verified |
| R21h | 4v4 | [Download](https://drive.google.com/file/d/1XD4PYhHOTmmIRmIJwThxrwGJ7nQGrshr/view?usp=drive_link) | ZIP header verified |
| R21j | 4v4 | [Download](https://drive.google.com/file/d/1wPpq2f5Jvhc69xlITrevd9LH6qN9_0D6/view?usp=drive_link) | ZIP header verified |
| R22 | 4v4 | [Download](https://drive.google.com/file/d/1l4d4744IJBkD-4U7hTHsZxAtwDV6L3iY/view?usp=drive_link) | HTTP 404 |
| R22b | 4v4 | [Download](https://drive.google.com/file/d/18ONDRmalE3NPOT4dGfNHLo_xrYsOqElw/view?usp=drive_link) | ZIP header verified |
| R22c | 4v4 | [Download](https://drive.google.com/file/d/1AW6B0uuY52eeQ2iQxsJjd0pYuBblhCe-/view?usp=drive_link) | ZIP header verified |
| R22d | 4v4 | [Download](https://drive.google.com/file/d/1WRu_bxyO60WOn7jRJUacVfzQ2XIuy957/view?usp=drive_link) | ZIP header verified |
| R22e | 4v4 | [Download](https://drive.google.com/file/d/1AmMH31Dgn4jbchzlRfkKZu_pqeF6N4O0/view?usp=drive_link) | ZIP header verified |
| R22f | 4v4 | [Download](https://drive.google.com/file/d/18BYte55VC31Xl3JMjYza1YbEX2Pk6ba-/view?usp=drive_link) | ZIP header verified |
| R22g | 4v4 | [Download](https://drive.google.com/file/d/1An67A8pWcTITup_Z77UjaNbJRQlnhB9q/view?usp=drive_link) | ZIP header verified |
| R22h | 4v4 | [Download](https://drive.google.com/file/d/1DKfUQYBiIwCPvvzfWh-BFox8zoFVFBLK/view?usp=drive_link) | No ZIP returned |
| R22i | 4v4 | [Download](https://drive.google.com/file/d/1A4rf7U8avgcv2wVwc-LvrFfySMr7DDn2/view?usp=drive_link) | ZIP header verified |
| R22j | 4v4 | [Download](https://drive.google.com/file/d/1vuRUEWN9exQ8JED9W7h72L8SeymBr-ED/view?usp=drive_link) | ZIP header verified |
| R23 | 4v4 | [Download](https://drive.google.com/file/d/1weFte1tzsmuzPWiZBFmS9zm8RVFMuzO-/view?usp=drive_link) | ZIP header verified |
| R23a | 4v4 | [Download](https://drive.google.com/file/d/1tfyQ9iwpyNK52ILDuqldwtRLiY8y0YjR/view?usp=drive_link) | ZIP header verified |
| R23b | 4v4 | [Download](https://drive.google.com/file/d/1cPoEs8d3LFlB6fAqlH8jE5wbVAhekhY4/view?usp=drive_link) | ZIP header verified |
| R23c (test) | 4v4 | [Download](https://drive.google.com/file/d/1Ek1eW7SpwwUJ4kD-gKrgjn8dwECR-1At/view?usp=drive_link) | ZIP header verified |
| R23d | 4v4 | [Download](https://drive.google.com/file/d/1cYzBUHAiLfsE6ympzz2g_QV_7wxkhQ9q/view?usp=drive_link) | ZIP header verified |
| R23e | 4v4 | [Download](https://drive.google.com/file/d/1NXY4jf0iqWlfaSkIAU57AH5ZcVp_xO0b/view?usp=drive_link) | ZIP header verified |
| R23f | 4v4 | [Download](https://drive.google.com/file/d/1lJEikU6QD9XV5XPLndklnXQB5dJQ5StT/view?usp=drive_link) | ZIP header verified |
| R23h | 4v4 | [Download](https://drive.google.com/file/d/1d0bGsnAJ6cFOec3qLFzMYqm96IfKNTTq/view?usp=drive_link) | ZIP header verified |
| R23x | 4v4 | [Download](https://drive.google.com/file/d/15efuHUvX1z1At0ysEDTvFOMjeGVviVXP/view?usp=drive_link) | ZIP header verified |
| R23z | 4v4 | [Download](https://drive.google.com/file/d/197vSjHDMR_-gBsMQIuAWKjdV77GfbYM-/view?usp=drive_link) | ZIP header verified |
| R24d (test) | 4v4 | [Download](https://drive.google.com/file/d/12cR436cTp02WGMRIRU0otb26OIarRxLH/view?usp=drive_link) | HTTP 404 |
| R24e (test) | 4v4 | [Download](https://drive.google.com/file/d/15D2ZBzxnOUoeTqMRG39ISv3x2WWSsxDm/view?usp=drive_link) | HTTP 404 |
| R24f | 4v4 | [Download](https://drive.google.com/file/d/1_6SRo0kuT_JO9irsE_nlvwH6IdSAzIaZ/view?usp=drive_link) | HTTP 404 |
| R24g | 4v4 | [Download](https://drive.google.com/file/d/1aJgCAJCsPnBIcg6LVSWjnQ2lsbi-s_qj/view?usp=drive_link) | HTTP 404 |
| R24h | 4v4 | [Download](https://drive.google.com/file/d/1J8eBrg1d7wkqZn6zOjIBsQoOfEuCp5Ox/view?usp=drive_link) | ZIP header verified |
| R24i | 4v4 | [Download](https://drive.google.com/file/d/17cvWgUsaOeBtI2KDtbdi1pkNbSRIkymT/view?usp=drive_link) | ZIP header verified |
| R24j | 4v4 | [Download](https://drive.google.com/file/d/1rJMnd8VE_WL88YSv_AO3RjvOFyLF8oTf/view?usp=drive_link) | ZIP header verified |
| R24k | 4v4 | [Download](https://drive.google.com/file/d/1ixUcQkNYGjlq_Aw6hw39xL7h9Yl6suGI/view?usp=drive_link) | ZIP header verified |
| R24l (test) | 4v4 | [Download](https://drive.google.com/file/d/1eePir4FRpbl1Tr8putnErkj3OnlA2ki9/view?usp=drive_link) | HTTP 404 |
| R24m | 4v4 | [Download](https://drive.google.com/file/d/1xRycy7Gff_qUv5WWccO3zOyVVfaeOgHW/view?usp=drive_link) | ZIP header verified |
| R24o | 4v4 | [Download](https://drive.google.com/file/d/15p6xlUBFseSZxYxRZ6AXINKlowkjC03O/view?usp=drive_link) | ZIP header verified |
| R24p | 4v4 | [Download](https://drive.google.com/file/d/1EUb-dhKkVkhclvT1RcbPNgkjQE7s_d1-/view?usp=drive_link) | ZIP header verified |
| R24q | 4v4 | [Download](https://drive.google.com/file/d/1cwZYxE1krBQHT3reNRU8LDueNcbXeDsI/view?usp=drive_link) | ZIP header verified |
| R13c | Predatore 3 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R13d | Predatore 3 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R14 | Predatore 3 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMapsA_R14.zip) | HTTP 404 |
| R15 Beta | Predatore 3 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMapsA_R15.zip) | ZIP header verified |
| R11 | Pack 4 | [Download](http://app-direct.net/production/public/files/1.02+/R11/KWCommunityPatch102PlusMaps4_R11.zip) | HTTP 404 |
| R13c | Predatore 1 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13c/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R13d | Predatore 1 | [Download](http://app-direct.net/production/public/files/1.02%2B/R13d/KWCommunityPatch102PlusMapsA_R13.zip) | HTTP 404 |
| R14 | Predatore 1 | [Download](http://app-direct.net/production/public/files/1.02+/R14/KWCommunityPatch102PlusMapsA_R14.zip) | HTTP 404 |
| R15 Beta | Predatore 1 | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/1.02+/R15/KWCommunityPatch102PlusMapsA_R15.zip) | ZIP header verified |
| F01 | arcademappack | [Download](https://kaneswrath.com/downloads/download/Arcade_R20c.zip) | HTTP 404 |
| F01e | arcademappack | [Download](https://kaneswrath.com/downloads/download/Arcade_R20e.zip) | HTTP 404 |
| F02 | arcademappack | [Download](https://kaneswrath.com/wp-content/uploads/download-manager-files/Arcade_R21c.zip) | HTTP 404 |
| F03 | arcademappack | [Download](https://kaneswrath.com/wp-content/uploads/download-manager-files/Arcade_F03_r21h.zip) | HTTP 404 |
| F04 | arcademappack | [Download](https://kaneswrath.com/downloads/download/Arcade_F04.zip) | HTTP 404 |
| F05 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F05/Arcade_F05.zip) | ZIP header verified |
| F06 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F06/Arcade_F06.zip) | ZIP header verified |
| F07 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F07/Arcade_F07.zip) | ZIP header verified |
| F08 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F08/Arcade_F08.zip) | ZIP header verified |
| F09 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F09/Arcade_F09-4.zip) | ZIP header verified |
| F10 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F10/Arcade_F10.zip) | ZIP header verified |
| F11 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F11/Arcade_F11.zip) | ZIP header verified |
| F12 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F12/Arcade_F12.zip) | ZIP header verified |
| F13 | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F13/Arcade_F13.zip) | ZIP header verified |
| F14d | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14/Arcade_F14u.zip) | ZIP header verified |
| F14e | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14e/Arcade_maps_F14e.zip) | ZIP header verified |
| F14f | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14f/Arcade_F14f.zip) | ZIP header verified |
| F14g | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14g/Arcade_F14g.zip) | ZIP header verified |
| F14h | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14h/Arcade_F14h.zip) | ZIP header verified |
| F14i | arcademappack | [Download](https://cgf-uploads.fra1.cdn.digitaloceanspaces.com/files/arcademappack/F14i/Arcade_F14i.zip) | ZIP header verified |
| R19 | neonbankaimaps1 | [Download](https://kaneswrath.com/downloads/download/R19-Neon-Banka-Maps.zip) | HTTP 404 |
| R19b | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1_W-0xRWsvOk6gAVlxpWp3wO89yKB5WW4/view?usp=sharing) | ZIP header verified |
| R19c | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1UBB-l-jK0JQRpw5Zo_hMy4NlBo_eE3B9/view?usp=sharing) | ZIP header verified |
| R20e | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1xWcUGyOqR-QjGZ3fvYkTMdusu9Qe8VPF/view?usp=sharing) | ZIP header verified |
| R20f | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1IhXsRtXHg2hChNxYRzMwSkOxPl0GzCsr/view?usp=sharing) | ZIP header verified |
| R21l | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1WGzzUN1UfYQcIGpMYbj_A4DU3asmf9Qg/view?usp=sharing) | ZIP header verified |
| R21m | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1ZS7-xEF4DvjbrCzIF3u0iDgm5OHWVSE8/view?usp=sharing) | ZIP header verified |
| R22n | neonbankaimaps1 | [Download](https://drive.google.com/file/d/1QkutktbWM-0v4h3WCwDYvisdm7Vgz5O1/view?usp=sharing) | ZIP header verified |
| R19 | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1H5WEbGwO08DEA8j6IJC-5P0o3f-vYJsd/view?usp=sharing) | ZIP header verified |
| R19c | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1XHr-9ZBgPfoKIxPogOy923FUtfSz-3u3/view?usp=sharing) | ZIP header verified |
| R19d | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1S2qxv8wNmXMtZ40Ql_AaF49zOODegmsW/view?usp=sharing) | ZIP header verified |
| R20e | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1Q0hQeUMkEZKtBEBv-4H8TsBD54qQYMCt/view?usp=sharing) | ZIP header verified |
| R20f | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1gR-OkhQHPySKi715E2jjOYv1qFSy4ssW/view?usp=sharing) | ZIP header verified |
| R20g | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1ZPI9TRadwMAS_RMgtWmxaJ-uqcwcO4uo/view?usp=sharing) | ZIP header verified |
| R21l | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1AYxwY-McTnoXGXGtPuqOgQZ413dKSUv3/view?usp=sharing) | ZIP header verified |
| R21m | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1aO7WUCZe-xiSQv8M2uRv0lqySweizitZ/view?usp=sharing) | ZIP header verified |
| R22n | neonbankaimaps2 | [Download](https://drive.google.com/file/d/1h8REwhEa8ZMnM6X4r53hHduhH9F5OixO/view?usp=sharing) | ZIP header verified |
| R19 | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1-ZK6c_YXeNQtowdVf4dZ5fOnlgMUuzCg/view?usp=sharing) | ZIP header verified |
| R19 | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1-ZK6c_YXeNQtowdVf4dZ5fOnlgMUuzCg/view?usp=sharing) | ZIP header verified |
| R19b | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1sYCo1RugHfJy1v-nzLUq6fg3d5Tmouph/view?usp=sharing) | ZIP header verified |
| R19c | neonbankaimaps3 | [Download](https://drive.google.com/file/d/19cUkAmQhjApQNRqtxG1WEtvpvENnV0Jg/view?usp=sharing) | ZIP header verified |
| R19d | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1f-ND7mZbnYCYl5g2Bp96CZ12Ey23fQgt/view?usp=sharing) | ZIP header verified |
| R20e | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1sijuNxeQVCtUG1iQpEedw5H0v2i_7F8o/view?usp=sharing) | ZIP header verified |
| R20f | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1ITDxnhaDOaWBssXjkeVbldHt8_YYz7Kr/view?usp=sharing) | ZIP header verified |
| R21l | neonbankaimaps3 | [Download](https://drive.google.com/file/d/19u5rK66YxbTa1osEuMe9ttq2jExnpBjw/view?usp=sharing) | ZIP header verified |
| R21m | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1gqknAoqStTSQPzxWppMr4A8icvwXXjHW/view?usp=sharing) | ZIP header verified |
| R22n | neonbankaimaps3 | [Download](https://drive.google.com/file/d/1EbqaKVdJC4pAQe9Xd-0VtrY4b3JHEoPN/view?usp=sharing) | ZIP header verified |
| F03 (R21h) | arcade | [Download](https://kaneswrath.com/?yh_download_id=13851&attachment_id=13856) | ZIP header verified |

## Website mirrors

| Package | Download | Version list | Check |
| --- | --- | --- | --- |
| R22j-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13975) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22h-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13976) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22g-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13977) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22f-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13978) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22e-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13979) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22d-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13980) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22c-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13981) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22b-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13982) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13973&attachment_id=13983) | [Source](https://kaneswrath.com/download/r22-1vs1-map-pack/) | ZIP header verified |
| R22j-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13960) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22h-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13961) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22g-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13962) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22f-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13963) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22e-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13964) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22d-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13965) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22c-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13966) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22b-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13967) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13958&attachment_id=13968) | [Source](https://kaneswrath.com/download/r22-2vs2-map-pack/) | ZIP header verified |
| R22j-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13956) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22i-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13955) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22h-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13953) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22g-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13954) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22f-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13952) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22e-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13951) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22d-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13950) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22c-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13949) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22b-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13948) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13946&attachment_id=13947) | [Source](https://kaneswrath.com/download/r22-4vs4-map-pack/) | ZIP header verified |
| R22j-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13984&attachment_id=13985) | [Source](https://kaneswrath.com/download/r22-legacy-map-pack/) | ZIP header verified |
| R23z-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15483) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23x-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15464) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23h-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15180) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23e-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15137) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23d-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15119) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23c-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=15107) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23b-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=14787) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23a-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=14673) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14535&attachment_id=14542) | [Source](https://kaneswrath.com/download/r23-1vs1-map-pack/) | ZIP header verified |
| R23z-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=15482) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23x-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=15463) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23h-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=15179) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23e-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=15138) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23d-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=15120) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23b-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=14788) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23a-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=14674) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14539&attachment_id=14543) | [Source](https://kaneswrath.com/download/r23-2vs2-map-pack/) | ZIP header verified |
| R23z-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=15481) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23x-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=15462) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23h-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=15178) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23e-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=15139) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23d-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=15121) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23b-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=14790) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23a-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=14672) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R23-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14540&attachment_id=14544) | [Source](https://kaneswrath.com/download/r23-4vs4-map-pack/) | ZIP header verified |
| R24q-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=16114) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24p-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=16097) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24m-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=16047) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24k-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=16018) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24j-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15973) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24i-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15963) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24h-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15944) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24g-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15934) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24f-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15929) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24e-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15920) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24d-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15886) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24c-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15856) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24b-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15820&attachment_id=15845) | [Source](https://kaneswrath.com/download/r24-1vs1-map-pack/) | ZIP header verified |
| R24q-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=16113) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24p-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=16096) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24m-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=16045) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24k-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=16017) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24j-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15972) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24i-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15962) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24h-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15943) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24g-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15933) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24f-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15928) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24e-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15919) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24d-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15885) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24c-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15855) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24b-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15844) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15821&attachment_id=15828) | [Source](https://kaneswrath.com/download/r24-2vs2-map-pack/) | ZIP header verified |
| R24q-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=16112) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24p-4vs4_Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=16099) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24o-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=16092) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24m-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=16044) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24k-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=16016) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24j-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15971) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24i-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15961) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24h-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15942) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24g-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15932) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24f-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15927) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24e-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15918) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24d-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15887) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24c-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15854) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24b-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15846) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15822&attachment_id=15827) | [Source](https://kaneswrath.com/download/r24-4v4-map-pack/) | ZIP header verified |
| R24q-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=16111) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24p-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=16094) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24m-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=16043) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24k-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=16015) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24j-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15970) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24i-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15960) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24h-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15941) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24g-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15931) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24f-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15926) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24e-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15917) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24d-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15888) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24c-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15853) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24b-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15848) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R24-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=15819&attachment_id=15826) | [Source](https://kaneswrath.com/download/r24-legacy-map-pack/) | ZIP header verified |
| R25i-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=17016) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25h-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=17002) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25g-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16978) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25f-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16969) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25e-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16957) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25c-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16933) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25b-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16926) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R25-All-in-One-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=16892&attachment_id=16913) | [Source](https://kaneswrath.com/download/r25-all-in-one-map-pack/) | ZIP header verified |
| R23z-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=15484) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| R23x-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=15461) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| R23f-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=15148) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| R23d-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=15118) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| R23b-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=14791) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| R23-Legacy-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=14555&attachment_id=14556) | [Source](https://kaneswrath.com/download/legacy-map-pack-r23/) | ZIP header verified |
| Arcade_R20c.zip | [Download](https://kaneswrath.com/wp-content/uploads/ys-download/Arcade_R20c.zip) | [Source](https://kaneswrath.com/download/arcade-map-pack/) | ZIP header verified |
| Arcade_R20e.zip | [Download](https://kaneswrath.com/wp-content/uploads/ys-download/Arcade_R20e.zip) | [Source](https://kaneswrath.com/download/arcade-map-pack/) | ZIP header verified |
| R21j-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13883) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21i_1vs1_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13884) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21h_1vs1_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13885) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21g_1vs1_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13886) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21f_1vs1_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13887) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21e-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13888) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21d-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13889) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21c-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13890) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21b-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13891) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21-1vs1-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13880&attachment_id=13892) | [Source](https://kaneswrath.com/download/r21-1vs1-map-pack/) | ZIP header verified |
| R21j-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13905) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21i_2vs2_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13904) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21h_2vs2_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13903) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21g_2vs2_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13902) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21f_2vs2_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13901) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21e-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13900) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21d-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13899) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21c-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13898) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21b-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13897) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21-2vs2-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13895&attachment_id=13896) | [Source](https://kaneswrath.com/download/r21-2vs2-map-pack/) | ZIP header verified |
| R21j-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13918) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21h_4vs4_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13916) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21g_4vs4_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13915) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21f_4vs4_Map_Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13914) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21e-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13913) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21d-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13912) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21c-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13911) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21b-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13910) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |
| R21-4vs4-Map-Pack.zip | [Download](https://kaneswrath.com/?yh_download_id=13908&attachment_id=13917) | [Source](https://kaneswrath.com/download/r21-4vs4-map-pack/) | ZIP header verified |

## Additional sources

- [R20e 1v1, 2v2 and 4v4 MEGA folder](https://mega.nz/folder/0eEnVK5B#89AhDFKaBAeGJxuJT_X0eA): Three public packs; ZIP headers, sizes and installer filenames verified (ZIP headers and indexes verified).

The R20 MEGA folder was linked by [CNCSeries](https://cncseries.ru/kw-patch-1-02/). Its three ZIPs were checked using public file metadata, decrypted byte ranges, and ZIP central directories. They contain the R20e 1v1, 2v2 and 4v4 installers. MEGA is a manual fallback; Tacitus currently downloads R20e through the Command Post links above.

## Leads that did not yield public downloads

- [Masterleaf R19 MEGA folder](https://mega.nz/folder/lHkkUDZb#S9s1jiTvxti-ZA3Gmtjlbg): public API returned unavailable.
- [Masterleaf R18 Drive folder](https://drive.google.com/drive/folders/1BGQolbVOsNtRds4b4JQ37_Ph6yb3HwlZ): HTTP 404.
- Shatabrick R12d links: [pack 1](https://www.shatabrick.net/downloads/102plusmp_d1.php), [pack 2](https://www.shatabrick.net/downloads/102plusmp_d2.php), [pack 3](https://www.shatabrick.net/downloads/102plusmp_d3.php). Requests timed out.
- Archived R20 pages were recovered through the Wayback Machine, but did not expose additional downloadable ZIPs.
