// Windows x64 single-file release: pinned Microsoft runtime + static C runtime.
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, readFile, rename, rm, stat, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { spawn } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

async function hashFile(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

async function main() {
  if (process.platform !== 'win32') throw new Error('Build the portable EXE on Windows.');
  const runtime = JSON.parse(await readFile(join(root, 'src-tauri/resources/webview2-runtime.json'), 'utf8'));
  const cache = join(root, 'src-tauri/target/portable-cache');
  await mkdir(cache, { recursive: true });
  const cab = join(cache, `Microsoft.WebView2.FixedVersionRuntime.${runtime.version}.${runtime.architecture}.cab`);
  const existingHash = await hashFile(cab).catch(error => {
    if (error.code === 'ENOENT') return null;
    throw error;
  });
  if (existingHash !== runtime.sha256) {
    console.log(`Downloading WebView2 ${runtime.version} (${runtime.architecture}) from Microsoft...`);
    const temporary = `${cab}.${process.pid}.download`;
    try {
      const response = await fetch(runtime.url);
      if (!response.ok || !response.body) throw new Error(`Runtime download failed: HTTP ${response.status}`);
      await pipeline(Readable.fromWeb(response.body), createWriteStream(temporary, { flags: 'wx' }));
      if (await hashFile(temporary) !== runtime.sha256) throw new Error('Runtime checksum mismatch; refusing to build.');
      await rm(cab, { force: true });
      await rename(temporary, cab);
    } finally {
      await rm(temporary, { force: true });
    }
  }
  console.log(`Verified WebView2 ${runtime.version}. Building the portable EXE...`);
  const targetDir = join(root, 'src-tauri/target/portable-build');
  await new Promise((done, reject) => {
    const child = spawn(process.execPath, [
      join(root, 'node_modules/@tauri-apps/cli/tauri.js'),
      'build', '--no-bundle', '--target', runtime.target, '--features', 'portable', '--', '--locked', '--bin', 'silo',
    ], {
      cwd: root, stdio: 'inherit', windowsHide: true,
      env: {
        ...process.env,
        SILO_WEBVIEW2_CAB: cab,
        CARGO_TARGET_DIR: targetDir,
        RUSTFLAGS: `${process.env.RUSTFLAGS ?? ''} -C target-feature=+crt-static`.trim(),
      },
    });
    child.once('error', reject);
    child.once('exit', code => code === 0 ? done() : reject(new Error(`Portable build exited with code ${code}`)));
  });
  const output = join(root, 'release/silo.exe');
  await mkdir(dirname(output), { recursive: true });
  await copyFile(join(targetDir, runtime.target, 'release/silo.exe'), output);
  await writeFile(`${output}.sha256`, `${await hashFile(output)}  silo.exe\n`);
  console.log(`\nPortable release: ${output} (${((await stat(output)).size / 1024 ** 2).toFixed(1)} MiB)`);
  console.log('Distribute silo.exe alone. The .sha256 file is optional verification metadata.');
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
