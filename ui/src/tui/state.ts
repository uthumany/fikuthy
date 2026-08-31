import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import type { PersistedUiState } from '../types.js';
export const defaultUiState: PersistedUiState = { version: 1, bannerMode: (['full', 'compact', 'minimal', 'hide'].includes(process.env.UTHARNESS_BANNER ?? '') ? process.env.UTHARNESS_BANNER : 'full') as PersistedUiState['bannerMode'], layoutMode: 'focus', theme: 'Utharness Carbon', draft: '', history: [], reducedMotion: false, unicode: process.env.UTHARNESS_ASCII !== '1', iconMode: (['nerd', 'unicode', 'ascii'].includes(process.env.UTHARNESS_ICONS ?? '') ? process.env.UTHARNESS_ICONS : (process.env.UTHARNESS_ASCII === '1' ? 'ascii' : 'unicode')) as PersistedUiState['iconMode'] };
export function uiStatePath(env: NodeJS.ProcessEnv = process.env): string { return path.join(env.XDG_STATE_HOME ?? path.join(os.homedir(), '.local', 'state'), 'utharness', 'ui.json'); }
export function normalizeUiState(value: unknown): PersistedUiState {
  if (!value || typeof value !== 'object') return { ...defaultUiState };
  const input = value as Partial<PersistedUiState>;
  return { ...defaultUiState,
    bannerMode: ['full', 'compact', 'minimal', 'hide'].includes(input.bannerMode ?? '') ? input.bannerMode! : defaultUiState.bannerMode,
    layoutMode: ['focus', 'workspace'].includes(input.layoutMode ?? '') ? input.layoutMode! : defaultUiState.layoutMode,
    theme: typeof input.theme === 'string' ? input.theme : defaultUiState.theme, draft: typeof input.draft === 'string' ? input.draft : '',
    history: Array.isArray(input.history) ? input.history.filter((item): item is string => typeof item === 'string').slice(-50) : [],
    reducedMotion: Boolean(input.reducedMotion), unicode: input.unicode !== false,
    iconMode: ['nerd', 'unicode', 'ascii'].includes(input.iconMode ?? '') ? input.iconMode! : defaultUiState.iconMode,
    selectedModel: typeof input.selectedModel === 'string' ? input.selectedModel : undefined,
    selectedProvider: typeof input.selectedProvider === 'string' ? input.selectedProvider : undefined };
}
export async function loadUiState(file = uiStatePath()): Promise<PersistedUiState> { try { return normalizeUiState(JSON.parse(await fs.readFile(file, 'utf8'))); } catch { return { ...defaultUiState }; } }
export async function saveUiState(state: PersistedUiState, file = uiStatePath()): Promise<void> { await fs.mkdir(path.dirname(file), { recursive: true, mode: 0o700 }); const temporary = `${file}.${process.pid}.tmp`; await fs.writeFile(temporary, `${JSON.stringify(state, null, 2)}\n`, { mode: 0o600 }); await fs.rename(temporary, file); }
