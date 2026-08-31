import chokidar, { type FSWatcher } from 'chokidar';
import { execa } from 'execa';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { z } from 'zod';
import { runtimeBinary } from './runtime-binary.js';
import type { Message, RuntimeSnapshot, ToolCard } from './types.js';

const runtimeSchema = z.object({
  workspace: z.string(),
  permission: z.string(),
  provider: z.string(),
  model: z.string(),
  context: z.string(),
  network: z.string(),
  projectSpecific: z.boolean(),
  platform: z.string(),
  androidVersion: z.string(),
  prefix: z.string(),
  termuxApi: z.string(),
  storage: z.string(),
  messages: z.array(z.object({
    id: z.string(),
    role: z.enum(['utharness', 'you', 'system', 'agent', 'tool', 'memory', 'error']),
    text: z.string(),
    time: z.string(),
    tool: z.object({
      id: z.string(),
      name: z.string(),
      icon: z.string(),
      state: z.enum(['waiting', 'running', 'completed', 'error', 'approval']),
      result: z.string(),
      metric: z.string(),
      elapsed: z.string()
    }).optional()
  })),
  git: z.object({ branch: z.string(), modified: z.number(), untracked: z.number(), additions: z.number(), deletions: z.number() }),
  activeAgents: z.number()
});

const now = () => new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
const id = () => `${Date.now()}-${Math.random().toString(16).slice(2)}`;

const initialMessages = (): Message[] => [
  { id: id(), role: 'utharness', text: 'Ready. Ask a question, reference @files, or run a slash command.', time: now() }
];

async function commandOutput(command: string, args: string[], cwd: string): Promise<string> {
  const result = await execa(command, args, { cwd, reject: false, timeout: 2_500 });
  return result.stdout.trim();
}

export async function loadSnapshot(cwd = process.cwd()): Promise<RuntimeSnapshot> {
  const home = os.homedir();
  const [branch, gitRoot, porcelain, diffStats, androidVersion, apiCommand] = await Promise.all([
    commandOutput('git', ['branch', '--show-current'], cwd),
    commandOutput('git', ['rev-parse', '--show-toplevel'], cwd),
    commandOutput('git', ['status', '--porcelain'], cwd),
    commandOutput('git', ['diff', '--numstat'], cwd),
    commandOutput('getprop', ['ro.build.version.release'], cwd),
    commandOutput('sh', ['-lc', 'command -v termux-battery-status'], cwd)
  ]);
  const isTermux = Boolean(process.env.TERMUX_VERSION || process.env.PREFIX?.includes('com.termux'));
  const workspacePath = gitRoot || cwd;
  const workspace = workspacePath.startsWith(home) ? `~${workspacePath.slice(home.length)}` : workspacePath;
  const provider = process.env.UTHARNESS_PROVIDER ?? (process.env.OPENROUTER_API_KEY ? 'openrouter' : 'offline');
  const storagePath = path.join(home, 'storage');
  let storage = 'sandbox';
  try {
    await fs.access(storagePath);
    storage = 'shared storage linked';
  } catch {
    // Shared storage is optional on Termux.
  }
  const snapshot = {
    workspace,
    permission: process.env.UTHARNESS_PERMISSION ?? 'offline (default deny)',
    provider,
    model: process.env.UTHARNESS_MODEL ?? 'gpt-4o-mini',
    context: process.env.UTHARNESS_CONTEXT ?? '128K context left',
    network: process.env.OPENROUTER_API_KEY ? 'connected' : 'offline',
    projectSpecific: Boolean(gitRoot),
    platform: isTermux ? 'termux' : process.platform,
    androidVersion: androidVersion || 'n/a',
    prefix: process.env.PREFIX ?? '',
    termuxApi: apiCommand ? 'available' : 'optional/missing',
    storage,
    git: parseGitSnapshot(branch, porcelain, diffStats),
    activeAgents: Number.parseInt(process.env.UTHARNESS_ACTIVE_AGENTS ?? '0', 10) || 0,
    messages: initialMessages()
  } satisfies RuntimeSnapshot;
  return runtimeSchema.parse(snapshot);
}

export function parseGitSnapshot(branch: string, porcelain: string, diffStats: string) {
  const statusLines = porcelain.split(/\r?\n/).filter(Boolean);
  const untracked = statusLines.filter(line => line.startsWith('??')).length;
  const modified = statusLines.length - untracked;
  let additions = 0;
  let deletions = 0;
  for (const line of diffStats.split(/\r?\n/).filter(Boolean)) {
    const [added, removed] = line.split(/\s+/);
    if (added !== '-') additions += Number.parseInt(added ?? '0', 10) || 0;
    if (removed !== '-') deletions += Number.parseInt(removed ?? '0', 10) || 0;
  }
  return { branch: branch || 'no-branch', modified, untracked, additions, deletions };
}

export async function runSkillCommand(args: string[], cwd = process.cwd()): Promise<string> {
  const binary = runtimeBinary();
  try {
    await fs.access(binary);
    const result = await execa(binary, ['skills', ...args], { cwd, reject: false, timeout: 8_000 });
    return (result.stdout || result.stderr).trim() || 'Skill Engine returned no output.';
  } catch (error) {
    return `Skill Engine unavailable: ${error instanceof Error ? error.message : String(error)}`;
  }
}

export async function submitPrompt(prompt: string, cwd = process.cwd()): Promise<{ text: string; tool?: ToolCard }> {
  const binary = runtimeBinary();
  let commandResult = '';
  try {
    await fs.access(binary);
    const result = await execa(binary, ['chat', prompt], { cwd, reject: false, timeout: 10_000 });
    commandResult = result.stdout.trim();
  } catch {
    commandResult = '';
  }

  return {
    text: commandResult || `I received: “${prompt}”\n\nI can inspect files, review Git state, and run bounded SAFE tasks from this terminal.`,
    tool: commandResult ? undefined : { id: id(), kind: 'AGENT', name: 'agent_response', icon: '✦', state: 'completed', result: 'Completed', metric: 'offline planner', elapsed: '42ms' }
  };
}

export function watchRuntime(cwd: string, onChange: () => void): FSWatcher {
  const watcher = chokidar.watch([path.join(cwd, 'UTHARNESS.md'), path.join(cwd, '.git', 'HEAD')], {
    ignoreInitial: true,
    persistent: false
  });
  watcher.on('all', (_event, changedPath) => {
    if (changedPath.endsWith('UTHARNESS.md') || changedPath.endsWith(path.join('.git', 'HEAD'))) onChange();
  });
  return watcher;
}
