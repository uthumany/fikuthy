# UTHARNESS QA Report

**Date:** 2026-08-27
**Repository:** `uthumany/utharnessly`
**QA target:** published v0.2.7 source, native release, Termux repository, NPM package, and PyPI package
**Method:** `DISCOVER → BUILD → INSTALL → TEST → DIAGNOSE → FIX → RETEST → REGRESSION → SCREENSHOT → DOCUMENT`

## Executive assessment

The current source tree has no reproducible P0 defect and the core local/hosted Rust, UI, CLI, package, Termux-layout, and security gates pass after the fixes recorded below. A release-facing version-drift defect was reproduced and corrected: the native CLI, NPM launcher, and Python launcher previously reported or targeted 0.1.0 while the live Termux/native release line was 0.2.6. The corrected source line is 0.2.7 and includes a regression test.

The v0.2.7 Termux repository is live and was cryptographically verified from its public Pages endpoints. Physical Android devices/emulators, iOS/iPadOS local terminals, FreeBSD, ARM desktop variants, and every named terminal emulator were not available for direct execution and are not marked as passed.

## Defect log

| Test ID | Feature | Environment and reproduction | Expected | Actual before fix | Severity | Root cause | Fix and regression coverage | Verification | Evidence / commit | Remaining risk |
|---|---|---|---|---|---|---|---|---|---|---|
| DEF-001 | Release version identity | Linux sandbox; `cargo run --quiet --bin utharness -- --version` on the public-source checkout | CLI should report the coordinated current release version | Reported `utharness 0.1.0` while published Termux/native line was v0.2.6 | P1 | Workspace and launcher version constants had never been advanced after the release series moved beyond 0.1.0 | Coordinated Rust, UI, NPM, and Python metadata/constants to 0.2.7; added `cli_version_matches_cargo_package_version` integration regression | Local CLI test passed; full Rust and package tests passed | `crates/utharness-cli/tests/cli.rs`; published in v0.2.7 and covered by the release-version regression test | NPM and PyPI 0.2.7 publication and canonical-index installs were verified |
| DEF-002 | QA execution procedure | Linux sandbox; attempted `npm --prefix packages/utharnessly-npm pack` from repository root | QA command should run against the package directory | npm looked for a nonexistent root `package.json` | P3 process/tool invocation error, not a product defect | `npm --prefix ... pack` behavior did not match the intended package-directory invocation in this environment | Retried from `packages/utharnessly-npm`; no code change needed | NPM syntax, pack, clean install, and version checks passed | `/tmp/utharness-npm-pack-final.log` | Future QA should use an explicit package-directory subshell |
| DEF-003 | Python QA execution | Linux sandbox; attempted `python3 -m pytest` | Test runner should be available or documented | pytest was not installed | P3 process/tooling issue, not a product defect | Project tests use standard-library `unittest`; pytest was assumed by the QA command | Retried with `PYTHONPATH=... python3 -m unittest discover`; CI already uses unittest | Four Python tests passed; package build/install/version passed | `python/utharnessly/tests/test_cli.py` | Optional pytest integration is not maintained |
| DEF-004 | Rust-to-Ink PTY evidence | Linux sandbox; `python3 ui/test/rust_bridge_capture.py` | Harness should assert actual child lifecycle status | Harness printed `exit marker: False` despite a clean Ctrl+C exit because it searched for unrelated output | P3 evidence-harness defect, not a product defect | Exit-status was collected but ignored; marker assertion was stale | Harness now converts `waitpid` status to an exit code and asserts `exit status: 0`; rerun passed with banner, prompt, and clean-exit markers | `python3 ui/test/rust_bridge_capture.py` passed after fix | `ui/test/rust_bridge_capture.py`; `/tmp/utharness-rust-ink-bridge-final.txt` | PTY simulation is not a substitute for every terminal emulator or Android device |

## Passing test suites

