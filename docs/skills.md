# UTHARNESS Skill Engine

The UTHARNESS Skill Engine turns procedural agent capabilities into searchable, reviewable, and lazily installable registry records. It keeps a lightweight SQLite/FTS catalog locally, installs selected skills under an isolated UTHARNESS-managed directory, records health and quarantine state, and refuses to execute arbitrary imported code without a reviewed adapter.

> **Design rule:** A catalog record is not proof that a skill is executable. Imported skills preserve unknown runtime, dependencies, commands, permissions, and entrypoints until those fields are declared or verified.

## Quick start

```bash
# List the built-in registry
utharness skills

# Search normalized metadata
utharness skills search testing

# Inspect the complete manifest for one skill
utharness skills info builtin.git-status

# Install a built-in skill lazily, test it, and run it
utharness skills install builtin.git-status
utharness skills test builtin.git-status
utharness skills run builtin.git-status

# Remove it; the prior installation is retained in quarantine for rollback inspection
utharness skills remove builtin.git-status

# Restore the newest quarantined installation if the replacement needs to be reverted
utharness skills rollback builtin.git-status

# Check registry health and list categories
utharness skills doctor
utharness skills categories
```

The interactive Ink UI exposes the same registry through `/skills` and `/skills search <query>`. `Ctrl+S` opens the Skill Manager overlay. The overlay reports indexed records, health, install state, and the review-gated behavior of external skills.

## Normalized manifest

Every imported or local skill is normalized into the following JSON-compatible shape:

```json
{
  "schemaVersion": 1,
  "id": "provider.owner.skill",
  "name": "Example Skill",
  "description": "Procedural capability description",
  "category": "coding",
  "source": {
    "provider": "github",
    "url": "https://github.com/owner/repository",
    "repository": "owner/repository",
    "commit": null
  },
  "version": "unknown",
  "runtime": ["node"],
  "entrypoint": null,
  "commands": [],
  "dependencies": [],
  "tools": [],
  "permissions": ["context.read"],
  "environment": [],
  "inputs": {},
  "outputs": {},
  "tags": [],
  "install": {
    "command": "npx skills add owner/repository",
    "working_directory": null,
    "package_manager": "npx"
  },
  "compatibility": {
    "operating_systems": [],
    "architectures": [],
    "agents": [],
    "notes": []
  },
  "license": null,
  "homepage": null,
  "documentation": null,
  "checksum": null,
  "updateSource": null
}
```

The registry stores the serialized manifest plus indexed columns for name, description, category, source, version, runtime, tags, status, health, installed version, failure reason, source hash, and timestamps. FTS5 indexes the searchable text. This is designed to handle a 100,000-plus record catalog without installing 100,000 dependency trees on a user machine.

## Source adapters

### VoltAgent curated catalog

The VoltAgent repository is a curated Markdown catalog of 1,000-plus skills from official development teams and the community. The adapter reads the public README, captures Markdown links and descriptions, preserves the link as provenance, tracks the current catalog group, and imports a bounded page of records. Fields that the catalog does not state remain `unknown` rather than being inferred as runnable commands or permissions.[1]

```bash
utharness skills sync --source voltagent --limit 500
```

### skills.sh

The public skills.sh documentation describes a paginated JSON API under `https://skills.sh/api/v1/`, including catalog listing, search, curated skills, skill detail, and optional security-audit endpoints. The adapter supports the documented all-time pagination shape and accepts `SKILLS_SH_TOKEN` when the deployment requires authentication. It reports HTTP 401 as a source-availability warning instead of returning an empty successful sync.[2]

```bash
# Bounded sync; safe for local development
utharness skills sync --source skills.sh --limit 500

# Full synchronization is intentionally explicit and should be scheduled outside the interactive CLI
SKILLS_SH_TOKEN="$TOKEN" utharness skills sync --source skills.sh --limit 100000
```

The current public web catalog presents an install convention based on `npx skills add <owner/repo>` and lists integrations for many agent clients, including Claude Code, Cursor, Codex, GitHub Copilot, Windsurf, Gemini, Cline, OpenCode, VS Code, and Zed.[3] UTHARNESS preserves this as an install hint, but it does not treat another client’s procedural skill as a trusted executable.

## Lazy installation and execution lifecycle

The lifecycle is deliberately staged:

1. **Discover.** Search the local SQLite/FTS registry or synchronize a bounded source page.
2. **Rank.** Combine text relevance with source, category, runtime, and local state. Autonomous planning includes the top local candidates in its planning context.
3. **Inspect.** Read the complete normalized manifest and review source provenance, permissions, runtime, dependencies, and health.
4. **Install.** Write the manifest and documentation into the UTHARNESS-managed installation directory. Existing content is moved into quarantine before replacement, and declared content checksums are verified.
5. **Test.** Re-evaluate operating-system, architecture, runtime, command, permission, and dependency compatibility.
6. **Activate.** Built-in skills use explicit UTHARNESS adapters. Imported skills remain metadata-only unless a reviewed adapter is available and the user explicitly permits external installation/execution.
7. **Unload.** Removal moves the active installation to a timestamped quarantine directory, allowing inspection or later rollback cleanup.

No imported shell command is executed merely because it appears in a manifest. Skills with unsafe permissions such as `system.root` or `credentials.export` are rejected by normalization. Dependency records are checked for duplicate names with conflicting pinned versions, and missing runtimes or commands produce explicit health states.

## Local and private skills

Create a `utharness.skill.json` manifest using the normalized shape, then import it into the local registry:

```bash
utharness skills import ./utharness.skill.json
utharness skills info local.example
```

This path is intended for workspace-owned, private, and internally reviewed skills. Importing a manifest does not execute its commands or install its dependencies.

## Compatibility and health states

| State | Meaning |
|---|---|
| `available` | Metadata is indexed but not installed. |
| `installed` | Manifest and documentation are present in the UTHARNESS-managed directory. |
| `healthy` | Declared platform, runtime, command, and permission checks pass for the current machine. |
| `manual` | The skill has no trusted executable entrypoint and is treated as procedural content. |
| `unknown` | The source did not publish enough data for compatibility verification. |
| `incompatible` | The declared operating system or architecture does not match the current host. |
| `quarantined` | The skill failed validation or testing and is withheld from normal execution. |
| `rollback` | The previous installation is restored from the timestamped quarantine snapshot. |
| `missing-runtime:*` / `missing-command:*` | A declared runtime or command is unavailable. |

## Scaling notes

The catalog is indexed, not materialized as local packages. A deployment can synchronize source pages incrementally, persist source hashes, and retain only bounded content caches. A 100,000-plus catalog therefore remains practical as registry metadata, while dependency installation is paid only for selected skills. Full source synchronization should run as an explicit, rate-limited job with a configured source credential where required; it should not run implicitly when a user opens the TUI.

## References

[1]: https://github.com/VoltAgent/awesome-agent-skills "VoltAgent awesome-agent-skills repository"

[2]: https://www.skills.sh/docs/api "skills.sh API Reference"

[3]: https://www.skills.sh/ "skills.sh Agent Skills Directory"
