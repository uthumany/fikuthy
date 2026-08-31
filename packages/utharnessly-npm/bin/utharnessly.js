#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { promises as fs } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pipeline } from 'node:stream/promises';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

const VERSION = '0.2.13';
const REPOSITORY = 'uthumany/utharnessly';
const BASE_URL = (process.env.UTHARNESSLY_RELEASE_BASE_URL || `https://github.com/${REPOSITORY}/releases/download/v${VERSION}`).replace(/\/$/, '');

function platformAsset() {
  const platform = process.platform;
  const arch = process.arch;
  if (platform === 'linux' && arch === 'x64') return ['utharnessly-linux-x64.tar.gz', 'tar.gz'];
  if (platform === 'darwin' && arch === 'x64') return ['utharnessly-macos-x64.tar.gz', 'tar.gz'];
  if (platform === 'darwin' && arch === 'arm64') return ['utharnessly-macos-arm64.tar.gz', 'tar.gz'];
  if (platform === 'win32' && arch === 'x64') return ['utharnessly-windows-x64.zip', 'zip'];
  throw new Error(`No published utharnessly binary for ${platform}/${arch}. Supported release targets are Linux x64, macOS x64/arm64, and Windows x64; use the source instructions at https://github.com/${REPOSITORY} on other targets.`);
}

function cacheRoot() {
  const base = process.env.XDG_CACHE_HOME || (process.platform === 'win32' ? process.env.LOCALAPPDATA : path.join(os.homedir(), '.cache')) || os.tmpdir();
  return path.join(base, 'utharnessly', VERSION);
}

async function download(url, destination) {
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok || !response.body) throw new Error(`download failed (${response.status}) for ${url}`);
  await pipeline(response.body, createWriteStream(destination));
}

async function verifyChecksum(archive, checksumFile) {
  const contents = await fs.readFile(checksumFile, 'utf8');
  const asset = path.basename(archive);
  const line = contents.split(/\r?\n/).find((entry) => {
    const fields = entry.trim().split(/\s+/);
    return fields.length >= 2 && fields.at(-1).replace(/^\*/, '') === asset;
  });
  if (!line) throw new Error(`SHA256SUMS does not contain ${asset}`);
  const expected = line.trim().split(/\s+/)[0].toLowerCase();
  const hash = createHash('sha256').update(await fs.readFile(archive)).digest('hex');
  if (hash !== expected) throw new Error(`checksum verification failed for ${asset}`);
}

async function extractArchive(archive, format, destination) {
  if (format === 'tar.gz') {
    await execFileAsync('tar', ['-xzf', archive, '-C', destination]);
    return;
  }
  if (format === 'zip') {
    await execFileAsync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', `Expand-Archive -LiteralPath '${archive.replaceAll("'", "''")}' -DestinationPath '${destination.replaceAll("'", "''")}' -Force`]);
    return;
  }
  throw new Error(`unsupported archive format: ${format}`);
}

async function ensureBinary(force = false) {
  const [asset, format] = platformAsset();
  const root = cacheRoot();
  const binary = path.join(root, process.platform === 'win32' ? 'utharness.exe' : 'utharness');
  if (!force) {
    try { await fs.access(binary); return { binary, ui: path.join(root, 'ui') }; } catch {}
  }
  await fs.rm(root, { recursive: true, force: true });
  await fs.mkdir(root, { recursive: true });
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), 'utharnessly-'));
  const archive = path.join(temp, asset);
  const checksums = path.join(temp, 'SHA256SUMS');
  try {
    process.stderr.write(`Downloading utharnessly v${VERSION} (${process.platform}/${process.arch})…\n`);
    await download(`${BASE_URL}/${asset}`, archive);
    await download(`${BASE_URL}/SHA256SUMS`, checksums);
    await verifyChecksum(archive, checksums);
    const extracted = path.join(temp, 'extracted');
    await fs.mkdir(extracted);
    await extractArchive(archive, format, extracted);
    const packageRoot = (await fs.readdir(extracted, { withFileTypes: true })).find((entry) => entry.isDirectory() && entry.name.startsWith('utharnessly-'));
    if (!packageRoot) throw new Error('release archive did not contain an utharnessly directory');
    const sourceRoot = path.join(extracted, packageRoot.name);
    await fs.copyFile(path.join(sourceRoot, path.basename(binary)), binary);
    if (process.platform !== 'win32') await fs.chmod(binary, 0o755);
    await fs.cp(path.join(sourceRoot, 'ui'), path.join(root, 'ui'), { recursive: true });
    return { binary, ui: path.join(root, 'ui') };
  } finally {
    await fs.rm(temp, { recursive: true, force: true });
  }
}

function run(binary, args) {
  const child = spawn(binary, args, { stdio: 'inherit', env: { ...process.env, UTHARNESS_RUNTIME_BIN: binary } });
  child.on('error', (error) => { console.error(`utharnessly: ${error.message}`); process.exitCode = 1; });
  child.on('exit', (code, signal) => { process.exitCode = signal ? 1 : (code ?? 1); });
}

const args = process.argv.slice(2);
if (args.includes('--version') || args.includes('-V')) {
  console.log(`utharnessly ${VERSION}`);
  process.exit(0);
}
if (args[0] === 'update') {
  try { await ensureBinary(true); console.log(`utharnessly ${VERSION} is ready.`); } catch (error) { console.error(`utharnessly update failed: ${error.message}`); process.exitCode = 1; }
} else if (args[0] === 'uninstall') {
  console.log('Remove the npm package with: npm uninstall -g utharnessly');
  console.log(`Remove the cached native runtime with: ${process.platform === 'win32' ? 'rmdir /s /q' : 'rm -rf'} "${cacheRoot()}"`);
} else {
  try {
    const { binary } = await ensureBinary();
    run(binary, args);
  } catch (error) {
    console.error(`utharnessly: ${error.message}`);
    process.exitCode = 1;
  }
}
