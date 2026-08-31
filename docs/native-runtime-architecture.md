# Utharness Native Runtime and SQLite Persistence Architecture

**Status:** Proposed implementation baseline
**Product:** Utharness Agent Terminal CLI
**Primary binary:** `utharness`
**Scope:** Native Rust runtime, persistent local state, optional local API, and browser sidecar boundary

## 1. Executive decision

Utharness should be implemented as a **single native Rust process with explicit async boundaries**, backed by one local SQLite database and a small number of durable filesystem areas for configuration, secrets references, session exports, logs, and temporary artifacts. The process should expose two user surfaces over the same application core: the React/Ink terminal UI and scriptable CLI commands. An optional local Axum server should expose read/write application operations to the existing browser control plane and future integrations, but it must remain loopback-only by default.

The application should not be split into a daemon and client in the first release. A daemon can be added later without changing domain contracts because the CLI, TUI, and HTTP adapter will already depend on the same `utharness-core` service interfaces. This keeps installation, crash recovery, and local security simple while preserving a clean path to a durable broker when background jobs, remote clients, or multi-process access become necessary.

> **Design rule:** The TUI, CLI, and local API are adapters. They never own business state. All state transitions flow through application services, which persist durable facts before publishing events.

The runtime should use Tokio for network I/O, timers, provider streaming, child-process supervision, and task orchestration. Tokio provides the async runtime, scheduling, timers, channels, and I/O primitives required for these workloads, while blocking filesystem and SQLite work should be isolated from the async executor.[1] SQLite should use WAL mode for local read/write concurrency, with a single serialized writer path and short transactions. WAL permits readers and writers to proceed concurrently on the same host, but all database users must remain on that host and checkpointing must be managed deliberately.[2]

## 2. Constraints and non-goals

The design is derived from the supplied product requirements. Utharness is a standalone **Agent Terminal CLI**, not UTHARNESS OS. Version one must be local-first, usable without a hosted account, safe by default, recoverable after interruption, and extensible across providers, tools, skills, agents, memory, jobs, and MCP.

| Constraint | Architectural consequence |
| --- | --- |
| Native Rust runtime plus bundled UI | Keep domain logic and the CLI in Rust; keep the React/Ink terminal surface in the root `ui/` package and the browser sidecar separate. |
| Persistent full-screen TUI plus direct CLI | Use one application service layer with React/Ink and Clap adapters. |
| Local-first operation | Default all storage, logs, sessions, and memory to platform-specific local directories. |
| Multiple providers and local models | Normalize provider requests, capabilities, streaming events, errors, and usage accounting behind a gateway trait. |
| Tools can mutate the workspace | Every sensitive call passes through a permission decision and audit record before execution. |
| Long-running work and scheduled jobs | Persist task state, leases, checkpoints, and retry metadata; reconcile abandoned work at startup. |
| Session restoration | Store messages, tool events, task graph, context summaries, active model route, draft input, and UI state as durable records. |
| Browser automation is ecosystem-heavy | Isolate Playwright in `packages/browser-driver`; communicate through a versioned JSON protocol. |
| Existing browser control plane | Add a loopback-only Axum API and SSE event stream without coupling the core to React concepts. |
| Cross-platform distribution | Avoid OS-specific assumptions in domain crates; centralize paths, process control, keyring, and shell integration behind ports. |

Version one does not require a hosted backend, mandatory telemetry, a desktop GUI, a mobile application, or an always-on remote service. The scheduler is a **local in-process scheduler** for jobs owned by the current workspace. It should not use an external task service or a Manus-triggered execution path; scheduled jobs are deterministic local runtime behavior, while AI judgment remains inside the configured provider execution flow.

## 3. Workspace layout

