import React from 'react';
import { Box, Text } from 'ink';
import stringWidth from 'string-width';
import wrapAnsi from 'wrap-ansi';
import type { ColorMode, Message, OverlayKind, PaletteItem, RuntimeSnapshot, ToolCard } from './types.js';
import { icon, spinnerFrames } from './tui/icons.js';
import { palette, tone } from './tui/theme.js';

export { getBreakpoint, getTermuxBreakpoint } from './tui/responsive.js';
export { getColorMode, palette } from './tui/theme.js';

export { ResponsiveBanner, PersistentHeader, BannerRenderer, BannerLayout, TerminalCapabilities, IconRegistry, TerminalResizeManager, bannerHeight, bannerTier, letterColors } from './tui/banner.js';

export function StartupTips({ colorMode }: { colorMode: ColorMode }) {
  return <Box flexDirection="column" marginBottom={1}><Text color={tone(palette.warning, colorMode)} bold>Tips for getting started:</Text><Text color={tone(palette.text, colorMode)}>1. Ask questions, edit files, or run commands.</Text><Text color={tone(palette.text, colorMode)}>2. Use <Text color={tone(palette.primary, colorMode)}>@path/to/file</Text> to add context.</Text><Text color={tone(palette.text, colorMode)}>3. Type <Text color={tone(palette.accent, colorMode)}>/help</Text> or press F1.</Text></Box>;
}

export function WorkspaceWarning({ colorMode }: { colorMode: ColorMode }) {
  return <Box borderStyle="round" borderColor={tone(palette.warning, colorMode)} paddingX={1} marginBottom={1}><Text color={tone(palette.warning, colorMode)}>{icon('warning')}  You are not in a project-specific directory.{String.fromCharCode(10)}   Open a workspace folder and run utharness there for best results.</Text></Box>;
}

function toolTone(tool: ToolCard, mode: ColorMode) { if (tool.state === 'error') return tone(palette.error, mode); if (tool.state === 'approval') return tone(palette.warning, mode); if (tool.state === 'running') return tone(palette.primary, mode); if (tool.state === 'waiting') return tone(palette.muted, mode); return tone(palette.success, mode); }
export function ToolCardView({ tool, width, colorMode, tick = 0 }: { tool: ToolCard; width: number; colorMode: ColorMode; tick?: number }) {
  const status = tool.state === 'running' ? `${spinnerFrames[tick % spinnerFrames.length]} Running` : tool.state === 'approval' ? '! Approval required' : tool.state === 'error' ? '✗ Error' : tool.state === 'waiting' ? '○ Waiting' : '✓ Completed';
  return <Box borderStyle="round" borderColor={tone(tool.state === 'running' ? palette.borderFocus : palette.border, colorMode)} paddingX={1} width={Math.max(22, Math.min(width, 82))} flexDirection="column"><Box justifyContent="space-between"><Text color={tone(palette.agent, colorMode)} bold>{tool.kind ?? 'TOOL'}  {tool.name}</Text><Text color={toolTone(tool, colorMode)}>{status}</Text></Box>{tool.state !== 'completed' && tool.detail ? <Text color={tone(palette.text, colorMode)}>{tool.detail}</Text> : null}<Text color={tone(palette.muted, colorMode)}>{tool.metric}{tool.elapsed ? `  ${tool.elapsed}` : ''}</Text></Box>;
}

const roleMeta = (role: Message['role']) => ({
  utharness: ['UTHARNESS', palette.agent, '◉'], you: ['YOU', palette.primary, '○'], system: ['SYSTEM', palette.warning, '!'],
  agent: ['AGENT', palette.agent, '◆'], tool: ['TOOL', palette.tool, '⚙'], memory: ['MEMORY', palette.accent, '◫'], error: ['ERROR', palette.error, '✗']
} as const)[role];
export function MessageRow({ message, width, colorMode, tick }: { message: Message; width: number; colorMode: ColorMode; tick: number }) {
  const [name, color, marker] = roleMeta(message.role);
  const bodyWidth = Math.max(16, width - 5);
  const body = stringWidth(message.text) > bodyWidth ? wrapAnsi(message.text, bodyWidth, { hard: true }) : message.text;
  return <Box flexDirection="column" marginBottom={1} width={width}><Box justifyContent="space-between"><Text color={tone(color, colorMode)} bold>{marker}  {name}</Text><Text color={tone(palette.muted, colorMode)}>{message.time}</Text></Box><Box paddingLeft={4}><Text color={tone(palette.text, colorMode)}>{body}</Text></Box>{message.tool ? <Box paddingLeft={4} marginTop={1}><ToolCardView tool={message.tool} width={bodyWidth} colorMode={colorMode} tick={tick} /></Box> : null}</Box>;
}

