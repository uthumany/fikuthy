# UTHARNESS Test Matrix

**Date:** 2026-08-27
**Repository:** `uthumany/utharnessly`
**Source QA target:** 0.2.8
**Published release:** v0.2.8

Status values are **Passed**, **Partial**, **Unverified**, or **Unsupported**. Partial and unverified combinations are intentionally not described as working.

## Platform and runtime matrix

| Platform/runtime | Native artifact | Installer path | Tested environment | Result | Evidence / notes |
|---|---|---|---|---|---|
| Linux x64 | Yes | curl, archive, NPM, PyPI, source | Linux sandbox and Ubuntu hosted CI | Passed | Rust/UI/CLI/package tests and live v0.2.8 archive verification. |
| macOS x64 | Yes | archive, NPM, PyPI, source | macOS hosted CI | Passed | Hosted build, lint, test, and package workflow. |
| macOS arm64 | Yes | curl, archive, NPM, PyPI, source | Apple Silicon macOS hosted CI | Passed | Architecture-specific release artifact and launcher routing. |
| Windows x64 | Yes | PowerShell, archive, NPM, PyPI, source | Windows hosted CI | Passed | Hosted build, lint, test, and package workflow. |
| Android Termux aarch64 | Yes | Signed Pages APT repository | GitHub-hosted Android cross-build; no device | Partial | Binary/package/repository built and verified; physical Termux install pending. |
| Android Termux x86_64 | Yes | Signed Pages APT repository | GitHub-hosted Android cross-build; no device | Partial | Binary/package/repository built and verified; physical Termux install pending. |
| iOS/iPadOS terminal apps | No local artifact | SSH/remote host | Not available | Unsupported local / remote workflow | Use Blink, Termius, a-Shell, iSH, or Secure ShellFish to reach a supported host. |
| FreeBSD | No published artifact | Source/remote | Not available | Unverified | No native package is claimed. |
| Linux ARM64 desktop | No published artifact | Source/remote | Not available | Partial/source-only | No matching release archive is claimed. |
| macOS ARM64 | No published artifact | Source/remote | Not available | Partial/source-only | No matching release archive is claimed. |
| Windows ARM64 | No published artifact | Source/remote | Not available | Partial/source-only | No matching release archive is claimed. |
| SSH session | Uses host artifact | Host-specific installer | Not directly exercised | Unverified | Terminal UI is expected to follow host capabilities; transport-specific E2E pending. |
| Container | Uses host artifact | Source/archive | Not directly exercised | Unverified | No container image is published. |
| CI | Matrix artifacts | Workflow | GitHub Actions | Passed | CI and Security green; Android cross-build release jobs passed. |

## Terminal condition matrix

| Condition | Widths/states | Result | Evidence |
|---|---|---|---|
| Compact terminal | 40 columns × 18 rows | Passed in Linux PTY simulation | `docs/qa/screenshots/termux/simulated/focus-40x18-linux-simulation.png` |
| Narrow terminal | 60 columns × 20 rows and 80 × 24 | Passed in Linux PTY simulation | `docs/qa/screenshots/termux/simulated/` |
| Standard terminal | 120 columns × 36 rows | Passed and visually inspected | `docs/qa/screenshots/linux/focus/focus-120x36-2026-08-27.png` |
| Wide terminal | 160 × 40 and 220 × 44 | Passed in Linux PTY simulation | `docs/qa/screenshots/linux/focus/` |
| Truecolor | `COLORTERM=truecolor` | Passed | PTY capture environment and screenshots. |
| Limited color / monochrome | `NO_COLOR=1` | Passed by UI unit test | `ui/test/ui.test.ts`. |
| Unicode banner | UTF-8 Linux PTY | Passed | Banner output and screenshots. |
| Resize events | PTY harness and responsive breakpoints | Partial | Breakpoint tests and capture matrix passed; live emulator resize pending. |
| Short height | Included in capture dimensions | Passed in Linux PTY simulation | 18–24 row captures retain fixed chrome. |
| Slow/offline provider | Local offline planner | Passed | `chat`, `doctor`, persistence, and mock-provider tests. |
| Interrupted process | Ctrl+C capture harness | Partial | Rust-to-Ink bridge captures Ctrl+C; broader restart scenarios remain pending. |

