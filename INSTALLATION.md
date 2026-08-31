# Install fikuthy

This page is the copyable installation entrypoint for the public [`fikuthy/fikuthy`](https://github.com/fikuthy/fikuthy) repository. The native CLI executable is named `fikuthy`; the distribution and package identity is `fikuthy`.

## npm and npx

The npm package is [`fikuthy`](https://www.npmjs.com/package/fikuthy). It is a thin launcher: it downloads the matching signed native release archive on first use, verifies `SHA256SUMS`, caches the native runtime and bundled Ink UI, and forwards arguments to `fikuthy`.

```bash
npm install --global fikuthy
fikuthy --help
fikuthy --version
fikuthy
```

Use `npx` without a permanent global install:

```bash
npx --yes fikuthy --help
npx --yes fikuthy --version
```

The package exposes both commands:

```bash
fikuthy --help
fikuthy --help
```

## pnpm, pnpx, Bun, and Deno

These tools resolve the same published npm package:

```bash
pnpm add --global fikuthy
fikuthy --version
pnpx fikuthy --help

bun add --global fikuthy
bunx fikuthy --help
```

The npm launcher currently publishes native artifacts for **Linux x64, macOS x64/arm64, and Windows x64**. Deno’s npm compatibility layer is documented as an **unverified/source fallback**, not as a tested native installation method; use the Git source workflow below unless you validate the command on your target Deno release.

## PyPI, pipx, and uv

The PyPI package is [`fikuthy`](https://pypi.org/project/fikuthy/). It exposes both `fikuthy` and `fikuthy` console scripts.

```bash
python -m pip install fikuthy
fikuthy --help
fikuthy --version
fikuthy
```

Isolate the command with `pipx` or `uv`:

```bash
pipx install fikuthy
uv tool install fikuthy
uvx fikuthy --help
```

## Shell installer for Linux and macOS

The POSIX installer downloads the matching GitHub release archive, verifies its checksum, installs the binary and bundled UI under `~/.local/bin`, and prints source-build instructions when no matching artifact exists.

```bash
curl -fsSL https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.sh | bash
export PATH="$HOME/.local/bin:$PATH"
fikuthy --help
fikuthy --version
fikuthy
```

Pin a release explicitly:

```bash
curl -fsSL https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.sh | \
  FIKUTHY_VERSION=0.2.13 bash
```

## Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/fikuthy/fikuthy/main/packaging/install.ps1 | iex
fikuthy --help
fikuthy --version
fikuthy
```

## Build from the GitHub source repository

Use this path on unsupported architectures, FreeBSD, Android terminal environments, or when a package registry or release archive is unavailable.

```bash
git clone https://github.com/fikuthy/fikuthy.git
cd fikuthy

# Requirements: Rust stable, Node.js 22+, and pnpm 10+
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cargo build --release

./target/release/fikuthy --help
./target/release/fikuthy --version
./target/release/fikuthy tui
```

## What is and is not published

| Method | Status | Copyable entrypoint |
|---|---|---|
| npm | Published | `npm install --global fikuthy` |
| npx | Published | `npx --yes fikuthy --help` |
| pnpm/pnpx | Published through npm registry | `pnpm add --global fikuthy` / `pnpx fikuthy` |
| Bun/bunx | Published through npm registry | `bunx fikuthy --help` |
| Deno | Unverified/source-only | Use the Git source workflow; `deno run npm:fikuthy` is not a tested product installer |
| pip/python | Published | `python -m pip install fikuthy` |
| pipx/uv/uvx | Published through PyPI | `pipx install fikuthy` / `uvx fikuthy --help` |
| curl | Published release archive | `curl .../packaging/install.sh \| bash` |
| PowerShell | Published Windows release archive | `irm .../packaging/install.ps1 \| iex` |
| Cargo | Source-build path | `cargo build --release` |
| Homebrew, apt, Nix, winget | No fikuthy package currently published | Use npm/PyPI/release archive/source build |
| Volta, mise, fnm, nvm, Corepack | Runtime prerequisite managers | Install Node 22+, then use npm/npx/pnpm |
| Rush, Lerna, cnpm | npm-compatible workflows | Use the registry package through the manager’s npm resolution path |

For update, uninstall, troubleshooting, operating-system, terminal-environment, and compatibility details, see [`docs/installation.md`](docs/installation.md). For the package source, see [`packages/fikuthy-npm`](packages/fikuthy-npm) and [`python/fikuthy`](python/fikuthy).
