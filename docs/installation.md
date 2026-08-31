# fikuthy installation, package, and platform guide

For the shortest copyable install page, start with [`INSTALLATION.md`](../INSTALLATION.md). This document contains the full platform, terminal, update, uninstall, and troubleshooting matrix.

`fikuthy` is distributed as a native Rust CLI with a bundled React/Ink terminal UI. The first public release provides signed release archives for **Linux x64, macOS x64/arm64, and Windows x64**, plus thin launchers on **npm** and **PyPI** that download and verify the matching native archive on first use. Other combinations are documented as source, remote-host, or unsupported paths rather than being presented as working installers.

## Termux / Android

The native Termux package is built from [`packages/fikuthy`](../packages/fikuthy), placed only in Termux’s `$PREFIX`, and distributed through the signed APT repository. It does not require root. The package and metadata are verified from the public Pages endpoints; physical-device and Android-version-specific behavior remains unverified in this Linux-only QA environment.

```bash
pkg update
pkg install curl
curl -fsSL https://fikuthy.github.io/fikuthy/termux/install.sh | bash
pkg update
pkg install fikuthy
fikuthy setup
fikuthy
```

Termux package updates must remain package-manager-owned:

```bash
pkg update
pkg upgrade fikuthy
```

The package uses only these prefix paths:

```text
$PREFIX/bin/fikuthy
$PREFIX/lib/fikuthy
$PREFIX/share/fikuthy
```

User state is never stored in the package prefix:

```text
$HOME/.config/fikuthy
$HOME/.local/share/fikuthy
$HOME/.cache/fikuthy
```

Use the built-in diagnostics and optional Android integration commands as follows:

```bash
fikuthy termux info
fikuthy termux setup
fikuthy termux doctor
fikuthy termux permissions
fikuthy termux keys install
fikuthy termux storage enable
fikuthy termux api
fikuthy termux api battery
```

Termux:API is optional. The core CLI, SQLite persistence, offline planner, diagnostics, and TUI continue to work when the matching Termux:API app or package is absent. Install optional capabilities only when needed:

```bash
pkg install termux-api git openssh python
```

## Quick start

After installation, run `fikuthy setup` in a terminal. The wizard scans the environment, provides Quick, Full, Developer, Local AI, Custom Provider, Blank Slate, and Import modes, and uses Space to toggle capabilities where applicable. It writes non-secret workspace settings to `fikuthy.json`; masked setup credentials are stored separately in `~/.fikuthy/secrets.env` with private permissions and are automatically loaded by the runtime.

The scriptable equivalent is:

```bash
fikuthy setup --non-interactive --mode quick --provider openrouter
```

Only gateways and capabilities backed by the current runtime appear as selectable. Operations that can mutate files or invoke a shell are saved with `ask` permission mode and still require the explicit runtime approval path.

### Release archive on Linux or macOS

```bash
curl -fsSL https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.sh | bash
export PATH="$HOME/.local/bin:$PATH"
fikuthy --help
fikuthy --version
fikuthy
```

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.ps1 | iex
fikuthy --help
fikuthy --version
fikuthy
```

### npm, npx, pnpm, or pnpx

```bash
npm install --global fikuthy
fikuthy --help
fikuthy

npx --yes fikuthy --version
pnpm add --global fikuthy
pnpx fikuthy --help
```

The npm package installs two aliases, `fikuthy` and `fikuthy`. It downloads the release archive into the user cache, verifies `SHA256SUMS`, and forwards all arguments to the native runtime. `npx` and `pnpx` do not permanently install the launcher unless their package-manager cache is retained.

### PyPI, pip, pipx, uv, or uvx

```bash
python -m pip install fikuthy
fikuthy --help
fikuthy

