# AI gateways and agents

Utharness can run without credentials or connect to an OpenAI-compatible API. Live chat uses server-sent events (SSE), prints token deltas as they arrive, combines the final response, and persists the completed assistant message in the workspace SQLite journal.

## Configure a gateway

Choose a provider and keep its key in the process environment, or enter it through the masked `utharness setup` prompt. Utharness never writes API keys to its database, project configuration, logs, or terminal history. Keys entered through setup are stored only in `~/.utharness/secrets.env`, atomically written with private permissions, and loaded without being displayed.

```bash
export UTHARNESS_PROVIDER=openrouter
export OPENROUTER_API_KEY='your-key'
export UTHARNESS_MODEL='openrouter/free'

utharness providers list
utharness providers test
utharness models list
utharness models test
utharness chat 'Explain the current Git changes'
```

The supported provider identifiers and credential variables are:

| Provider | `UTHARNESS_PROVIDER` | Credential variable | Default endpoint |
| --- | --- | --- | --- |
| OpenRouter | `openrouter` | `OPENROUTER_API_KEY` | `https://openrouter.ai/api/v1` |
| OpenAI | `openai` | `OPENAI_API_KEY` | `https://api.openai.com/v1` |
| Groq | `groq` | `GROQ_API_KEY` | `https://api.groq.com/openai/v1` |
| Together | `together` | `TOGETHER_API_KEY` | `https://api.together.xyz/v1` |
| DeepSeek | `deepseek` | `DEEPSEEK_API_KEY` | `https://api.deepseek.com/v1` |
| Fireworks | `fireworks` | `FIREWORKS_API_KEY` | `https://api.fireworks.ai/inference/v1` |
| NVIDIA NIM | `nvidia` | `NVIDIA_API_KEY` | `https://integrate.api.nvidia.com/v1` |
| Ollama | `ollama` | None | `http://127.0.0.1:11434/v1` |
| Custom | `custom` | `UTHARNESS_API_KEY` | `http://127.0.0.1:8000/v1` |

When `UTHARNESS_PROVIDER` is omitted, Utharness selects the first configured provider in this order: OpenRouter, OpenAI, Groq, Together, DeepSeek, Fireworks, then NVIDIA NIM. Set the provider explicitly when more than one key exists.

`UTHARNESS_API_KEY` overrides the provider-specific credential. `UTHARNESS_PROVIDER_URL` overrides the endpoint, and `UTHARNESS_MODEL` overrides the model. Remote endpoints must use HTTPS. Plain HTTP is accepted only for `localhost`, `127.0.0.1`, or `::1`.

## Local Ollama

Start an Ollama model separately, then use its OpenAI-compatible endpoint:

```bash
ollama pull qwen2.5-coder:7b
ollama serve

export UTHARNESS_PROVIDER=ollama
export UTHARNESS_MODEL=qwen2.5-coder:7b
utharness providers test
utharness chat 'List the main modules in this repository'
```

## Custom gateways

Any gateway implementing `GET /v1/models` and OpenAI-compatible `POST /v1/chat/completions` can be used:

```bash
export UTHARNESS_PROVIDER=custom
export UTHARNESS_PROVIDER_URL=https://gateway.example.com/v1
export UTHARNESS_API_KEY='your-key'
export UTHARNESS_MODEL='organization/model-name'
utharness providers test custom
```

The streaming endpoint must return SSE records whose `data:` payloads contain `choices[].delta.content`, followed by `data: [DONE]`.

## Agent runtime

`utharness agents run` invokes the real bounded inspection engine. The selected model produces a structured plan, and every requested tool is checked against the SAFE policy before execution and recorded in the runtime journal.

```bash
utharness agents list
utharness agents run 'Inspect the repository and identify its largest risks' --max-steps 4
```

The current allowlist is deliberately read-only: `list_directory`, `read_file`, `git_status`, and `git_diff`. Shell execution remains a separate, explicit approval path through `utharness run --command ... --allow`. Write-capable autonomous tools, multi-agent delegation, messaging transports, job scheduling, and MCP server execution are not claimed as complete yet.

This is an original implementation inspired by common agent-terminal capabilities. It does not embed or copy Hermes Agent source code.
