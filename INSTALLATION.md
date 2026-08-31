# Install utharnessly

This page is the copyable installation entrypoint for the public [`uthumany/utharnessly`](https://github.com/uthumany/utharnessly) repository. The native CLI executable is named `utharness`; the distribution and package identity is `utharnessly`.

## npm and npx

The npm package is [`utharnessly`](https://www.npmjs.com/package/utharnessly). It is a thin launcher: it downloads the matching signed native release archive on first use, verifies `SHA256SUMS`, caches the native runtime and bundled Ink UI, and forwards arguments to `utharness`.

```bash
npm install --global utharnessly
utharness --help
utharness --version
utharness
```

Use `npx` without a permanent global install:

```bash
npx --yes utharnessly --help
npx --yes utharnessly --version
```

The package exposes both commands:

```bash
utharnessly --help
utharness --help
```

## pnpm, pnpx, Bun, and Deno

These tools resolve the same published npm package:

```bash
pnpm add --global utharnessly
utharness --version
pnpx utharnessly --help

bun add --global utharnessly
bunx utharnessly --help
```

The npm launcher currently publishes native artifacts for **Linux x64, macOS x64/arm64, and Windows x64**. Deno’s npm compatibility layer is documented as an **unverified/source fallback**, not as a tested native installation method; use the Git source workflow below unless you validate the command on your target Deno release.

## PyPI, pipx, and uv

The PyPI package is [`utharnessly`](https://pypi.org/project/utharnessly/). It exposes both `utharness` and `utharnessly` console scripts.

```bash
python -m pip install utharnessly
utharness --help
utharness --version
utharness
```

Isolate the command with `pipx` or `uv`:

```bash
pipx install utharnessly
uv tool install utharnessly
uvx utharnessly --help
```

## Shell installer for Linux and macOS

The POSIX installer downloads the matching GitHub release archive, verifies its checksum, installs the binary and bundled UI under `~/.local/bin`, and prints source-build instructions when no matching artifact exists.

```bash
curl -fsSL https://raw.githubusercontent.com/uthumany/utharnessly/main/packaging/install.sh | bash
export PATH="$HOME/.local/bin:$PATH"
utharness --help
utharness --version
utharness
```

Pin a release explicitly:

```bash
curl -fsSL https://raw.githubusercontent.com/uthumany/utharnessly/main/packaging/install.sh | \
  UTHARNESS_VERSION=0.2.13 bash
```

## Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/uthumany/utharnessly/main/packaging/install.ps1 | iex
utharness --help
utharness --version
utharness
```

## Build from the GitHub source repository

Use this path on unsupported architectures, FreeBSD, Android terminal environments, or when a package registry or release archive is unavailable.

```bash
git clone https://github.com/uthumany/utharnessly.git
cd utharnessly

# Requirements: Rust stable, Node.js 22+, and pnpm 10+
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cargo build --release

./target/release/utharness --help
./target/release/utharness --version
./target/release/utharness tui
```

## What is and is not published

| Method | Status | Copyable entrypoint |
|---|---|---|
| npm | Published | `npm install --global utharnessly` |
| npx | Published | `npx --yes utharnessly --help` |
| pnpm/pnpx | Published through npm registry | `pnpm add --global utharnessly` / `pnpx utharnessly` |
| Bun/bunx | Published through npm registry | `bunx utharnessly --help` |
| Deno | Unverified/source-only | Use the Git source workflow; `deno run npm:utharnessly` is not a tested product installer |
| pip/python | Published | `python -m pip install utharnessly` |
| pipx/uv/uvx | Published through PyPI | `pipx install utharnessly` / `uvx utharnessly --help` |
| curl | Published release archive | `curl .../packaging/install.sh \| bash` |
| PowerShell | Published Windows release archive | `irm .../packaging/install.ps1 \| iex` |
| Cargo | Source-build path | `cargo build --release` |
| Homebrew, apt, Nix, winget | No utharnessly package currently published | Use npm/PyPI/release archive/source build |
| Volta, mise, fnm, nvm, Corepack | Runtime prerequisite managers | Install Node 22+, then use npm/npx/pnpm |
| Rush, Lerna, cnpm | npm-compatible workflows | Use the registry package through the manager’s npm resolution path |

For update, uninstall, troubleshooting, operating-system, terminal-environment, and compatibility details, see [`docs/installation.md`](docs/installation.md). For the package source, see [`packages/utharnessly-npm`](packages/utharnessly-npm) and [`python/utharnessly`](python/utharnessly).
