# Native Backend Implementation Plan

## Completed in this repository

The first native runtime slice is deliberately small but real. It includes a focused Cargo workspace, embedded SQLite migrations, WAL-mode storage, persisted workspaces and sessions, ordered messages, tasks, checkpoints, runtime events, FTS5 memory, tool-call and audit tables, permission modes, workspace-aware shell execution, secret redaction, a Clap CLI, a React/Ink terminal UI, diagnostics, and process-level integration tests.

The offline planner is a deterministic provider-free mode. It makes the CLI useful on a clean machine and keeps tests credential-free. It is a provider boundary, not a pretend cloud integration.

## Next implementation sequence

| Milestone | Work | Verification |
| --- | --- | --- |
| Provider gateway | Add normalized OpenAI-compatible, Anthropic, Gemini, Ollama, and local HTTP adapters with streaming events and health checks. | Mock-provider contract tests and retry/failover tests. |
| Tool runtime | Add file read/write/edit, process supervision, PTY sessions, Git operations, HTTP, browser protocol, and capability schemas. | Permission, timeout, cancellation, redaction, and fixture tests. |
| Agent engine | Add context assembly, planner output schema, task steps, model continuation, tool observations, verification, and checkpoint resume. | Offline mock-provider E2E flow with pause, cancel, restart, and resume. |
| Scheduler | Add once, interval, and cron jobs with leases, retry policy, and persisted run history. | Duplicate wakeup and process restart tests. |
| Skills and MCP | Add manifest validation, built-in skill loader, activation state, MCP server lifecycle, tools, resources, and prompts. | Fixture servers and manifest tests. |
| Local API | Add loopback-only Axum API, bearer token, SSE event replay, and browser control-plane projection. | API auth, reconnect, workspace scoping, and event ordering tests. |
| Release engineering | Expand platform builds, publish signed shell/PowerShell archives, add npm/PyPI/JSR thin launchers where maintained, checksums, SBOM, signing, and release notes. | Clean-machine artifact and installer tests. |

## Definition of backend readiness

The runtime is ready for the next milestone when all existing tests pass, the new behavior has a focused unit or integration test, the CLI can exercise the behavior without UI-only shortcuts, errors are actionable, secrets are redacted, and a process restart does not lose committed state.