pipx install fikuthy
uv tool install fikuthy
uvx fikuthy --version
```

The Python package is a dependency-free launcher. It uses the same release archive and checksum verification as npm. The package requires Python 3.9 or newer; the downloaded application itself is the native Rust runtime and bundled Ink UI.

## Installation matrix

| Method | Status | Copyable command or workflow | Entry point and requirements |
|---|---|---|---|
| `curl` | **Supported and tested** | `curl -fsSL https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.sh \| bash` | Installs a matching POSIX release archive into `~/.local/bin`; requires `curl`, `tar`, and `sha256sum`; add `~/.local/bin` to `PATH`. |
| npm | **Supported and tested** | `npm install --global fikuthy` | Installs `fikuthy` and `fikuthy`; requires Node.js 18+ for the launcher and a supported release target. |
| npx | **Supported and tested** | `npx --yes fikuthy --help` | Ephemeral npm launcher; native archive is cached per user. |
| pnpm | **Supported and tested** | `pnpm add --global fikuthy` | Provides the same npm launcher; requires pnpm and Node.js 18+. |
| pnpx | **Supported and tested** | `pnpx fikuthy --version` | Ephemeral pnpm launcher; native archive is cached per user. |
| Bun / `bunx` | **Partially supported** | `bunx fikuthy --help` | Bun can execute the dependency-free npm launcher, but the downloaded native target still must match Linux x64, macOS x64, or Windows x64. Bun is not used to build the Ink bundle in release archives. |
| Deno | **Unverified/source-only** | Use the Git source workflow; do not treat `deno run npm:fikuthy` as a tested product installer. | Deno npm compatibility may be useful for experiments, but no native Deno distribution path is currently verified. |
| `uv` / `uvx` | **Supported and tested** | `uv tool install fikuthy` or `uvx fikuthy --version` | Installs the PyPI launcher; requires Python 3.9+ through uv and a supported native release target. |
| `pip` / `pipx` / `python -m` | **Supported and tested** | `python -m pip install fikuthy`; `pipx install fikuthy` | Installs the PyPI launcher and its `fikuthy` alias; requires Python 3.9+. |
| Cargo | **Source/native CLI only** | `cargo build --release` | Produces `target/release/fikuthy`; a bare `cargo install` does not include the bundled UI and is not advertised as a complete TUI install. |
| Homebrew | **Unavailable as a product formula** | Use the curl installer or Git source workflow; do not run `brew install fikuthy`. | Homebrew can install prerequisites, but no maintained tap formula is published yet. |
| apt | **Unavailable as a product repository** | Use the curl installer or Git source workflow; `sudo apt install git curl build-essential pkg-config` installs prerequisites only. | No Debian repository is claimed. |
| Nix | **Source-only** | Provide Rust stable, Node 22, pnpm, and Git in a Nix shell, then use the source workflow. | No flake or Nixpkgs derivation is published yet. |
| Volta / mise / fnm / nvm | **Prerequisite managers** | Install Node 22 with the manager, then use npm/pnpm or the source UI workflow. | These tools manage Node/PATH; they do not install the Rust runtime. Verify with `node --version`. |
| Corepack | **Supported prerequisite path** | `corepack enable && corepack prepare pnpm@10.15.0 --activate && pnpm --dir ui install --frozen-lockfile` | Bootstraps the repository-pinned pnpm version; it does not install a published product binary. |
| Rush / Lerna / cnpm | **Not required** | Use the repository’s pnpm lockfile and `pnpm --dir ui ...`; do not add an orchestrator solely for compatibility claims. | No Rush/Lerna workspace or cnpm-specific distribution is maintained. |
| Git | **Supported source path** | `git clone https://github.com/fikuthy/fikuthy.git && cd fikuthy` | Requires Rust stable, Node.js 22, and pnpm to build the complete CLI/TUI. |
| winget | **Prerequisite channel** | `winget install OpenJS.NodeJS.LTS Git.Git` | Installs prerequisites on Windows; no fikuthy winget manifest is published. Use the PowerShell installer or source build afterward. |
| Termux APT repository | **Supported and published; repository verified** | `curl -fsSL https://fikuthy.github.io/fikuthy/termux/install.sh | bash && pkg install fikuthy` | Requires Termux, `curl`, and a network connection; no root. Android device/emulator execution remains to be verified. |

## Source installation and development

```bash
git clone https://github.com/fikuthy/fikuthy.git
cd fikuthy
. "$HOME/.cargo/env"  # if Rustup is installed in the default location
corepack enable
corepack prepare pnpm@10.15.0 --activate
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui typecheck
pnpm --dir ui build
cargo build --release
./target/release/fikuthy --help
./target/release/fikuthy tui --headless
```

The runtime stores SQLite data under the platform data directory, or under `FIKUTHY_HOME` when explicitly set. `FIKUTHY_DB` overrides the database path. Provider configuration is environment-based; credentials are not written by the installers.

## Update, uninstall, and clean reinstall