```text
utharness/
├── Cargo.toml
├── crates/
│   ├── utharness-cli/          # Clap commands, exit codes, stdout/stderr contracts
│   ├── utharness-core/         # Domain types, service ports, event model, state machine
│   ├── utharness-agent/        # Planner, executor, verification, context assembly
│   ├── utharness-models/       # Model registry, capabilities, routing, usage metadata
│   ├── utharness-tools/        # Files, shell, PTY, Git, HTTP, browser, process tools
│   ├── utharness-memory/       # Memory writes, retrieval, FTS5 queries, compaction
│   ├── utharness-skills/       # Manifest loader, activation, validation, skill execution
│   ├── utharness-agents/       # Child-agent lifecycle, delegation, aggregation
│   ├── utharness-scheduler/    # Job parsing, due selection, leases, retry policy
│   ├── utharness-security/     # Policies, path sandbox, secret redaction, audit decisions
│   ├── utharness-config/       # TOML loading, defaults, migrations, platform paths
│   ├── utharness-storage/      # SQLite pool, migrations, repositories, transactions
│   ├── utharness-protocol/     # Versioned DTOs for local API and browser sidecar
│   └── utharness-server/       # Axum loopback API, SSE, health and diagnostics routes
├── ui/                         # TypeScript + React/Ink terminal UI
├── packages/
│   └── browser-driver/         # TypeScript + Playwright sidecar
├── migrations/                 # Embedded, append-only SQLite migrations
├── skills/builtin/             # Built-in skill manifests and resources
├── themes/                     # Semantic TOML theme files
├── tests/                      # Integration, snapshot, fixture, and protocol tests
├── docs/
└── scripts/
```

### 3.1 Dependency direction

`utharness-core` is the dependency center and must not depend on TUI, CLI, Axum, Playwright, or concrete storage. It defines domain entities, commands, events, errors, and ports. `utharness-storage` implements persistence ports. `utharness-agent`, `utharness-tools`, `utharness-memory`, `utharness-scheduler`, and `utharness-agents` implement application services using core ports. `utharness-cli`, `ui/`, and `utharness-server` are delivery adapters that compose the services in `utharness-cli`'s application bootstrap.

| Crate | Owns | Must not own |
| --- | --- | --- |
| `utharness-core` | IDs, commands, events, state machines, port traits | SQL, terminal drawing, HTTP, provider SDK details |
| `utharness-storage` | SQLite connections, migrations, repositories, transaction helpers | Agent policy, TUI state interpretation |
| `utharness-agent` | Agent loop and task orchestration | Direct terminal rendering or raw SQL |
| `utharness-tools` | Tool schemas and concrete execution | Bypassing permission engine |
| `utharness-security` | Authorization, sandboxing, redaction, audit | Provider selection or UI concerns |
| `ui/` | Layout, input, rendering, local view model | Direct file/shell/provider calls |
| `utharness-server` | API DTOs, authentication on loopback, SSE | Independent business rules |

### 3.2 Core ports

The first implementation should define traits in `utharness-core` and provide concrete implementations through dependency injection. The important ports are:

```rust
#[async_trait]
pub trait SessionStore {
    async fn create(&self, input: CreateSession) -> Result<SessionId>;
    async fn load(&self, id: SessionId) -> Result<SessionSnapshot>;
    async fn append_event(&self, event: NewEvent) -> Result<EventId>;
    async fn checkpoint(&self, checkpoint: NewCheckpoint) -> Result<CheckpointId>;
}

#[async_trait]
pub trait ProviderGateway {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse>;
    async fn stream(&self, request: ModelRequest) -> Result<ModelStream>;
    async fn health(&self, model: ModelRef) -> Result<ProviderHealth>;
}

#[async_trait]
pub trait ToolExecutor {
    async fn describe(&self, tool: ToolName) -> Result<ToolDescriptor>;
    async fn execute(&self, request: ToolRequest, ctx: ExecutionContext) -> Result<ToolResult>;
}

#[async_trait]
pub trait EventSink {
    async fn publish(&self, event: RuntimeEvent) -> Result<()>;
}
```

The concrete `AppContext` should contain shared handles to repositories, policy engine, provider gateway, tool registry, memory service, scheduler, and event bus. It should be constructed once at process startup and passed by `Arc` to adapters.

