import type { ColorMode } from '../types.js';

export const palette = {
  background: '#0A0D12', surface: '#10151D', surfaceActive: '#151D27', border: '#263241', borderFocus: '#41D9FF',
  primary: '#49D7FF', accent: '#A970FF', success: '#4DDB8A', warning: '#F5C542', error: '#FF5D73', text: '#E8EDF3',
  muted: '#8190A3', agent: '#C084FC', tool: '#58C7FA'
} as const;
export const bannerGradient = ['#FFD43B', '#FFBE55', '#FFA15A', '#FF8267', '#FF6B81'];
export function getColorMode(env: NodeJS.ProcessEnv = process.env): ColorMode {
  if (env.NO_COLOR || env.UTHARNESS_ASCII === '1') return 'mono';
  if (env.UTHARNESS_COLOR === 'truecolor' || env.COLORTERM?.includes('truecolor')) return 'truecolor';
  if (env.UTHARNESS_COLOR === 'ansi256' || env.TERM?.includes('256color')) return 'ansi256';
  return 'ansi16';
}
export function tone(color: string, mode: ColorMode): string | undefined { return mode === 'mono' ? undefined : color; }
