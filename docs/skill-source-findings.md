# Skill source findings

Research date: 2026-08-26.

## VoltAgent awesome-agent-skills

Source: https://github.com/VoltAgent/awesome-agent-skills

The public repository describes itself as a curated collection of 1000+ agent skills from official development teams and the community, compatible with Claude Code, Codex, Gemini CLI, Cursor, and other agent clients. Its README is a curated Markdown catalog that links to skill pages and source repositories, with a strong emphasis on official/team-created skills rather than mass-generated entries. The visible repository state during research showed the `main` branch, 578 commits, an MIT license, and public links to skill pages under `officialskills.sh`.

The adapter should therefore treat the repository README and its linked pages as catalog input, preserve the linked repository/page URL as provenance, extract the surrounding description and visible organization/category headings, and mark fields not present in the catalog as unknown instead of inventing runtime, dependencies, commands, permissions, or entrypoints.

## skills.sh

Sources: https://www.skills.sh/ and https://www.skills.sh/docs/api

The site describes skills as reusable capabilities for AI agents and advertises installation using `npx skills add <owner/repo>` and discovery using `npx skills find <query>`. The visible site shows support links for Claude Code, Cursor, Codex, GitHub Copilot, Windsurf, Gemini, Cline, AMP, Antigravity, OpenClaw, Droid, Goose, Kilo, Kiro CLI, Nous Research, OpenCode, Roo, Trae, VS Code, and Zed. The visible leaderboard showed 1,270,185 all-time activity and entries including `find-skills` from `vercel-labs/skills`, `frontend-design` from `anthropics/skills`, and `agent-browser` from `vercel-labs/agent-browser`.

The documented API base is `https://skills.sh` with JSON endpoints under `/api/v1/`. `GET /api/v1/skills` supports `view=all-time|trending|hot`, zero-indexed `page`, and `per_page` from 1 to 500. The response includes `id`, `slug`, `name`, `source`, `installs`, `sourceType`, `installUrl`, `url`, and pagination with `total` and `hasMore`. `GET /api/v1/skills/search` accepts `q` (minimum two characters), `limit` up to 200, and optional `owner`, and returns the same core identity/source/install fields plus query metadata. `GET /api/v1/skills/curated` returns grouped official skills and counts. `GET /api/v1/skills/:source/:skill` returns `id`, `source`, `slug`, `installs`, an optional content hash, and an optional file tree containing relative paths and full file contents. `GET /api/v1/skills/audit/:source/:skill` is authenticated and may return partner audit verdicts with pass/warn/fail status, risk levels, summaries, timestamps, and categories; it returns 404 when no audit exists.

The source adapter should use the public catalog/search/detail endpoints for incremental metadata indexing, persist the source hash and retrieval timestamp, store raw file contents only in a bounded cache, and treat audit data as optional enrichment. The registry should not claim that all listed skills are runnable in UTHARNESS: many skills are procedural Markdown content for external agent clients, and runtime/command/dependency fields must remain unknown until inspected or explicitly declared by a manifest.

## Implementation implications

A truthful 100,000+ registry can be represented by an indexed catalog of normalized metadata and provenance, while lazy installation fetches only a selected skill’s source. The initial importer should support a deterministic fixture and bounded sync in tests, with configurable pagination/page limits and clear sync status, rather than downloading 1.2M records or arbitrary repositories into the user machine. Unsafe or ambiguous skills should be quarantined or require explicit permission before installation/execution.

A live request to `GET https://skills.sh/api/v1/skills?view=all-time&page=0&per_page=1` returned HTTP 401 from the current environment. The public API documentation describes the endpoint and JSON response shape, but the adapter must treat authentication as configurable and report an unavailable/auth-required source rather than silently claiming a successful catalog sync. The public HTML catalog remains useful for bounded discovery and provenance, while full API synchronization should accept a configured bearer/OIDC token when available.