## 4. Runtime topology and event flow

```text
                 ┌─────────────────────────────┐
                 │ CLI / React/Ink / Axum adapter │
                 └──────────────┬──────────────┘
                                │ commands + subscriptions
                 ┌──────────────▼──────────────┐
                 │       utharness-core         │
                 │ application services/events  │
                 └───────┬───────────┬─────────┘
                         │           │
              ┌──────────▼───┐   ┌──▼─────────────┐
              │ Agent runtime │   │ Runtime event  │
              │ + task graph  │   │ bus            │
              └───┬─────┬──────┘   └──┬─────────────┘
                  │     │             │
        ┌─────────▼┐ ┌──▼─────────┐ ┌─▼─────────────┐
        │ Tool layer │ │ Provider  │ │ Storage writer │
        │ + policy   │ │ gateway   │ │ + repositories │
        └─────┬─────┘ └───────────┘ └──────┬─────────┘
              │                            │
        ┌─────▼──────────────┐     ┌───────▼────────┐
        │ shell/files/Git/PTY│     │ SQLite + FTS5   │
        │ HTTP/browser/MCP   │     │ WAL + migrations│
        └────────────────────┘     └────────────────┘
```

The event bus is in-process and typed. Commands cause a service to validate input, make the durable state transition, and publish an event containing the resulting ID and version. UI subscribers consume events to redraw. The event bus is not the source of truth; a restarted process rebuilds its view by reading SQLite and then resumes from durable task and lease state.

The recommended event envelope is:

```text
RuntimeEvent {
  event_id: UUIDv7,
  aggregate_type: session | task | agent | job | tool | memory | system,
  aggregate_id: UUID,
  sequence: i64,
  event_type: string,
  payload_json: JSON,
  created_at: timestamp,
  trace_id: UUID,
}
```

`sequence` is allocated per aggregate and enforced by a unique constraint. Consumers can discard duplicate or older events. This makes UI delivery and local API reconnects idempotent.

## 5. SQLite persistence design

### 5.1 Database placement and connection policy

Use one database at `~/.local/share/utharness/utharness.db`, or the platform equivalent returned by `utharness-config`. Workspace-specific records carry a `workspace_id` and canonical path. Do not create one database per session; that makes global search, diagnostics, migrations, and backups unnecessarily difficult.

`rusqlite` is the storage binding. It provides prepared statements, transactions, backup support, hooks, and SQLite type conversion suitable for repository implementations.[3] Because `rusqlite::Connection` is synchronous, the storage service should run its writer connection on a dedicated blocking thread. Read operations may use a small bounded pool of blocking connections. No async task should hold a SQLite transaction across a network call, provider stream, process wait, or terminal interaction.

