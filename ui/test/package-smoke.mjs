import assert from 'node:assert/strict';
import {cpSync, mkdirSync, mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';

const root = mkdtempSync(join(tmpdir(), 'utharness-ui-package-'));
const isolatedUi = join(root, 'ui');
mkdirSync(isolatedUi, {recursive: true});
cpSync(new URL('../dist', import.meta.url), join(isolatedUi, 'dist'), {recursive: true});
cpSync(new URL('../package.json', import.meta.url), join(isolatedUi, 'package.json'));

const result = spawnSync(process.execPath, [join(isolatedUi, 'dist/index.js')], {
  cwd: root,
  encoding: 'utf8',
  timeout: 5000,
});
const combined = `${result.stdout}\n${result.stderr}`;

// A pipe has no terminal raw mode, so Ink may stop after resolving and
// rendering the application. The package contract under test is that startup
// reaches Ink without relying on a parent node_modules tree or typeless ESM.
assert.match(combined, /utharness-agent|Raw mode is not supported/);
assert.doesNotMatch(combined, /ERR_MODULE_NOT_FOUND|MODULE_TYPELESS_PACKAGE_JSON/);
assert.doesNotMatch(combined, /Dynamic require of/);
console.log('isolated UI package resolved all runtime modules');
