import React, { useEffect, useMemo, useState } from 'react';
import { Box, Text, useApp, useInput, useStdout } from 'ink';
import { execa } from 'execa';
import { authMethods, developerTools, modes, progress, providers, recommendedTools, tools, type AuthMethod, type EnvironmentReport, type SetupMode } from './setup-data.js';
import { runtimeBinary } from './runtime-binary.js';

type Stage = 'scan' | 'mode' | 'provider' | 'custom_url' | 'auth' | 'secret' | 'model' | 'tools' | 'import' | 'review' | 'saving' | 'done' | 'error';
const cyan = '#25d9cb', purple = '#db79ff', green = '#69d26f', yellow = '#ffbe55', red = '#ff6b6b', muted = '#8b93a7';

function List({ items, selected, marked }: { items: Array<{ id: string; label: string; description: string }>; selected: number; marked?: Set<string> }) {
  return <Box flexDirection="column">{items.map((item, index) => <Text key={item.id} color={index === selected ? cyan : undefined} wrap="truncate-end">{index === selected ? '› ' : '  '}{marked ? `[${marked.has(item.id) ? '●' : ' '}]` : `${index + 1}.`} <Text bold={index === selected}>{item.label}</Text> <Text color={muted}>— {item.description}</Text></Text>)}</Box>;
}
function Progress({ completed, total, label }: { completed: number; total: number; label: string }) {
  const value = progress(completed, total), filled = Math.round(value / 10);
  return <Text color={cyan}>⋘ {label} ⋙  <Text color={green}>{'█'.repeat(filled)}</Text><Text color={muted}>{'▒'.repeat(10 - filled)}</Text> {value}%</Text>;
}