| Test ID | Suite | Result | Evidence |
|---|---|---|---|
| PASS-001 | Rust format | Pass | `cargo fmt --all -- --check` |
| PASS-002 | Rust strict Clippy | Pass | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| PASS-003 | Rust workspace unit/integration/doc tests | Pass: 9 unit/doc test groups and 4 CLI integration tests | `cargo test --workspace` |
| PASS-004 | CLI version/help | Pass at source version 0.2.7 | `target/release/utharness --version`, `--help` |
| PASS-005 | UI typecheck | Pass | `pnpm --dir ui typecheck` |
| PASS-006 | UI tests | Pass: 5 tests | `pnpm --dir ui test` |
| PASS-007 | UI production build | Pass | `pnpm --dir ui build` |
| PASS-008 | Python unittest | Pass: 4 tests | `PYTHONPATH=python/utharnessly/src python3 -m unittest discover -s python/utharnessly/tests -v` |
| PASS-009 | Python wheel/sdist | Pass | `python3 -m build`; local virtualenv install and `utharnessly --version` |
| PASS-010 | NPM package | Pass | `node --check`, `npm pack --dry-run`, clean local install, `utharnessly --version` |
| PASS-011 | Termux package layout | Pass for Android-compatible hosted builds and host inspection | CI release jobs; local `.deb` layout/checksum checks |
| PASS-012 | Termux CLI simulation | Pass | Simulated `TERMUX_VERSION`/`PREFIX` setup, info, doctor, keys, update, uninstall, API missing/present branches |
| PASS-013 | Mock-provider autonomous E2E | Pass | Local OpenAI-compatible mock; plan parse, SAFE read-only steps, persistence, redaction check |
| PASS-014 | CLI safety matrix | Pass | SAFE shell denial, explicit trusted shell, destructive denylist, invalid session, memory empty state, init |
| PASS-015 | TUI PTY matrix | Pass | 40, 60, 80, 120, 160, and 220 columns; short-height layouts |
| PASS-016 | TUI visual inspection | Pass for inspected Linux screenshots | `docs/qa/screenshots/` and `docs/qa/visual-findings.md` |
| PASS-017 | Dependency audit | Pass where available | `pnpm audit --prod --audit-level high`; hosted `cargo audit`; local cargo-audit executable was not installed |
| PASS-018 | Hosted CI | Pass | [CI run 33107434874][1] |
| PASS-019 | Hosted Security | Pass | [Security run 33107731856][2] |
| PASS-020 | Published Termux repository | Pass | v0.2.7 Pages scripts, key identity, InRelease/Release.gpg signatures, package indexes, packages, and SHA-256 manifest verified |
| PASS-021 | NPM registry publication | Pass | `npm view utharnessly@0.2.7`, clean registry install, and `utharnessly --version` |
| PASS-022 | PyPI registry publication | Pass | Canonical PyPI JSON/Simple metadata, clean `pip install --index-url https://pypi.org/simple utharnessly==0.2.7`, and `utharnessly --version` |
| PASS-023 | v0.2.7 release workflow | Pass | [Release workflow 33107731796][3] built native and Android artifacts, signed metadata, deployed Pages, and uploaded assets |

## Test conditions and limitations

The Linux sandbox directly executed the Rust runtime, SQLite persistence, local mock-provider flow, NPM launcher, Python launcher, package builders, shell scripts, and Ink PTY captures. Hosted CI executed Linux, macOS, and Windows build/test jobs and Android cross-build/package jobs for Termux `aarch64` and `x86_64`.

No physical Android device or emulator was available. Consequently, Android 8/12/14 behavior, real Termux `pkg install`, Android soft-keyboard behavior, orientation changes, device low-RAM behavior, actual Termux:API app calls, and offline package-manager recovery are not marked as device-tested. iOS/iPadOS terminal applications, FreeBSD, ARM desktop targets, SSH transport, containers, and individual terminal emulators remain source/remote or unverified workflows.

## Release risk assessment

The highest remaining product risk is the absence of direct Android-device validation. The v0.2.7 native release, signed Termux repository, NPM package, and PyPI package are live and were verified from their public endpoints. No critical or high-severity defect is currently reproducible in the tested environments. No API key or signing private key is included in this report.

[1]: https://github.com/uthumany/utharnessly/actions/runs/33107434874 "Hosted CI run"
[2]: https://github.com/uthumany/utharnessly/actions/runs/33107731856 "Hosted Security run"
[3]: https://github.com/uthumany/utharnessly/actions/runs/33107731796 "v0.2.7 release workflow"
