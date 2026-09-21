# Tacitus 0.1.0 release candidate

Portable replay library and playback manager for Command & Conquer 3: Kane's Wrath.
Download `tacitus.exe` and run it on Windows x64. No installer is required.
Kane's Wrath must be installed to play replays.

## Included

- Replay imports with duplicate detection, match details, search and sorting.
- Faction monograms and grouped filters requiring all selected factions.
- Game installation detection and exact replay-content preparation.
- Downloads for supported missing historical map packs and offline cache reuse.
- Optional `.kwreplay` association, including imports into an already running app.
- Embedded WebView2 runtime; first launch extracts it into the user's local cache.

## Candidate limitations

- The executable is unsigned. Publisher signing and clean Windows 10/11 acceptance
  checks are pending; this candidate is not yet approved for public release.
- Some exact historical and custom map packs are unavailable. The development
  catalogue's 96.4% content coverage includes locally cached packs and is not a
  guarantee of automatic downloads on a fresh installation.
- There is no automatic updater. Obtain later builds from this repository's
  Releases page and replace the executable after closing Tacitus. Data is kept
  separately. Reapply the file association if you change the executable's path.
- The bundled browser receives updates through new Tacitus builds.

## Before publishing

- [ ] Test the final candidate on clean Windows 10 and Windows 11 standard-user profiles.
- [ ] Verify first/cached launch, spaces and Unicode in paths, and moved executable.
- [ ] Verify Explorer opening with Tacitus closed/open and with another default app.
- [ ] Verify import, stock playback, fresh pack download, cancellation and offline playback.
- [ ] Watch representative stock and community replays through completion.
- [ ] Sign the final executable and regenerate its SHA-256 after signing.
- [ ] Verify the signed artifact's signature and checksum, then replace draft assets.
- [ ] Choose public distribution access; the repository is currently private.
- [ ] Update these notes with acceptance results and publish the draft.

The checksum can be checked with `Get-FileHash .\tacitus.exe -Algorithm SHA256`.

## Local validation

- Frontend: 28 tests passed; production build passed.
- Rust: 89 tests reported passing; four optional corpus checks returned early
  because `TACITUS_REPLAY_CORPUS` was unset (85 tests exercised fixtures).
- Command Post client: two offline tests passed in an isolated Python environment.
- npm audit: no reported vulnerabilities, including development dependencies.
- Rust audit: no vulnerability errors; seven upstream warnings remain for
  unmaintained transitive crates and the Linux GLib dependency's unsoundness advisory.
  Warnings remain visible in CI.