## Installer and package matrix

| Method | Scope | Result | Evidence / boundary |
|---|---|---|---|
| curl POSIX installer | Linux x64 and macOS x64/arm64 | Passed in local/hosted checks | Native archive and checksum flow; macOS local execution unavailable. |
| PowerShell installer | Windows x64 | Passed in hosted workflow | Native Windows shell not available locally. |
| npm / npx / pnpm / pnpx | Supported native release targets | Source package 0.2.8 clean install passed; NPM 0.2.8 live install passed | NPM launcher downloads, verifies, caches, and forwards. |
| Bun / bunx | Launcher compatibility | Partial | Documentation-only compatibility class; Bun runtime not installed in sandbox. |
| PyPI / pip / pipx / python -m | Supported native release targets | Source wheel/sdist clean install passed; PyPI 0.2.8 canonical-index install passed | Python unittest, build, local virtualenv install, and version checks passed. |
| uv / uvx | Python launcher path | Partial | Manifest supports it; uv-specific installation not executed in this environment. |
| Cargo | Source/native CLI | Passed for build; complete UI install requires separate UI build | `cargo build --release`; bare cargo install is not advertised as complete TUI packaging. |
| Termux APT | Android aarch64/x86_64 | Repository verified; device install pending | v0.2.8 live Pages repository signatures, indexes, packages, and checksums passed. |
| Homebrew | No maintained formula | Unsupported product channel | Use curl/archive or source workflow. |
| apt | No Debian repository | Unsupported product channel | Use curl/archive or source workflow; Termux APT is separate. |
| Nix | No flake/derivation | Source-only | Use Rust/Node/pnpm source workflow. |
| Volta/mise/fnm/nvm/Corepack | Prerequisite managers | Partial | Managers can provide Node/pnpm prerequisites but do not install the Rust product. |
| Rush/Lerna/cnpm | Not used by repository | Unsupported/not required | Repository uses pnpm lockfile directly. |
| Git | Source installation | Passed for clone/build workflow | Requires Rust stable, Node 22, and pnpm. |
| winget | Prerequisite channel | Unsupported product package | Use PowerShell installer or source after installing prerequisites. |

## Feature matrix

| Feature | Normal path | Invalid/empty path | Persistence/recovery | Result |
|---|---|---|---|---|
| Startup/banner | `utharness`, `tui --headless` | Narrow/limited color | Process exit and rerun | Passed in CLI and PTY tests. |
| Offline chat | `chat PROMPT` | Empty search and invalid session | SQLite session/messages | Passed. |
| Autonomous planner | `autonomous PROMPT` | Missing key, invalid JSON, unsupported tool | Event log and bounded plan | Mock-provider E2E passed; real provider not repeated. |
| SAFE shell policy | `run --command` | Mutation without `--allow` | No command execution | Passed. |
| Trusted shell policy | `run --allow --command` | Destructive denylist | Exit status reported | Passed for safe command; destructive command denied. |
| Memory | `memory add/search/list` | No matches | SQLite FTS | Passed. |
| Sessions | `sessions new/list` | Invalid UUID/missing session | SQLite | Passed. |
| Doctor | `doctor` | Missing optional provider/tools | Readable diagnostics | Passed. |
| Skill registry | `skills`, search, categories, install/remove/test/doctor/run | Unsafe/external metadata | Quarantine/health state | Built-in lifecycle passed; external adapters intentionally limited. |
| Termux diagnostics | `termux info/setup/api/permissions/doctor` | Missing API/storage | No-root path setup | Simulation passed; device pending. |

## Release status

The published v0.2.8 native release, signed Termux repository, NPM package, and PyPI package are available and verified from public endpoints. No critical or high-severity defect is currently reproducible in the tested environments. The principal remaining risk is direct device and restricted-platform validation.