At open time, every connection should apply:

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
PRAGMA trusted_schema = OFF;
PRAGMA temp_store = MEMORY;
```

`NORMAL` is an intentional local-performance choice; a `--durable-writes` configuration should switch to `FULL` for users who prefer stronger power-loss durability. A background checkpoint task should run `PRAGMA wal_checkpoint(PASSIVE)` during idle periods and diagnostics should report WAL size. Backups must copy the database through SQLite's backup API or after a clean checkpoint; copying only the main file while a WAL is active is unsafe because committed state can still be in the `-wal` file.[2]

### 5.2 Table groups

The schema is organized around durable facts rather than UI screens.

| Group | Tables | Purpose |
| --- | --- | --- |
| Identity and configuration | `app_meta`, `workspaces`, `settings`, `providers`, `models` | Installation, workspace, and model routing state. |
| Sessions and conversation | `sessions`, `messages`, `message_parts`, `context_snapshots`, `checkpoints` | Restorable conversations and execution context. |
| Work execution | `tasks`, `task_steps`, `task_dependencies`, `task_runs`, `runtime_events` | Plans, states, leases, retries, and event history. |
| Agents | `agents`, `agent_runs`, `agent_messages` | Child-agent lifecycle and delegation results. |
| Tools and security | `tool_calls`, `permission_decisions`, `audit_log` | Requested actions, authorization, and redacted outcomes. |
| Memory | `memories`, `memory_fts`, `memory_links` | Scoped records and full-text retrieval. |
| Extensibility | `skills`, `skill_runs`, `mcp_servers`, `mcp_tools` | Installed skills and configured MCP capabilities. |
| Automation | `jobs`, `job_runs` | Schedules, leases, retry policy, and run history. |
| Maintenance | `migrations` via `user_version`, `diagnostics`, `usage_records` | Upgrade and health metadata. |

### 5.3 Canonical schema

The first migration should create the following core tables. IDs are UUID text values generated in Rust, timestamps are UTC RFC 3339 strings or integer milliseconds consistently across the codebase; integer milliseconds are recommended for ordering and range queries.

```sql
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  canonical_path TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_opened_at INTEGER,
  settings_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active','paused','complete','failed','archived')),
  provider_id TEXT,
  model_id TEXT,
  cwd TEXT NOT NULL,
  theme TEXT NOT NULL,
  draft_input TEXT NOT NULL DEFAULT '',
  scroll_offset INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  closed_at INTEGER,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES messages(id),
  role TEXT NOT NULL CHECK (role IN ('user','assistant','system','tool')),
  content TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('streaming','complete','failed','cancelled')),
  provider_id TEXT,
  model_id TEXT,
  token_input INTEGER,
  token_output INTEGER,
  created_at INTEGER NOT NULL,
  completed_at INTEGER,
  sequence INTEGER NOT NULL,
  UNIQUE(session_id, sequence)
);

CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
  parent_id TEXT REFERENCES tasks(id),
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('queued','planning','running','waiting','paused','complete','failed','cancelled')),
  assigned_agent_id TEXT,
  lease_owner TEXT,
  lease_expires_at INTEGER,
  retry_count INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  started_at INTEGER,
  completed_at INTEGER,
  version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE task_steps (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('queued','active','complete','failed','skipped')),
  tool_call_id TEXT,
  result_json TEXT,
  UNIQUE(task_id, position)
);

CREATE TABLE runtime_events (
  id TEXT PRIMARY KEY,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  sequence INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  trace_id TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(aggregate_type, aggregate_id, sequence)
);

