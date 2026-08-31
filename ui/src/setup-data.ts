export type SetupMode = 'quick' | 'full' | 'developer' | 'local_ai' | 'custom' | 'blank' | 'import' | 'exit';
export type AuthMethod = 'api_key' | 'oauth' | 'environment' | 'skip';
export type ProviderOption = { id: string; label: string; description: string; model: string; key?: string };
export type ToolOption = { id: string; label: string; description: string; risk: 'safe' | 'ask' };
export type ScanComponent = { id: string; label: string; state: 'AVAILABLE' | 'MISSING' | 'BROKEN' | 'OPTIONAL'; required: boolean; version?: string; installHint?: string };
export type EnvironmentReport = { os: string; architecture: string; shell: string; terminal: string; packageManager?: string; components: ScanComponent[] };

export const modes: Array<{ id: SetupMode; label: string; description: string }> = [
  { id: 'quick', label: 'Quick Start', description: 'provider, model, and safe defaults' },
  { id: 'full', label: 'Full Setup', description: 'choose every runtime capability' },
  { id: 'developer', label: 'Developer Setup', description: 'coding, Git, terminal, skills, memory, and agents' },
  { id: 'local_ai', label: 'Local AI Setup', description: 'Ollama with no hosted API key' },
  { id: 'custom', label: 'Custom Provider', description: 'OpenAI-compatible URL, key, and model' },
  { id: 'blank', label: 'Blank Slate', description: 'offline planner and workspace reading only' },
  { id: 'import', label: 'Import Configuration', description: 'validate an existing utharness.json' },
  { id: 'exit', label: 'Exit', description: 'leave without changing configuration' }
];

export const providers: ProviderOption[] = [
  { id: 'openrouter', label: 'OpenRouter', description: 'OpenAI-compatible model aggregator', model: 'openrouter/free', key: 'OPENROUTER_API_KEY' },
  { id: 'openai', label: 'OpenAI', description: 'direct OpenAI API', model: 'gpt-4o-mini', key: 'OPENAI_API_KEY' },
  { id: 'groq', label: 'Groq', description: 'low-latency hosted inference', model: 'llama-3.3-70b-versatile', key: 'GROQ_API_KEY' },
  { id: 'together', label: 'Together AI', description: 'open-model inference', model: 'meta-llama/Llama-3.3-70B-Instruct-Turbo', key: 'TOGETHER_API_KEY' },
  { id: 'deepseek', label: 'DeepSeek', description: 'chat and coding models', model: 'deepseek-chat', key: 'DEEPSEEK_API_KEY' },
  { id: 'fireworks', label: 'Fireworks AI', description: 'OpenAI-compatible model API', model: 'accounts/fireworks/models/llama-v3p3-70b-instruct', key: 'FIREWORKS_API_KEY' },
  { id: 'nvidia', label: 'NVIDIA NIM', description: 'hosted Nemotron models', model: 'nvidia/nemotron-3-super-120b-a12b', key: 'NVIDIA_API_KEY' },
  { id: 'ollama', label: 'Ollama', description: 'local model server; no API key', model: 'qwen2.5-coder:7b' },
  { id: 'custom', label: 'Custom endpoint', description: 'OpenAI-compatible endpoint', model: 'default', key: 'UTHARNESS_API_KEY' }
];

export const authMethods: Array<{ id: AuthMethod; label: string; description: string }> = [
  { id: 'api_key', label: 'API Key', description: 'masked input; private secrets.env' },
  { id: 'oauth', label: 'OAuth', description: 'when supported by the provider adapter' },
  { id: 'environment', label: 'Environment Variable', description: 'reuse an existing provider variable' },
  { id: 'skip', label: 'Skip', description: 'save incomplete setup and validate later' }
];

export const tools: ToolOption[] = [
  { id: 'workspace_read', label: 'Files & repository search', description: 'read, list, and search files', risk: 'safe' },
  { id: 'git_inspection', label: 'Git inspection', description: 'status, diff, and history', risk: 'safe' },
  { id: 'terminal', label: 'Terminal & processes', description: 'bounded commands with approval', risk: 'ask' },
  { id: 'file_write', label: 'File editing', description: 'write and patch with approval', risk: 'ask' },
  { id: 'skills', label: 'Skills & MCP', description: 'validated extension registries', risk: 'safe' },
  { id: 'memory', label: 'Persistent memory', description: 'project notes across sessions', risk: 'safe' },
  { id: 'session_search', label: 'Sessions', description: 'search local conversation history', risk: 'safe' },
  { id: 'task_planning', label: 'Agents & planning', description: 'checkpoints and bounded plans', risk: 'safe' }
];

export const recommendedTools = ['workspace_read', 'git_inspection', 'skills', 'memory'];
export const developerTools = tools.map(tool => tool.id);
export function progress(completed: number, total: number) { return total <= 0 ? 100 : Math.max(0, Math.min(100, Math.round(completed / total * 100))); }
