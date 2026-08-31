export type Breakpoint = 'tiny' | 'compact' | 'standard' | 'wide';
export type ColorMode = 'truecolor' | 'ansi256' | 'ansi16' | 'mono';
export type BannerMode = 'full' | 'compact' | 'minimal' | 'hide';
export type IconMode = 'nerd' | 'unicode' | 'ascii';
export type LayoutMode = 'focus' | 'workspace';
export type OverlayKind = 'commands' | 'models' | 'files' | 'agents' | 'tasks' | 'memory' | 'jobs' | 'logs' | 'help' | 'context' | null;
export type Role = 'utharness' | 'you' | 'system' | 'agent' | 'tool' | 'memory' | 'error';
export type ToolState = 'waiting' | 'running' | 'completed' | 'error' | 'approval';
export type ToolKind = 'PLAN' | 'READ' | 'WRITE' | 'EDIT' | 'DIFF' | 'SHELL' | 'TEST' | 'BUILD' | 'GIT' | 'BROWSER' | 'SEARCH' | 'HTTP' | 'AGENT' | 'SKILL' | 'MCP' | 'MEMORY' | 'ERROR';

export type ToolCard = { id: string; kind?: ToolKind; name: string; icon: string; state: ToolState; result: string; metric: string; elapsed: string; detail?: string };
export type Message = { id: string; role: Role; text: string; time: string; tool?: ToolCard };
export type GitSnapshot = { branch: string; modified: number; untracked: number; additions: number; deletions: number };
export type RuntimeSnapshot = {
  workspace: string; permission: string; provider: string; model: string; context: string; network: string;
  projectSpecific: boolean; platform: string; androidVersion: string; prefix: string; termuxApi: string; storage: string;
  git: GitSnapshot; activeAgents: number; messages: Message[];
};
export type PaletteItem = { id: string; label: string; description: string; shortcut?: string; overlay?: Exclude<OverlayKind, null>; command?: string };
export type PersistedUiState = {
  version: 1; bannerMode: BannerMode; layoutMode: LayoutMode; theme: string; draft: string; history: string[];
  reducedMotion: boolean; unicode: boolean; iconMode: IconMode; selectedModel?: string; selectedProvider?: string;
};
