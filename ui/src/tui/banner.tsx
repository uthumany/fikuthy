import React from 'react';
import { Box, Text } from 'ink';
import type { BannerMode, ColorMode, IconMode } from '../types.js';
import { tone } from './theme.js';

export type BannerTier = 'full' | 'compressed' | 'wrapped' | 'compact' | 'minimal' | 'hide';

export const letterColors = ['#B44CFF', '#20D6F4', '#55DB24', '#FFD21F', '#FF8A16', '#FF3D4F', '#3478F6', '#B44CFF', '#B44CFF'] as const;
const glyphs: Record<string, string[]> = {
  U: ['█ █', '█ █', '█ █', '█ █', '███'], T: ['███', ' █ ', ' █ ', ' █ ', ' █ '],
  H: ['█ █', '█ █', '███', '█ █', '█ █'], A: [' █ ', '█ █', '███', '█ █', '█ █'],
  R: ['██ ', '█ █', '██ ', '█ █', '█ █'], N: ['█ █', '███', '███', '█ █', '█ █'],
  E: ['███', '█  ', '██ ', '█  ', '███'], S: [' ██', '█  ', ' █ ', '  █', '██ ']
};
const word = [...'UTHARNESS'];

export function detectTerminalCapabilities(env: NodeJS.ProcessEnv = process.env) {
  const term = env.TERM ?? '';
  return {
    unicode: env.UTHARNESS_ASCII !== '1' && term !== 'dumb',
    nerdFonts: env.UTHARNESS_ICONS === 'nerd' || env.NERD_FONT === '1',
    color: !env.NO_COLOR && term !== 'dumb'
  };
}

export function bannerTier(width: number, rows: number, mode: BannerMode = 'full'): BannerTier {
  if (mode === 'hide') return 'hide';
  if (width < 40 || rows < 12) return 'minimal';
  if (mode === 'minimal') return 'minimal';
  if (mode === 'compact' || width < 60 || rows < 18) return 'compact';
  if (width < 90 || rows < 24) return 'wrapped';
  if (width < 120 || rows < 30) return 'compressed';
  return 'full';
}

export function bannerHeight(tier: BannerTier): number {
  return ({ full: 10, compressed: 8, wrapped: 9, compact: 4, minimal: 2, hide: 0 })[tier];
}

const iconSets: Record<IconMode, Record<string, string>> = {
  nerd: { agents: '󰚩', models: '󰆧', skills: '', mcp: '󰘬', memory: '', tools: '󰒓', terminal: '' },
  unicode: { agents: '◉', models: '◇', skills: '</>', mcp: '⎇', memory: '▤', tools: '⚒', terminal: '>_' },
  ascii: { agents: '[A]', models: '[M]', skills: '[S]', mcp: '[C]', memory: '[D]', tools: '[T]', terminal: '>_' }
};

export function resolveIconMode(requested: IconMode, env: NodeJS.ProcessEnv = process.env): IconMode {
  if (requested === 'nerd' && !detectTerminalCapabilities(env).nerdFonts) return 'unicode';
  if (!detectTerminalCapabilities(env).unicode) return 'ascii';
  return requested;
}

const nav = [
  ['agents', 'AGENTS', letterColors[0]], ['models', 'MODELS', letterColors[1]], ['skills', 'SKILLS', letterColors[2]],
  ['mcp', 'MCP', letterColors[3]], ['memory', 'MEMORY', letterColors[4]], ['tools', 'TOOLS', letterColors[5]], ['terminal', 'TERMINAL', letterColors[6]]
] as const;

function Wordmark({ rows, colorMode, doubled = false }: { rows: number[]; colorMode: ColorMode; doubled?: boolean }) {
  return <Box flexDirection="column">{rows.map(row => <Box key={row}>{word.map((letter, index) => {
    const text = glyphs[letter]![row]!;
    return <Text key={`${letter}-${index}`} color={tone(letterColors[index]!, colorMode)} bold>{doubled ? text.replaceAll('█', '██') : text} </Text>;
  })}</Box>)}</Box>;
}

function TerminalBlock({ colorMode }: { colorMode: ColorMode }) {
  const purple = tone(letterColors[0], colorMode); const green = tone(letterColors[2], colorMode);
  return <Box flexDirection="column" marginRight={2}>
    <Text color={purple}>╔══════════╗</Text><Text color={purple}>║          ║</Text>
    <Text color={purple}>║ <Text color={green} bold>&gt;_</Text>       ║</Text><Text color={purple}>║          ║</Text><Text color={purple}>╚══════════╝</Text>
  </Box>;
}

function StatusBlocks({ tier, colorMode, icons }: { tier: BannerTier; colorMode: ColorMode; icons: IconMode }) {
  const set = iconSets[icons];
  const item = ([id, label, color]: typeof nav[number]) => <Text key={id} color={tone(color, colorMode)} bold>{set[id]} {tier === 'compact' ? label[0] : label}</Text>;
  if (tier === 'wrapped') return <Box flexDirection="column"><Box gap={2}>{nav.slice(0, 4).map(item)}</Box><Box gap={2}>{nav.slice(4).map(item)}</Box></Box>;
  return <Box gap={1}>{nav.map(item)}</Box>;
}

export function ResponsiveBanner({ width, rows, mode, colorMode, iconMode }: { width: number; rows: number; mode: BannerMode; colorMode: ColorMode; iconMode: IconMode }) {
  const tier = bannerTier(width, rows, mode);
  if (tier === 'hide') return null;
  const icons = resolveIconMode(iconMode);
  if (tier === 'minimal') return <Box justifyContent="space-between"><Text bold>{word.map((letter, index) => <Text key={`${letter}-${index}`} color={tone(letterColors[index]!, colorMode)}>{letter}</Text>)} <Text color={tone(letterColors[2], colorMode)}>&gt;_</Text></Text><Text dimColor>F1 help</Text></Box>;
  if (tier === 'compact') return <Box flexDirection="column"><Text color={tone(letterColors[0], colorMode)}>┌─ <Text bold>UTHARNESS</Text> &gt;_ ─┐</Text><StatusBlocks tier={tier} colorMode={colorMode} icons={icons} /><Text dimColor>AUTONOMOUS AGENT HARNESS</Text></Box>;
  const separator = '╌'.repeat(Math.max(20, Math.min(width - 2, tier === 'full' ? 104 : 82)));
  return <Box flexDirection="column">
    <Text dimColor>{separator}</Text>
    <Box>{tier === 'full' ? <TerminalBlock colorMode={colorMode} /> : null}<Wordmark rows={tier === 'compressed' ? [0, 2, 4] : [0, 1, 2, 3, 4]} colorMode={colorMode} doubled={tier === 'full'} /></Box>
    <Text dimColor>{separator}</Text>
    <StatusBlocks tier={tier} colorMode={colorMode} icons={icons} />
    <Text color={tone(letterColors[2], colorMode)} bold>&gt; <Text color={tone('#E8EDF3', colorMode)}>AUTONOMOUS AI AGENT TERMINAL HARNESS</Text> &lt;</Text>
  </Box>;
}

export const PersistentHeader = ResponsiveBanner;
export const BannerRenderer = ResponsiveBanner;
export const BannerLayout = { tier: bannerTier, height: bannerHeight };
export const TerminalCapabilities = { detect: detectTerminalCapabilities };
export const IconRegistry = { resolveMode: resolveIconMode, sets: iconSets };
export const TerminalResizeManager = { debounceMs: 75 };
