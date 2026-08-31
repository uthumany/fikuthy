# Setup and environment diagnostics

`fikuthy setup` is a thin interactive client over the canonical Rust setup engine. The renderer collects choices; Rust owns dependency scanning, validation, configuration, secrets, and storage initialization.

## Interactive modes

- **Quick Start** configures a provider, validated model, safe tools, sessions, SQLite, and memory.
- **Full Setup** adds explicit capability selection.
- **Developer Setup** enables the complete approval-gated coding capability set.
- **Local AI Setup** validates an Ollama-compatible local endpoint without requiring a hosted key.
- **Custom Provider** accepts an OpenAI-compatible `/v1` endpoint, masked key, and discovered model.
- **Blank Slate** uses the offline deterministic planner with workspace reading only.
- **Import Configuration** validates and imports an existing schema-version-1 `fikuthy.json`.

Arrow keys navigate, Enter selects, Space toggles capabilities, and Escape returns to the previous step. API-key characters are rendered only as bullets and are sent to the native setup process through stdin, never command-line arguments.

## Configuration and secrets

Non-secret workspace configuration is written to `./fikuthy.json`. Global setup metadata is written to `~/.fikuthy/config.yaml`. Setup-managed credentials are stored separately in `~/.fikuthy/secrets.env`.

On Unix, the secrets file is created atomically with mode `0600`. Values containing line breaks or NUL bytes are rejected. The runtime loads only missing environment values, so an explicit process environment always wins.

## Scriptable setup

```bash
fikuthy setup --scan
fikuthy setup --quick --non-interactive --provider openrouter --skip-validation
printf '%s' "$OPENROUTER_API_KEY" | fikuthy setup --quick --non-interactive \
  --provider openrouter --model openrouter/free --api-key-stdin
fikuthy setup --developer --non-interactive --provider ollama --skip-validation
fikuthy setup --non-interactive --mode import --import-config ./saved-fikuthy.json
```

`--api-key-stdin` is the safe automation path: it avoids shell history and process argument listings. Omit `--skip-validation` to require a live `/models` request and confirmation that the chosen model exists.

## Diagnostics and recovery

```bash
fikuthy doctor
fikuthy doctor --fix
fikuthy provider test
fikuthy models test
fikuthy setup
```

The scanner checks required Git, curl, and Node.js components plus optional Python, uv, pip, npm, pnpm, Bun, Deno, SQLite, ripgrep, ffmpeg, Docker, Podman, SSH, GitHub CLI, Rust, Cargo, and Ollama installations. Results come from real executable health checks. Missing or broken components include platform-package-manager guidance; progress values are calculated from completed checks rather than timers.

`doctor --fix` repairs user configuration directories and prints explicit platform installation commands for missing required system components. It does not silently invoke `sudo` or alter system packages.

## Installer guarantees

The POSIX and PowerShell installers require Node.js 18 or newer, verify the published archive checksum, verify that the native executable and built UI bundle exist, run the installed binary’s version health check, and only then report success. Remote non-loopback provider URLs must use HTTPS.