export function StatusBar({ snapshot, width, colorMode }: { snapshot: RuntimeSnapshot; width: number; colorMode: ColorMode }) {
  const git = `${snapshot.git.branch}${snapshot.git.modified || snapshot.git.untracked ? ` M${snapshot.git.modified} ?${snapshot.git.untracked}` : ''}${snapshot.git.additions || snapshot.git.deletions ? ` +${snapshot.git.additions} -${snapshot.git.deletions}` : ''}`;
  const segments = [snapshot.workspace, snapshot.permission, `${snapshot.provider}/${snapshot.model}`, git, snapshot.context, snapshot.activeAgents ? `Agents ${snapshot.activeAgents}` : 'Agents 0'];
  if (width < 40) return <Text color={tone(palette.primary, colorMode)} wrap="truncate-end">{snapshot.provider}/{snapshot.model} · {git}</Text>;
  return <Box borderStyle="single" borderColor={tone(palette.border, colorMode)} paddingX={1} width={Math.max(20, width)}><Text color={tone(palette.primary, colorMode)} wrap="truncate-end">{segments.join('  │  ')}</Text></Box>;
}

export function Navigation({ colorMode, width }: { colorMode: ColorMode; width: number }) {
  const items = [['chat', 'Chat'], ['task', 'Tasks'], ['file', 'Files'], ['agent', 'Agents'], ['skill', 'Skills'], ['memory', 'Memory'], ['job', 'Jobs'], ['model', 'Models'], ['tool', 'Tools'], ['logs', 'Logs'], ['settings', 'Settings']] as const;
  return <Box width={width} borderStyle="single" borderColor={tone(palette.border, colorMode)} flexDirection="column" paddingX={1}><Text bold color={tone(palette.primary, colorMode)}>WORKSPACE</Text>{items.map(([glyph, label], index) => <Text key={label} color={tone(index === 0 ? palette.borderFocus : palette.muted, colorMode)}>{index === 0 ? '›' : ' '} {icon(glyph)} {label}</Text>)}</Box>;
}

export function Inspector({ snapshot, colorMode, width }: { snapshot: RuntimeSnapshot; colorMode: ColorMode; width: number }) {
  return <Box width={width} borderStyle="single" borderColor={tone(palette.border, colorMode)} flexDirection="column" paddingX={1}><Text color={tone(palette.accent, colorMode)} bold>TASK  CONTEXT  AGENTS</Text><Text color={tone(palette.muted, colorMode)}>Active task</Text><Text color={tone(palette.text, colorMode)}>Idle — ready for input</Text><Text color={tone(palette.muted, colorMode)}>Provider / model</Text><Text color={tone(palette.text, colorMode)} wrap="truncate-end">{snapshot.provider}/{snapshot.model}</Text><Text color={tone(palette.muted, colorMode)}>Git</Text><Text color={tone(palette.success, colorMode)}>{snapshot.git.branch} · {snapshot.git.modified + snapshot.git.untracked} changes</Text></Box>;
}

export function Overlay({ kind, items, selected, query, width, colorMode }: { kind: Exclude<OverlayKind, null>; items: PaletteItem[]; selected: number; query: string; width: number; colorMode: ColorMode }) {
  const title = kind.toUpperCase();
  return <Box borderStyle="round" borderColor={tone(palette.accent, colorMode)} paddingX={1} width={Math.max(28, Math.min(width, 78))} flexDirection="column"><Box justifyContent="space-between"><Text color={tone(palette.accent, colorMode)} bold>{title}</Text><Text color={tone(palette.muted, colorMode)}>Esc close</Text></Box>{query ? <Text color={tone(palette.primary, colorMode)}>⌕ {query}</Text> : null}{items.slice(0, 12).map((item, index) => <Box key={item.id} justifyContent="space-between"><Text color={tone(index === selected ? palette.borderFocus : palette.text, colorMode)}>{index === selected ? '› ' : '  '}{item.label}  <Text color={tone(palette.muted, colorMode)}>{item.description}</Text></Text><Text color={tone(palette.muted, colorMode)}>{item.shortcut ?? ''}</Text></Box>)}{items.length === 0 ? <Text color={tone(palette.muted, colorMode)}>No matching items</Text> : null}<Text color={tone(palette.muted, colorMode)}>↑/↓ select · Enter open · Esc close</Text></Box>;
}
