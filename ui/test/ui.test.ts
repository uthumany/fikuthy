import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { editComposer } from '../src/tui/composer.js';
import { bannerVariant, effectiveLayout, getBreakpoint, getTermuxBreakpoint, workspaceWidths } from '../src/tui/responsive.js';
import { bannerHeight, bannerTier, detectTerminalCapabilities, letterColors, resolveIconMode } from '../src/tui/banner.js';
import { getColorMode } from '../src/tui/theme.js';
import { loadUiState, normalizeUiState, saveUiState } from '../src/tui/state.js';
import { loadSnapshot, parseGitSnapshot } from '../src/runtime.js';
import { runtimeBinary } from '../src/runtime-binary.js';
import { authMethods, modes, progress, providers, recommendedTools, tools } from '../src/setup-data.js';

test('setup exposes only runtime-backed providers and capabilities', () => {
  assert.deepEqual(modes.map(item => item.id), ['quick', 'full', 'developer', 'local_ai', 'custom', 'blank', 'import', 'exit']);
  assert.ok(providers.some(item => item.id === 'ollama' && item.key === undefined));
  assert.ok(providers.some(item => item.id === 'nvidia' && item.key === 'NVIDIA_API_KEY'));
  assert.ok(providers.every(item => item.id !== 'anthropic'));
  assert.ok(tools.every(item => item.risk === 'safe' || item.risk === 'ask'));
  assert.ok(recommendedTools.every(id => tools.some(item => item.id === id)));
  assert.deepEqual(authMethods.map(item => item.id), ['api_key', 'oauth', 'environment', 'skip']);
  assert.equal(progress(3, 4), 75); assert.equal(progress(0, 0), 100);
});

test('resolves native runtime paths for package and source layouts', () => {
  assert.equal(runtimeBinary('/workspace', { UTHARNESS_RUNTIME_BIN: '/opt/utharness' }), '/opt/utharness');
  assert.ok(runtimeBinary(process.cwd(), {}).endsWith(path.join('target', 'release', process.platform === 'win32' ? 'utharness.exe' : 'utharness')));
});

test('maps required terminal width breakpoints', () => {
  assert.equal(getBreakpoint(40), 'tiny'); assert.equal(getBreakpoint(60), 'compact'); assert.equal(getBreakpoint(80), 'standard');
  assert.equal(getBreakpoint(119), 'standard'); assert.equal(getBreakpoint(120), 'wide'); assert.equal(getBreakpoint(160), 'wide');
});

test('maps Termux mobile-first width tiers', () => {
  assert.equal(getTermuxBreakpoint(40), 'tiny'); assert.equal(getTermuxBreakpoint(50), 'compact');
  assert.equal(getTermuxBreakpoint(89), 'compact'); assert.equal(getTermuxBreakpoint(90), 'standard');
});

test('collapses workspace mode and banner responsively', () => {
  assert.equal(effectiveLayout('workspace', 'standard'), 'focus'); assert.equal(effectiveLayout('workspace', 'wide'), 'workspace');
  assert.equal(bannerVariant('full', 'wide', 40), 'full'); assert.equal(bannerVariant('full', 'standard', 30), 'full');
  assert.equal(bannerVariant('compact', 'wide', 40), 'compact'); assert.equal(bannerVariant('hide', 'wide', 40), 'hide');
  const widths = workspaceWidths(120); assert.equal(widths.navigation + widths.chat + widths.inspector, 118);
});

test('maps the complete banner width and height matrix without hiding by default', () => {
  assert.deepEqual([20, 30, 40, 60, 80, 100, 120, 160, 200].map(width => bannerTier(width, 40, 'full')), ['minimal', 'minimal', 'compact', 'wrapped', 'wrapped', 'compressed', 'full', 'full', 'full']);
  assert.ok(['minimal', 'compact', 'wrapped', 'compressed', 'full'].every(tier => bannerHeight(tier as Parameters<typeof bannerHeight>[0]) > 0));
  assert.equal(letterColors.length, 9);
});

test('falls back safely when Nerd Fonts, Unicode, or color are unavailable', () => {
  assert.equal(resolveIconMode('nerd', { TERM: 'xterm-256color' }), 'unicode');
  assert.equal(resolveIconMode('nerd', { TERM: 'xterm-256color', NERD_FONT: '1' }), 'nerd');
  assert.equal(resolveIconMode('unicode', { TERM: 'dumb' }), 'ascii');
  assert.deepEqual(detectTerminalCapabilities({ TERM: 'dumb', NO_COLOR: '1' }), { unicode: false, nerdFonts: false, color: false });
});

test('respects terminal color capability fallbacks', () => {
  assert.equal(getColorMode({ NO_COLOR: '1' }), 'mono'); assert.equal(getColorMode({ COLORTERM: 'truecolor' }), 'truecolor');
  assert.equal(getColorMode({ TERM: 'xterm-256color' }), 'ansi256');
});

test('composer supports multiline and editing shortcuts', () => {
  const plain = { backspace: false, delete: false, leftArrow: false, rightArrow: false, return: false, ctrl: false, shift: false };
  assert.deepEqual(editComposer('hello', 5, '', { ...plain, return: true, shift: true }), { value: 'hello\n', cursor: 6 });
  assert.deepEqual(editComposer('hello world', 11, 'w', { ...plain, ctrl: true }), { value: 'hello ', cursor: 6 });
  assert.equal(editComposer('send', 4, '', { ...plain, return: true }).submit, true);
  assert.deepEqual(editComposer('draft', 3, 'u', { ...plain, ctrl: true }), { value: '', cursor: 0 });
});

test('persists and restores safe UI preferences', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'utharness-ui-test-')); const file = path.join(directory, 'ui.json');
  const state = normalizeUiState({ bannerMode: 'compact', layoutMode: 'workspace', draft: 'unsent', history: ['one'], unicode: false });
  await saveUiState(state, file); assert.deepEqual(await loadUiState(file), state); assert.equal(normalizeUiState({ bannerMode: 'invalid' }).bannerMode, 'full');
  await fs.rm(directory, { recursive: true, force: true });
});

test('parses real Git telemetry without fabricating values', () => {
  assert.deepEqual(parseGitSnapshot('main', ' M ui/src/app.tsx\n?? new.txt', '12\t4\tui/src/app.tsx'), { branch: 'main', modified: 1, untracked: 1, additions: 12, deletions: 4 });
});

test('detects Termux runtime metadata without requiring Android services', async () => {
  const previousTermux = process.env.TERMUX_VERSION; const previousPrefix = process.env.PREFIX;
  process.env.TERMUX_VERSION = '0.118.0'; process.env.PREFIX = '/data/data/com.termux/files/usr';
  const snapshot = await loadSnapshot(process.cwd()); assert.equal(snapshot.platform, 'termux'); assert.equal(snapshot.prefix, '/data/data/com.termux/files/usr');
  if (previousTermux === undefined) delete process.env.TERMUX_VERSION; else process.env.TERMUX_VERSION = previousTermux;
  if (previousPrefix === undefined) delete process.env.PREFIX; else process.env.PREFIX = previousPrefix;
});

test('loads a validated runtime snapshot with live telemetry', async () => {
  const snapshot = await loadSnapshot(process.cwd()); assert.ok(snapshot.workspace.length > 0); assert.equal(snapshot.messages.length, 1);
  assert.equal(snapshot.messages[0]?.role, 'utharness'); assert.ok(snapshot.git.branch.length > 0); assert.ok(snapshot.platform.length > 0); assert.ok(snapshot.termuxApi.length > 0);
});