For the curl or PowerShell installers, rerun the installer with `FIKUTHY_VERSION=vX.Y.Z` after a release is published. For npm, use `npm update --global fikuthy`; for pnpm, use `pnpm update --global fikuthy`; for pip, use `python -m pip install --upgrade fikuthy`; for pipx, use `pipx upgrade fikuthy`; and for uv, use `uv tool upgrade fikuthy`. The launcher also accepts `fikuthy update` to clear and redownload its cached native archive.

To uninstall the npm launcher, run `npm uninstall --global fikuthy` and remove the per-user cache at `$XDG_CACHE_HOME/fikuthy` or `~/.cache/fikuthy`. For Python, run `python -m pip uninstall fikuthy` or `pipx uninstall fikuthy`, then remove the same cache. The `uninstall` subcommand prints these commands without silently modifying a package-manager environment. For a clean source reinstall, remove the checkout and clone it again; remove `target/` and `ui/node_modules/` only when a clean rebuild is needed.

## Compatibility matrix

| Platform / runtime | Release archive | npm/PyPI launcher | Source build | Validation status |
|---|---:|---:|---:|---|
| Ubuntu/Linux x64 | Yes | Yes | Yes | **Tested locally and in hosted CI** |
| macOS x64 | Yes | Yes | Yes | **Tested in hosted CI; native local shell not used** |
| Windows x64 | Yes | Yes | Yes | **Tested in hosted CI; native local shell not used** |
| Linux arm64 | No | No matching archive | Possible if Rust/Node toolchains are available | **Partially supported; source only** |
| macOS arm64 | Yes | Yes | Yes | **Built and tested on a hosted Apple Silicon runner; native local shell not used** |
| Windows arm64 | No | No matching archive | Possible if Rust/Node toolchains are available | **Partially supported; source only** |
| Android / Termux | Yes: signed APT packages | No matching npm/PyPI archive | Possible with Android toolchains | **Package/repository verified; physical-device test pending** |
| iOS/iPadOS | No | No | No local native claim | **SSH/remote-host workflow** |
| FreeBSD | No | No | Possible if dependencies are available | **Source/remote workflow** |
| Other Unix-like systems | No | No | Possible if dependencies are available | **Source/remote workflow** |

## Terminal matrix

| Terminal environment | Status | Notes |
|---|---|---|
| Windows Terminal, WezTerm, Alacritty, Tabby, Cmder | **Supported rendering class** | Pair with the Windows x64 release or source workflow; Windows-hosted CI validates the CLI build, not every emulator. |
| Kitty, WezTerm, Alacritty, Ghostty, Tilix on Linux | **Supported rendering class** | Linux PTY screenshots and terminal smoke tests were run on Linux; emulator-specific glyph differences remain possible. |
| Ghostty, iTerm2, Warp, WezTerm, Kitty on macOS | **Supported rendering class** | Use the macOS x64 or arm64 release or source workflow; hosted CI validates macOS builds. |
| Termux | **Supported package path; device validation pending** | Use the signed v0.2.13 repository. Real terminal behavior, Android-version compatibility, and Termux:API execution still require device testing. |
| Termius, ConnectBot, TermAI, Moshi | **Remote-client workflow** | These are SSH/client environments, not independently tested native build targets. |
| Blink Shell, Secure ShellFish | **Remote-client workflow** | Connect to a supported Linux/macOS host. |
| a-Shell, iSH | **Unsupported local native target** | Use SSH or a remote/container host. |
| xterm, Konsole, Hyper | **ANSI terminal frontends** | Use a supported host and release/source installation; the frontend does not install fikuthy. |

## Troubleshooting

If the native UI reports that its bundle is missing after a source build, run `pnpm --dir ui install --frozen-lockfile && pnpm --dir ui build`. If `fikuthy` is not found after the archive installer, add `~/.local/bin` to `PATH` and open a new shell. If a launcher reports that no binary exists for the current architecture, use a source build or a supported remote host; do not bypass the check. If the terminal is garbled, verify UTF-8 and ANSI support, inspect `TERM`, and try `NO_COLOR=1`. If a provider is unavailable, the offline planner and local diagnostics remain available; provider credentials are read from environment variables and are never stored by the package launchers.

## Development checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
CI=1 pnpm --dir ui install --frozen-lockfile
pnpm --dir ui typecheck
pnpm --dir ui test
pnpm --dir ui build
```
