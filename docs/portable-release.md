# Portable Windows release

Build on Windows with Node.js LTS, the stable Rust MSVC toolchain, and the
Visual Studio C++ build tools:

```sh
npm run build:portable
```

The output is `release/tacitus.exe`, a Windows x64 application. Distribute that
file by itself. The accompanying `tacitus.exe.sha256` is optional checksum
metadata; the application does not need it to run. There is no installer or
runtime download when the application starts.

The build embeds the frontend, Rust application, SQLite, and the compressed
Microsoft WebView2 Fixed Version runtime. It statically links the application’s
C runtime and the WebView2 loader. It uses Tauri’s production build with bundling
disabled, so it does not produce an MSI or NSIS setup program.

## First launch and data

On first launch, Tacitus extracts its private browser into
`%LOCALAPPDATA%\tacitus\runtimes`. This can take several seconds before the
window appears. Later launches reuse the completed runtime. Preparation uses
Windows’ built-in `expand.exe` and `icacls.exe`, without an elevation request.
The permissions step supports the WebView2 renderer’s AppContainer on Windows 10.
The executable can be moved or renamed; no adjacent files are required.

An exclusive file lock serializes simultaneous first launches. Extraction takes
place in a temporary directory and is published only after the required browser
files and permissions are ready. An interrupted extraction is never marked as
complete. Missing core runtime files trigger extraction again on the next launch.

The catalogue remains at `%APPDATA%\tacitus\catalogue.sqlite3`. Replay content
continues to use `%LOCALAPPDATA%\tacitus\playback`. Removing the executable does
not delete either the catalogue or the caches. Kane’s Wrath and replay/map files
are separate from the application distribution.

## Runtime pin and updates

`src-tauri/resources/webview2-runtime.json` pins the Microsoft download URL,
version, architecture, and SHA-256. The build tool caches the downloaded CAB in
`src-tauri/target/portable-cache`. Both the download tool and Rust build script
verify its checksum before embedding it. Ordinary development and non-portable
builds do not need the CAB.

To update the browser, obtain a new x64 Fixed Version package from
[Microsoft’s WebView2 downloads](https://developer.microsoft.com/en-us/microsoft-edge/webview2/),
update the pin, and rebuild. Fixed Version runtimes do not update themselves, so
runtime updates ship with new Tacitus releases. Existing runtime caches have
version/hash-specific names and are left intact when another version runs.
Microsoft documents this deployment model and its Windows 10 permissions in
[WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution#the-fixed-version-runtime-distribution-mode).

## Validation

The cache preparation regression tests run without downloading or embedding a
real browser:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --bin tacitus
```

For a release smoke check, copy only the EXE into another directory, start it
with a fresh Windows user profile, and check both first and subsequent launches.
The `msedgewebview2.exe` child processes must run from Tacitus’s runtime cache.
The initial prototype passed a local Windows 11 smoke test using only the EXE
in a separate folder, empty redirected application-data directories, and a
renamed executable on the second launch. The bundled UI and private runtime
were verified through WebView2 and process paths. First launch took about
5.7 seconds and the cached launch about 0.5 seconds on the development machine.
All 88 Rust tests passed, including the four runtime preparation regressions.

This build command does not code-sign the executable; signing and clean Windows
10/11 testing remain release steps.
