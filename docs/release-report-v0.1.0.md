# utharnessly v0.1.0 release report

## Release identity

| Field | Verified value |
|---|---|
| Repository | [`uthumany/utharnessly`][1] |
| Release | [`v0.1.0`][2] |
| Git commit | `5771be7f5917f90d27898a1cd24768840c96384f` |
| Git tag | `v0.1.0` (annotated, pushed to origin) |
| npm package | [`utharnessly@0.1.0`][3] |
| PyPI package | [`utharnessly==0.1.0`][4] |
| Native release assets | Linux x64, macOS x64, Windows x64, plus `SHA256SUMS` |
| Release workflow | GitHub Actions run `33021732796`, success |

The tag points at the verified release-preparation commit. The `main` branch is synchronized with `origin/main`, and the working tree was clean after publishing and live-install verification. The repository rename from the former `utharness-runtime` identity preserved history and the old GitHub URL continues to redirect.

## Published distribution paths

The GitHub Release contains `utharnessly-linux-x64.tar.gz`, `utharnessly-macos-x64.tar.gz`, `utharnessly-windows-x64.zip`, and `SHA256SUMS`. The POSIX and PowerShell installers verify the checksum when the checksum asset is available. The npm and PyPI packages are thin, dependency-free launchers: they download the matching archive on first use, verify its checksum, cache the runtime and bundled Ink UI, and forward arguments to the native `utharness` binary.

Both registries expose `utharness` and `utharnessly` entry points. The launchers also support `--help`, `--version`, `update`, and explicit uninstall guidance. Unsupported operating-system and architecture combinations fail with source-build or remote-host guidance rather than attempting an incompatible download.

## Validation completed

| Validation area | Result and evidence |
|---|---|
| Fresh Rust dependency/build path | Passed: `cargo fmt --all -- --check`, strict Clippy, workspace tests, and `cargo build --release` |
| Fresh UI dependency/build path | Passed: frozen pnpm install, TypeScript type-check, UI tests, production build |
| UI/TUI rendering | Passed: PTY capture and screenshot matrix at 40, 60, 80, 120, 160, and 220 columns, including short-height layouts |
| CLI entry points | Passed: `utharness --help`, `utharness --version`, default startup, `tui --headless`, `init`, `chat`, `config show`, `doctor`, `providers`, `skills`, `agents`, and `tools` |
| Persistence workflows | Passed: sessions, offline chat persistence, memory add/search, checkpoint creation, isolated `UTHARNESS_HOME`, and database diagnostics |
| Safety/error behavior | Passed: SAFE shell denial, destructive-command denylist rejection, offline provider-missing behavior, and explicit unsupported-architecture errors |
| npm artifact | Passed: syntax check, `npm pack --dry-run`, clean local install, real registry install, npx, pnpm dlx, cache download, update, uninstall guidance, and reinstall |
| PyPI artifacts | Passed: Python unit tests, source distribution, wheel, strict Twine checks, clean venv install, real registry install, update, uninstall guidance, and reinstall |
| Release archives | Passed: all three live assets downloaded and verified against `SHA256SUMS`; archives contain the native binary and `ui/dist/index.js` |
| POSIX installer | Passed: local fixture and live GitHub Release installation; installed binary reports `utharness 0.1.0` and bundled UI is present |
| Hosted CI | Passed: CI run `33021478397` and Security run `33021478578` |
| Hosted release | Passed: Release run `33021732796` built and uploaded all release assets |

## Tested installers and package managers

The curl installer, npm global install, npx, pnpm global launcher, pnpm dlx, Python `pip` in a clean virtual environment, and the PyPI launcher were executed successfully against the public v0.1.0 release. The local package simulations additionally covered actual package uninstall and reinstall, isolated cache directories, update redownloads, and checksum rejection.

Cargo and Git source workflows were validated locally with Rust stable, Node 22, and pnpm 10.15.0. Bun, bunx, Deno, pipx, uv, uvx, Homebrew, apt, Nix, Volta, mise, fnm, nvm, Corepack, Rush, Lerna, cnpm, and winget are documented according to their real role: direct source execution, prerequisite/runtime management, remote workflow, or unavailable product package. They are not reported as native product installers unless the corresponding distribution exists and was tested.

## Platform and terminal compatibility

| Combination | Status |
|---|---|
| Ubuntu/Linux x64 | Supported; tested locally and in hosted CI |
| macOS x64 | Supported release target; tested in hosted CI, not run in the local Linux shell |
| Windows x64 | Supported release target; tested in hosted CI, not run in the local Linux shell |
| Linux/macOS/Windows ARM | Source-only or future release targets; no matching v0.1.0 archive |
| Android/Termux | Partial/source workflow; use SSH when Rust/Node tooling is unavailable |
| iOS/iPadOS | Remote-host/SSH workflow; no local native claim |
| FreeBSD and other Unix-like systems | Source or remote workflow; no native release archive |
| Linux PTY terminal rendering | Tested at 40–220 columns with compact and short-height captures |
| Windows/macOS terminal emulators | Rendering class documented; emulator-specific native execution was not locally performed |

The documented terminal environments include Windows Terminal, WezTerm, Alacritty, Tabby, Cmder, Kitty, Ghostty, Tilix, iTerm2, Warp, Termux, Termius, ConnectBot, TermAI, Moshi, Blink Shell, a-Shell, iSH, Secure ShellFish, xterm, Konsole, and Hyper. Client-only and restricted environments are explicitly classified as remote-host workflows where appropriate.

## Failures found and resolved

The first hosted package-validation run failed because the CI test step did not add the in-tree Python `src` directory to `PYTHONPATH`. The workflow was corrected to run `PYTHONPATH=src python -m unittest discover -s tests -v`; the subsequent complete CI run passed. Two local fixture-script failures were also corrected during QA: one used the wrong npm working-directory invocation, and one checked a checksum manifest from the wrong directory. A real npm launcher regression involving a missing stream-pipeline import was fixed and retested through both local and live registry installation paths.

No known functional test failures remain for the tested combinations. GitHub Actions emitted non-blocking warnings that several existing third-party actions target the deprecated Node.js 20 runtime; the workflows still pass, but those action major versions should be refreshed when compatible releases are available.

## Known limitations and unsupported combinations

The release does not publish ARM64 archives, native Android/iOS/iPadOS artifacts, FreeBSD archives, Homebrew formulas, Debian repositories, Nix flakes, winget manifests, or product packages for Rush, Lerna, cnpm, or other runtime managers. These combinations remain source-build, remote-host, or prerequisite-only paths. The bounded autonomous command requires an explicitly configured OpenRouter-compatible provider; the offline planner, persistence, diagnostics, and safety workflows work without provider credentials.

The Rust launcher starts the bundled Ink UI with Node.js on supported release targets. A complete TUI source build therefore requires Node.js 22 or newer and the repository-pinned pnpm version. Package launchers require Node.js 18+ for the npm launcher or Python 3.9+ for the PyPI launcher, while the downloaded native archive remains limited to the published host targets above.

[1]: https://github.com/uthumany/utharnessly "utharnessly repository"
[2]: https://github.com/uthumany/utharnessly/releases/tag/v0.1.0 "utharnessly v0.1.0 release"
[3]: https://www.npmjs.com/package/utharnessly "utharnessly on npm"
[4]: https://pypi.org/project/utharnessly/0.1.0/ "utharnessly 0.1.0 on PyPI"