CREATE TABLE checkpoints (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
  label TEXT NOT NULL,
  git_revision TEXT,
  state_json TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
```

### 5.4 Memory and FTS5

Store memory records in a normal table and expose a content-linked FTS5 table for retrieval. FTS5 is SQLite's full-text search module and supports `MATCH`, relevance ordering, column filters, prefixes, phrases, and external-content tables.[4]

```sql
CREATE TABLE memories (
  id TEXT PRIMARY KEY,
  workspace_id TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
  session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
  agent_id TEXT,
  scope TEXT NOT NULL CHECK (scope IN ('working','session','project','long_term','preference','agent','task')),
  kind TEXT NOT NULL,
  content TEXT NOT NULL,
  source TEXT NOT NULL,
  importance REAL NOT NULL DEFAULT 0.5,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER
);

CREATE VIRTUAL TABLE memory_fts USING fts5(
  content,
  kind,
  source,
  content='memories',
  content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 1'
);
```

Triggers should keep `memory_fts` synchronized, or the repository should update both tables inside the same write transaction. Search must always scope by workspace and exclude soft-deleted records. Embeddings are optional and should not be required for the first migration; if added, store an embedding reference and model metadata rather than assuming a fixed vector dimension.

### 5.5 Jobs and leases

```sql
CREATE TABLE jobs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  schedule_kind TEXT NOT NULL CHECK (schedule_kind IN ('once','interval','cron')),
  schedule_expr TEXT NOT NULL,
  task_template_json TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  next_run_at INTEGER,
  last_run_at INTEGER,
  lease_owner TEXT,
  lease_expires_at INTEGER,
  retry_policy_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE job_runs (
  id TEXT PRIMARY KEY,
  job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  status TEXT NOT NULL CHECK (status IN ('claimed','running','complete','failed','skipped','cancelled')),
  scheduled_at INTEGER NOT NULL,
  started_at INTEGER,
  finished_at INTEGER,
  error_redacted TEXT,
  task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
  UNIQUE(job_id, scheduled_at)
);
```

The scheduler claims due jobs in a short `BEGIN IMMEDIATE` transaction by writing a lease owner and expiry, inserts a `job_runs` row, advances `next_run_at`, and commits before executing the task. A crashed process leaves an expired lease; the next scheduler pass can reclaim it and mark the prior run interrupted or retryable. This avoids duplicate execution when the scheduler loop wakes twice and avoids holding a transaction during AI or shell work.

## 6. Agent execution lifecycle

The agent runtime is a state machine, not an unstructured loop. A request follows this sequence:

1. Create or resume a session and append the user message.
2. Build a context snapshot from workspace instructions, recent messages, task state, selected memories, Git summary, and tool capabilities.
3. Persist a `planning` task and ordered `task_steps`.
4. Call the normalized provider gateway using the model router's selected route.
5. For every requested tool call, validate the schema and construct an `ExecutionContext` containing workspace, session, task, agent, requested scopes, and trace ID.
6. Ask the permission engine for a decision. If approval is required, persist `permission_decisions` and transition the task to `waiting`; do not execute the tool.
7. If allowed, persist a `tool_calls` record before execution, execute with timeout/cancellation, redact the result, then persist completion and publish a `ToolCompleted` event.
8. Feed the observation back to the model, continuing until the model returns a final response, the user cancels, a policy denies execution, or a retry budget is exhausted.
9. Run verification commands or checks defined by the task. Persist a checkpoint containing task state, relevant tool results, Git revision, model route, and memory writes.
10. Append the final assistant message, update session status, and publish `TaskCompleted` or `TaskFailed`.

All transitions should be validated by a small domain function such as `TaskState::transition(next)`. Invalid transitions return a typed error and are logged; they are never silently coerced.

Streaming provider output should be converted into typed events such as `AssistantDelta`, `ToolRequested`, `UsageUpdated`, `ProviderWarning`, and `ProviderCompleted`. The event bus may stream deltas to the TUI, but the storage layer should periodically persist a stream checkpoint and always persist the final canonical message. If the process crashes during streaming, the incomplete message remains `streaming` and startup reconciliation marks it `failed` or `interrupted` with a recoverable continuation marker.

## 7. Security and permission boundaries

The permission engine is a mandatory dependency of every mutating or externally visible tool. Modes are `SAFE`, `ASK`, `TRUSTED`, and `CUSTOM`. The engine evaluates both the tool category and the concrete request.

| Category | SAFE | ASK | TRUSTED | CUSTOM |
| --- | --- | --- | --- | --- |
| Read files inside workspace | Allow | Allow | Allow | Rule-based |
| Write or delete files | Deny | Prompt | Allow inside workspace | Rule-based |
| Shell execution | Allow only immutable diagnostics | Prompt and allowlist | Allow with denylist | Rule-based |
| Network and browser | Deny by default | Prompt per host/action | Allow configured scopes | Rule-based |
| Git commit/checkout | Deny | Prompt | Allow | Rule-based |
| Secrets/environment | Redacted/filtered | Prompt | Explicit key references only | Rule-based |

The file engine must canonicalize paths, reject traversal outside allowed workspace roots, enforce maximum file size, detect binary content, and deny protected paths such as `.git/objects`, credential files, and the Utharness database unless the user explicitly performs a maintenance command. Shell execution must use a sanitized environment, an explicit working directory, timeouts, process-group cancellation, stdout/stderr limits, and secret redaction before persistence or display.

Secrets should be referenced by provider ID and retrieved from the OS keyring or environment variables. API keys must never be stored in `providers` as plaintext or written to logs, events, checkpoints, model prompts, or tool output. Audit records should retain the decision, actor, tool, normalized target, policy mode, and redacted reason.

## 8. Local API and browser sidecar

`utharness-server` should bind to `127.0.0.1` on an automatically selected port, or use a Unix domain socket on Unix-like systems. It should publish a short-lived bearer token in the runtime directory and require it for non-health routes. Do not bind to `0.0.0.0` by default.

The initial API surface should be versioned under `/api/v1`:

| Route | Purpose |
| --- | --- |
| `GET /api/v1/health` | Liveness, schema version, runtime version. |
| `GET /api/v1/sessions` | List sessions for a workspace. |
| `GET /api/v1/sessions/:id` | Load a restorable session snapshot. |
| `POST /api/v1/sessions/:id/messages` | Submit a user message. |
| `POST /api/v1/tasks/:id/actions` | Run, pause, cancel, approve, or checkpoint. |
| `GET /api/v1/events` | SSE stream with `Last-Event-ID` replay support. |
| `GET /api/v1/files` | Workspace-scoped file listing/search. |
| `GET /api/v1/diagnostics` | Actionable health checks. |

The Playwright sidecar should receive a narrow protocol command (`open`, `snapshot`, `click`, `type`, `scroll`, `screenshot`, `download`, `close`) and return structured results. It must not receive arbitrary Rust internals or the SQLite file. Browser permissions should be evaluated before sidecar dispatch, and cookies/profiles should live in a separately permissioned data directory.

## 9. Startup, crash recovery, and shutdown

Startup should execute in this order: resolve platform paths, acquire a single-instance lock, open the database, run migrations, validate configuration, reconcile abandoned sessions/tasks/jobs, load the workspace, initialize provider and tool registries, start the scheduler, optionally start the local API, and finally enter the TUI or CLI command.

Reconciliation rules are deterministic:

| Record | Recovery action |
| --- | --- |
| `messages.status = streaming` | Mark interrupted and offer continuation from the last persisted context. |
| `tasks.status = running` with expired lease | Mark `paused` with reason `process_restarted`. |
| `tool_calls.status = running` | Mark `unknown` and require explicit user review; never replay automatically for mutating tools. |
| `jobs.lease_expires_at < now` | Release lease and apply retry policy. |
| Unfinished checkpoint write | Ignore incomplete transaction; SQLite rollback preserves the previous checkpoint. |

Graceful shutdown should stop accepting new commands, cancel or pause active work according to policy, flush event batches, write a session UI snapshot, run a passive WAL checkpoint, close providers and child processes, release the single-instance lock, and restore the terminal. `Ctrl+C` in the TUI should cancel the active operation first; a second interrupt may request process shutdown.

## 10. Observability and diagnostics

Use `tracing` with JSON logs written to the local logs directory and a compact human-readable layer for the TUI. Every command, model request, tool call, permission decision, scheduler claim, database migration, and recovery action receives a `trace_id`. Secret redaction must happen before the event enters a formatter or persistence repository.

`utharness doctor` should run actionable checks for version, paths, database integrity, migration level, WAL health, workspace accessibility, shell availability, Git repository state, provider configuration, model health, browser sidecar availability, installed skills, MCP configuration, permission policy, and network reachability. Each failure should include a remediation string and a severity.

## 11. Testing strategy

The test suite should be organized by risk and run without provider credentials by default.

| Layer | Tests |
| --- | --- |
| Core domain | State transitions, event sequencing, serialization, invalid command rejection. |
| Storage | Fresh migrations, upgrade migrations, foreign keys, rollback behavior, concurrent readers, WAL checkpointing, FTS5 synchronization, backup/restore. |
| Agent | Context assembly, tool-call loop, streaming deltas, cancellation, retry/failover, checkpoint creation, crash reconciliation. |
| Security | Path traversal, protected paths, command allow/deny lists, environment filtering, secret redaction, approval flow, audit records. |
| Tools | File size/binary handling, shell timeout, process cancellation, Git read/write policy, HTTP restrictions, browser protocol fixtures. |
| Scheduler | Once/interval/cron parsing, due selection, leases, duplicate suppression, retry/backoff, restart recovery. |
| TUI | Snapshot tests at `40x15`, `60x20`, `80x24`, `100x30`, `120x40`, and `160x50`; focus movement, overlays, ASCII fallback, resize. |
| API | Auth token, route validation, SSE reconnect/replay, workspace scoping, error contracts. |
| End-to-end | Initialize, configure local model fixture, create session, chat, inspect file fixture, run safe shell command, create memory, checkpoint, close, reopen, resume, run skill, schedule job, doctor. |

Provider adapters should use contract tests against a local mock server. Real provider tests should be opt-in and never run in pull requests. Property tests should target path normalization, cron next-run calculations, event sequence monotonicity, and redaction.

## 12. Implementation sequence

| Milestone | Deliverable | Exit criteria |
| --- | --- | --- |
| 0. Workspace foundation | Cargo workspace, error types, IDs, config paths, tracing, CI checks. | `cargo check`, formatting, lint, and empty binary work on Linux/macOS/Windows CI. |
| 1. Storage kernel | SQLite open policy, migrations, repositories, event log, backup command. | Fresh install and upgrade tests pass; sessions and messages survive restart. |
| 2. CLI and TUI shell | Clap commands, terminal lifecycle, React/Ink layout, keyboard routing, theme loading. | `utharness`, `init`, `version`, `sessions`, `resume`, `doctor` work without a provider. |
| 3. Local tools and security | File, shell, PTY, Git, policy engine, audit log. | Safe tools execute; denied and approval-required actions cannot bypass policy. |
| 4. Agent runtime | Context builder, planner, task graph, provider gateway, streaming, checkpoints. | Mock-provider end-to-end task completes, pauses, cancels, and resumes. |
| 5. Memory and skills | Memory CRUD/FTS5, scoped retrieval, built-in skill manifests and runner. | Memory search returns workspace-scoped ranked results; skills validate and run. |
| 6. Jobs and agents | Scheduler leases, job runs, child agents, result aggregation. | Restart and duplicate-wakeup tests show at-most-one claimed run per schedule slot. |
| 7. Browser/MCP/API | Protocol, Playwright sidecar, loopback API, SSE, MCP manager. | Browser fixtures, API auth, event replay, and MCP lifecycle tests pass. |
| 8. Release hardening | Installers, checksums, SBOM, signing where available, docs, artifact matrix. | Clean-machine acceptance flow passes for all supported launchers and target binaries. |

Each milestone follows the same loop: implement, format, lint, build, run focused tests, exercise the runtime, inspect logs, fix defects, run regression tests, and commit. A green compilation is not a completion signal; runtime behavior and recovery tests are required.

## 13. Definition of ready for implementation

Before writing milestone 0, freeze these contracts in code and documentation: serialized IDs and timestamps, task and session state enums, event envelope, permission decision shape, tool request/result schemas, provider streaming events, migration policy, filesystem path policy, API error format, and checkpoint payload version. Every later crate should depend on these contracts rather than inventing parallel representations.

The browser control plane already provides a useful visual target for the TUI and local API. Its local React state should eventually be replaced by SSE-backed projections from `runtime_events`, while all mutation controls should call the same service commands used by the CLI. This keeps the UI honest and ensures that native runtime behavior, persistence, and terminal behavior share one implementation.

## References

[1]: https://tokio.rs/tokio/tutorial "Tokio Tutorial — asynchronous Rust runtime"
[2]: https://sqlite.org/wal.html "SQLite Write-Ahead Logging"
[3]: https://docs.rs/rusqlite/ "rusqlite API documentation"
[4]: https://www.sqlite.org/fts5.html "SQLite FTS5 Extension"