export function SetupApp() {
  const { exit } = useApp(); const { stdout } = useStdout();
  const [stage, setStage] = useState<Stage>('scan'), [selected, setSelected] = useState(0);
  const [mode, setMode] = useState<SetupMode>('quick'), [provider, setProvider] = useState('openrouter');
  const [auth, setAuth] = useState<AuthMethod>('environment'), [secret, setSecret] = useState('');
  const [model, setModel] = useState('openrouter/free'), [modelOptions, setModelOptions] = useState<string[]>([]);
  const [customUrl, setCustomUrl] = useState('http://127.0.0.1:8000/v1'), [importPath, setImportPath] = useState('');
  const [enabled, setEnabled] = useState(new Set(recommendedTools)), [report, setReport] = useState<EnvironmentReport | null>(null);
  const [error, setError] = useState(''); const columns = Math.max(30, stdout.columns ?? 80), compact = columns < 70;
  const providerInfo = providers.find(item => item.id === provider) ?? providers[0]!;
  const selectedTools = useMemo(() => tools.filter(tool => enabled.has(tool.id)), [enabled]);
  const list = stage === 'mode' ? modes : stage === 'provider' ? providers : stage === 'auth' ? authMethods : stage === 'tools' ? tools : stage === 'model' ? modelOptions.map(id => ({ id, label: id, description: id === model ? 'recommended default' : 'available model' })) : [];

  useEffect(() => { void execa(runtimeBinary(), ['setup', '--scan'], { cwd: process.cwd(), reject: false }).then(result => {
    if (result.exitCode !== 0) { setError(result.stderr || 'Environment scan failed'); setStage('error'); return; }
    try { setReport(JSON.parse(result.stdout) as EnvironmentReport); setStage('mode'); } catch { setError('Environment scanner returned invalid data'); setStage('error'); }
  }); }, []);

  const loadModels = async (method: AuthMethod, selectedProvider = provider, info = providerInfo) => {
    if (method === 'oauth') { setError(`${info.label} OAuth is not exposed by this provider adapter. Choose API Key or Environment Variable.`); setStage('error'); return; }
    if (method === 'skip') { setModelOptions([info.model]); setModel(info.model); setStage(mode === 'full' || mode === 'developer' ? 'tools' : 'review'); return; }
    const env = { ...process.env, UTHARNESS_PROVIDER: selectedProvider, UTHARNESS_MODEL: info.model, ...(secret && info.key ? { [info.key]: secret } : {}), ...(selectedProvider === 'custom' ? { UTHARNESS_PROVIDER_URL: customUrl } : {}) };
    const result = await execa(runtimeBinary(), ['models', 'list'], { cwd: process.cwd(), env, reject: false });
    if (result.exitCode !== 0) { setError(result.stderr || result.stdout || 'Credential or model validation failed'); setStage('error'); return; }
    const found = result.stdout.split(/\r?\n/).filter(line => line && line !== 'MODELS' && !line.startsWith('active:'));
    const choices = found.length ? found : [info.model]; setModelOptions(choices); setModel(choices.includes(info.model) ? info.model : choices[0]!); setSelected(0); setStage('model');
  };

  const save = async () => {
    setStage('saving');
    const args = mode === 'import' ? ['setup', '--non-interactive', '--mode', 'import', '--import-config', importPath] : ['setup', '--non-interactive', '--mode', mode, '--provider', mode === 'blank' ? 'offline' : provider, '--model', mode === 'blank' ? 'deterministic-planner' : model, '--tools', (mode === 'blank' ? ['workspace_read'] : [...enabled]).join(',') || 'none'];
    if (mode !== 'import' && provider === 'custom') args.push('--provider-url', customUrl);
    if (mode !== 'import' && auth === 'api_key' && secret) args.push('--api-key-stdin');
    if (mode !== 'import' && auth === 'skip') args.push('--skip-validation');
    const result = await execa(runtimeBinary(), args, { cwd: process.cwd(), input: auth === 'api_key' ? secret : undefined, reject: false }); setSecret('');
    if (result.exitCode === 0) setStage('done'); else { setError(result.stderr || result.stdout || 'Setup failed'); setStage('error'); }
  };

  useInput((input, key) => {
    if (key.ctrl && input === 'c') { setSecret(''); exit(); return; }
    if (stage === 'done' && key.return) { exit(); return; }
    if (stage === 'error' && (key.return || key.escape)) { setError(''); setStage('mode'); setSelected(0); return; }
    if (stage === 'secret' || stage === 'import' || stage === 'custom_url') {
      const value = stage === 'secret' ? secret : stage === 'import' ? importPath : customUrl;
      const update = stage === 'secret' ? setSecret : stage === 'import' ? setImportPath : setCustomUrl;
      if (key.escape) { if (stage === 'secret') setSecret(''); setStage(stage === 'secret' ? 'auth' : stage === 'custom_url' ? 'provider' : 'mode'); return; }
      if (key.backspace || key.delete) { update(value.slice(0, -1)); return; }
      if (key.return) { if (!value.trim()) return; if (stage === 'secret') void loadModels('api_key'); else if (stage === 'custom_url') { setStage('auth'); setSelected(0); } else setStage('review'); return; }
      if (input && !key.ctrl) update(value + input); return;
    }
    if (stage === 'review' && key.return) { void save(); return; }
    if (!list.length || stage === 'scan' || stage === 'saving') return;
    if (key.upArrow) setSelected(value => Math.max(0, value - 1));
    if (key.downArrow) setSelected(value => Math.min(list.length - 1, value + 1));
    if (stage === 'tools' && input === ' ') { const id = tools[selected]?.id; if (id) setEnabled(current => { const next = new Set(current); next.has(id) ? next.delete(id) : next.add(id); return next; }); return; }
    if (key.escape || key.leftArrow) { if (stage === 'mode') exit(); else if (stage === 'provider') setStage('mode'); else if (stage === 'auth') setStage('provider'); else if (stage === 'model') setStage(providerInfo.key ? 'auth' : 'provider'); else if (stage === 'tools') setStage('model'); else if (stage === 'review') setStage(mode === 'blank' ? 'mode' : mode === 'import' ? 'import' : 'model'); setSelected(0); return; }
    if (!key.return && input !== ' ') return;
    if (stage === 'mode') {
      const next = modes[selected]?.id ?? 'quick'; if (next === 'exit') { exit(); return; } setMode(next);
      if (next === 'blank') { setProvider('offline'); setModel('deterministic-planner'); setEnabled(new Set(['workspace_read'])); setStage('review'); }
      else if (next === 'import') setStage('import');
      else if (next === 'local_ai') { const info = providers.find(item => item.id === 'ollama')!; setProvider('ollama'); setAuth('environment'); void loadModels('environment', 'ollama', info); }
      else if (next === 'custom') { setProvider('custom'); setModel('default'); setStage('provider'); setSelected(providers.findIndex(item => item.id === 'custom')); }
      else { if (next === 'developer') setEnabled(new Set(developerTools)); setStage('provider'); setSelected(0); }
    } else if (stage === 'provider') { const info = providers[selected] ?? providers[0]!; setProvider(info.id); setModel(info.model); if (info.id === 'custom') setStage('custom_url'); else if (!info.key) void loadModels('environment', info.id, info); else { setStage('auth'); setSelected(0); } }
    else if (stage === 'auth') { const method = authMethods[selected]?.id ?? 'environment'; setAuth(method); if (method === 'api_key') setStage('secret'); else void loadModels(method); }
    else if (stage === 'model') { setModel(modelOptions[selected] ?? providerInfo.model); if (mode === 'full' || mode === 'developer') { setStage('tools'); setSelected(0); } else setStage('review'); }
    else if (stage === 'tools' && key.return) setStage('review');
  });

  const available = report?.components.filter(item => item.state === 'AVAILABLE').length ?? 0, total = report?.components.length ?? 0;
  return <Box flexDirection="column" paddingX={compact ? 1 : 3} width={columns}>
    <Text bold color={purple}>UTHARNESS · INTERACTIVE SETUP</Text><Text color={muted}>{'─'.repeat(Math.max(20, Math.min(columns - 2, 76)))}</Text>
    {stage === 'scan' ? <><Text>◇ Detecting environment and scanning prerequisites…</Text><Progress completed={0} total={1} label="scanning" /></> : null}
    {stage === 'mode' ? <><Text>◇ {report?.os}/{report?.architecture} · {report?.shell} · {report?.terminal}</Text><Progress completed={available} total={total} label="environment scan complete" /><List items={modes} selected={selected} /></> : null}
    {stage === 'provider' ? <><Text bold>◉ Select AI provider</Text><List items={providers} selected={selected} /></> : null}
    {stage === 'custom_url' ? <><Text bold>◉ Custom OpenAI-compatible /v1 endpoint</Text><Text color={cyan}>› {customUrl}<Text inverse> </Text></Text><Text color={muted}>HTTPS required except loopback development endpoints</Text></> : null}
    {stage === 'auth' ? <><Text bold>◆ Authentication · {providerInfo.label}</Text><List items={authMethods} selected={selected} /></> : null}
    {stage === 'secret' ? <><Text bold>◆ Enter {providerInfo.key}</Text><Text color={cyan}>› {'•'.repeat(secret.length)}<Text inverse> </Text></Text><Text color={muted}>Masked · never logged · private secrets.env</Text></> : null}
    {stage === 'model' ? <><Text bold>◇ Select validated model</Text><List items={list} selected={selected} /></> : null}
    {stage === 'tools' ? <><Text bold>&gt;_ Select runtime capabilities</Text><List items={tools} selected={selected} marked={enabled} /></> : null}
    {stage === 'import' ? <><Text bold>▤ Import utharness.json</Text><Text color={cyan}>› {importPath}<Text inverse> </Text></Text></> : null}
    {stage === 'review' ? <Box flexDirection="column"><Text bold>Review and validate</Text><Text>Mode       <Text color={cyan}>{mode}</Text></Text><Text>Provider   <Text color={cyan}>{provider}</Text></Text><Text>Model      <Text color={cyan}>{model}</Text></Text><Text>Credential <Text color={auth === 'skip' ? yellow : green}>{mode === 'blank' ? 'not required' : auth === 'api_key' ? 'masked key ready' : auth}</Text></Text><Text>Tools      <Text color={cyan}>{(mode === 'blank' ? ['workspace_read'] : selectedTools.map(item => item.id)).join(', ')}</Text></Text><Text color={cyan}>Enter saves, validates, and prepares first chat.</Text></Box> : null}
    {stage === 'saving' ? <Box flexDirection="column"><Text color={cyan}>◇ Validating provider and model…</Text><Text color={muted}>Saving configuration only after every required check succeeds.</Text></Box> : null}
    {stage === 'done' ? <Box flexDirection="column"><Text color={green} bold>✓ Setup complete and validated</Text><Text>Configuration, private secrets, storage, tools, and model route are ready.</Text><Text color={cyan}>Press Enter, then run `utharness` to start the first chat.</Text></Box> : null}
    {stage === 'error' ? <Box flexDirection="column"><Text color={red} bold>! Setup needs attention</Text><Text>{error}</Text><Text color={muted}>Press Enter to return to setup.</Text></Box> : null}
    {['mode', 'provider', 'auth', 'model', 'tools'].includes(stage) ? <Text color={muted}>↑↓ navigate · {stage === 'tools' ? 'Space toggle · ' : ''}Enter select · Esc previous · Ctrl+C exit</Text> : null}
  </Box>;
}
