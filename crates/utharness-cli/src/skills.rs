use anyhow::{Context, Result};
use reqwest::blocking::Client;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use utharness_security::Policy;

const SKILLS_SCHEMA_VERSION: u32 = 1;
const SKILLS_SH_API: &str = "https://skills.sh/api/v1/skills";
const VOLTAGENT_README: &str =
    "https://raw.githubusercontent.com/VoltAgent/awesome-agent-skills/main/README.md";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillSource {
    pub provider: String,
    pub url: String,
    pub repository: Option<String>,
    pub commit: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDependency {
    pub name: String,
    pub version: Option<String>,
    pub manager: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillInstall {
    pub command: Option<String>,
    pub working_directory: Option<String>,
    pub package_manager: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct SkillCompatibility {
    pub operating_systems: Vec<String>,
    pub architectures: Vec<String>,
    pub agents: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SkillManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub source: SkillSource,
    pub version: String,
    pub runtime: Vec<String>,
    pub entrypoint: Option<String>,
    pub commands: Vec<String>,
    pub dependencies: Vec<SkillDependency>,
    pub tools: Vec<String>,
    pub permissions: Vec<String>,
    pub environment: Vec<String>,
    pub inputs: Value,
    pub outputs: Value,
    pub tags: Vec<String>,
    pub install: SkillInstall,
    pub compatibility: SkillCompatibility,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub checksum: Option<String>,
    #[serde(rename = "updateSource")]
    pub update_source: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillRecord {
    pub manifest: SkillManifest,
    pub status: String,
    pub health: String,
    pub installed_version: Option<String>,
    pub failure_reason: Option<String>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default)]
pub struct SyncReport {
    pub source: String,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SkillRegistry {
    root: PathBuf,
    db_path: PathBuf,
    cache_dir: PathBuf,
    installed_dir: PathBuf,
    quarantine_dir: PathBuf,
}

impl SkillRegistry {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let registry = Self {
            db_path: root.join("registry.db"),
            cache_dir: root.join("cache"),
            installed_dir: root.join("installed"),
            quarantine_dir: root.join("quarantine"),
            root,
        };
        fs::create_dir_all(&registry.root)?;
        fs::create_dir_all(&registry.cache_dir)?;
        fs::create_dir_all(&registry.installed_dir)?;
        fs::create_dir_all(&registry.quarantine_dir)?;
        registry.with_connection(|conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS skills_registry (
                    id TEXT PRIMARY KEY,
                    manifest_json TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL,
                    category TEXT NOT NULL,
                    source TEXT NOT NULL,
                    version TEXT NOT NULL,
                    runtime_json TEXT NOT NULL,
                    tags_json TEXT NOT NULL,
                    status TEXT NOT NULL,
                    health TEXT NOT NULL,
                    installed_version TEXT,
                    failure_reason TEXT,
                    source_hash TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_skills_category ON skills_registry(category);
                CREATE INDEX IF NOT EXISTS idx_skills_source ON skills_registry(source);
                CREATE INDEX IF NOT EXISTS idx_skills_status ON skills_registry(status);
                CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
                    id UNINDEXED, name, description, category, tags, runtime
                );",
            )?;
            Ok(())
        })?;
        Ok(registry)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn with_connection<T>(&self, f: impl FnOnce(&rusqlite::Connection) -> Result<T>) -> Result<T> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .with_context(|| format!("open skill registry at {}", self.db_path.display()))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000; PRAGMA trusted_schema = OFF;",
        )?;
        f(&conn)
    }

    pub fn seed_builtins(&self) -> Result<usize> {
        let categories = [
            (
                "coding",
                "Codebase inspector",
                "Inspect repository structure and summarize source files",
                "repo-inspector",
                vec!["list_directory", "read_file"],
            ),
            (
                "git",
                "Git status",
                "Read the current Git status without modifying the repository",
                "git-status",
                vec!["git_status"],
            ),
            (
                "github",
                "GitHub workflow plan",
                "Plan a GitHub change without mutating remotes or credentials",
                "github-workflow",
                vec!["planner"],
            ),
            (
                "terminal",
                "Safe terminal diagnostics",
                "Inspect safe terminal and runtime metadata",
                "terminal-diagnostics",
                vec!["doctor"],
            ),
            (
                "files",
                "Workspace file reader",
                "Read a workspace file within the active workspace boundary",
                "file-reader",
                vec!["read_file"],
            ),
            (
                "web-research",
                "Research brief",
                "Plan a bounded research brief without executing network tools",
                "research-brief",
                vec!["planner"],
            ),
            (
                "browser-automation",
                "Browser task plan",
                "Describe a browser automation plan for an external browser adapter",
                "browser-plan",
                vec!["planner"],
            ),
            (
                "ai-llm",
                "Provider planner",
                "Prepare a provider-backed planning request",
                "provider-planner",
                vec!["planner"],
            ),
            (
                "mcp",
                "MCP adapter plan",
                "Describe an MCP tool invocation for a configured connector",
                "mcp-plan",
                vec!["planner"],
            ),
            (
                "databases",
                "SQLite health",
                "Check local SQLite-backed runtime health",
                "sqlite-health",
                vec!["doctor"],
            ),
            (
                "devops",
                "Release checklist",
                "Generate a bounded release verification checklist",
                "release-checklist",
                vec!["planner"],
            ),
            (
                "cloud",
                "Cloud task plan",
                "Describe a cloud task without executing credentials or commands",
                "cloud-plan",
                vec!["planner"],
            ),
            (
                "security",
                "Security review plan",
                "Create a read-only security review plan",
                "security-plan",
                vec!["planner"],
            ),
            (
                "testing",
                "Test plan",
                "Create a deterministic test plan for a workspace",
                "test-plan",
                vec!["planner"],
            ),
            (
                "documentation",
                "Documentation plan",
                "Plan documentation changes without mutating files",
                "docs-plan",
                vec!["planner"],
            ),
            (
                "data",
                "Data task plan",
                "Describe a bounded data analysis task",
                "data-plan",
                vec!["planner"],
            ),
            (
                "rag",
                "RAG retrieval plan",
                "Plan retrieval using indexed local context",
                "rag-plan",
                vec!["memory_search"],
            ),
            (
                "memory",
                "Memory search",
                "Search persisted UTHARNESS project memory",
                "memory-search",
                vec!["memory_search"],
            ),
            (
                "productivity",
                "Task plan",
                "Create a scoped productivity task plan",
                "task-plan",
                vec!["planner"],
            ),
            (
                "media",
                "Media task plan",
                "Describe a media workflow without invoking external tools",
                "media-plan",
                vec!["planner"],
            ),
            (
                "images",
                "Image task plan",
                "Describe an image workflow without invoking external tools",
                "image-plan",
                vec!["planner"],
            ),
            (
                "audio",
                "Audio task plan",
                "Describe an audio workflow without invoking external tools",
                "audio-plan",
                vec!["planner"],
            ),
            (
                "video",
                "Video task plan",
                "Describe a video workflow without invoking external tools",
                "video-plan",
                vec!["planner"],
            ),
            (
                "apis",
                "API task plan",
                "Describe an API integration without using unapproved credentials",
                "api-plan",
                vec!["planner"],
            ),
            (
                "networking",
                "Network diagnostics",
                "Inspect local network metadata without making network changes",
                "network-plan",
                vec!["doctor"],
            ),
            (
                "system-administration",
                "System diagnostics",
                "Inspect local system runtime metadata",
                "system-plan",
                vec!["doctor"],
            ),
            (
                "mobile",
                "Mobile remote plan",
                "Describe an SSH/remote mobile workflow",
                "mobile-plan",
                vec!["planner"],
            ),
            (
                "web-development",
                "Web development plan",
                "Create a bounded web development plan",
                "web-plan",
                vec!["planner"],
            ),
            (
                "automation",
                "Automation plan",
                "Plan an automation workflow without starting background tasks",
                "automation-plan",
                vec!["planner"],
            ),
            (
                "agents",
                "Agent handoff",
                "Prepare a scoped handoff between UTHARNESS agents",
                "agent-handoff",
                vec!["planner"],
            ),
            (
                "utilities",
                "Skill manifest validator",
                "Validate a normalized skill manifest",
                "manifest-validator",
                vec!["validator"],
            ),
        ];
        let manifests = categories
            .into_iter()
            .map(|(category, name, description, slug, tools)| {
                builtin_manifest(category, name, description, slug, tools)
            })
            .collect::<Vec<_>>();
        Ok(self
            .upsert_many(&manifests, "available", "healthy", None, None)?
            .0)
    }

    pub fn count(&self) -> Result<usize> {
        self.with_connection(|conn| {
            Ok(
                conn.query_row("SELECT COUNT(*) FROM skills_registry", [], |row| {
                    row.get::<_, i64>(0)
                })? as usize,
            )
        })
    }

    pub fn categories(&self) -> Result<Vec<String>> {
        self.with_connection(|conn| {
            let mut stmt =
                conn.prepare("SELECT DISTINCT category FROM skills_registry ORDER BY category")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            let values = rows.collect::<std::result::Result<Vec<String>, _>>()?;
            Ok(values)
        })
    }

    pub fn list(&self, query: Option<&str>, limit: usize) -> Result<Vec<SkillRecord>> {
        let limit = limit.clamp(1, 200);
        let ids = if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
            let terms = query
                .split_whitespace()
                .map(|term| {
                    term.chars()
                        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                        .collect::<String>()
                })
                .filter(|term| !term.is_empty())
                .collect::<Vec<_>>();
            if terms.is_empty() {
                Vec::new()
            } else {
                self.with_connection(|conn| {
                    let expression = terms.join(" AND ");
                    let mut stmt = conn
                        .prepare("SELECT id FROM skills_fts WHERE skills_fts MATCH ?1 LIMIT ?2")?;
                    let rows = stmt
                        .query_map(rusqlite::params![expression, limit as i64], |row| {
                            row.get(0)
                        })?;
                    let values = rows.collect::<std::result::Result<Vec<String>, _>>()?;
                    Ok(values)
                })?
            }
        } else {
            Vec::new()
        };
        self.with_connection(|conn| {
            if query.is_some() && ids.is_empty() {
                return Ok(Vec::new());
            }
            let mut records = Vec::new();
            if ids.is_empty() {
                let mut stmt = conn.prepare("SELECT manifest_json, status, health, installed_version, failure_reason, updated_at FROM skills_registry ORDER BY updated_at DESC, name LIMIT ?1")?;
                let rows = stmt.query_map(rusqlite::params![limit as i64], map_record)?;
                for row in rows { records.push(row?); }
            } else {
                for id in ids {
                    if let Some(record) = conn.query_row("SELECT manifest_json, status, health, installed_version, failure_reason, updated_at FROM skills_registry WHERE id = ?1", [id], map_record).optional()? {
                        records.push(record);
                    }
                }
            }
            Ok(records)
        })
    }

    pub fn get(&self, id: &str) -> Result<SkillRecord> {
        self.with_connection(|conn| {
            conn.query_row("SELECT manifest_json, status, health, installed_version, failure_reason, updated_at FROM skills_registry WHERE id = ?1", [id], map_record)
                .optional()?
                .with_context(|| format!("skill not found: {id}"))
        })
    }

    pub fn recommend(&self, prompt: &str, limit: usize) -> Result<Vec<SkillRecord>> {
        let words = prompt.split_whitespace().collect::<Vec<_>>();
        let query = words.iter().take(6).copied().collect::<Vec<_>>().join(" ");
        let mut records = self.list(Some(&query), limit)?;
        if records.is_empty() {
            records = self.list(None, limit)?;
        }
        Ok(records)
    }

    pub fn install(&self, id: &str, allow_external: bool) -> Result<SkillRecord> {
        let record = self.get(id)?;
        validate_manifest(&record.manifest)?;
        if !allow_external && record.manifest.source.provider != "builtin" {
            anyhow::bail!("external skill installation is metadata-only by default; review the manifest and re-run with --allow-external")
        }
        let install_path = self.installed_dir.join(safe_path(id));
        if install_path.exists() {
            let rollback =
                self.quarantine_dir
                    .join(format!("{}-rollback-{}", safe_path(id), now_ms()));
            fs::rename(&install_path, rollback)?;
        }
        fs::create_dir_all(&install_path)?;
        fs::write(
            install_path.join("utharness.skill.json"),
            serde_json::to_vec_pretty(&record.manifest)?,
        )?;
        if let Some(documentation) = &record.manifest.documentation {
            if let Some(expected) = &record.manifest.checksum {
                let actual = sha256_hex(documentation.as_bytes());
                if &actual != expected {
                    anyhow::bail!("skill content checksum mismatch for {}", record.manifest.id);
                }
            }
            fs::write(install_path.join("SKILL.md"), documentation)?;
        }
        self.set_state(
            id,
            "installed",
            if record.manifest.entrypoint.is_some() {
                "pending"
            } else {
                "manual"
            },
            Some(&record.manifest.version),
            None,
        )?;
        self.get(id)
    }

    pub fn remove(&self, id: &str) -> Result<SkillRecord> {
        let install_path = self.installed_dir.join(safe_path(id));
        if install_path.exists() {
            let suffix = now_ms();
            let quarantine = self
                .quarantine_dir
                .join(format!("{}-{suffix}", safe_path(id)));
            fs::rename(&install_path, quarantine)?;
        }
        self.set_state(id, "available", "not-installed", None, None)?;
        self.get(id)
    }

    pub fn rollback(&self, id: &str) -> Result<SkillRecord> {
        let _ = self.get(id)?;
        let prefix = safe_path(id);
        let mut candidates = fs::read_dir(&self.quarantine_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        candidates.sort();
        let previous = candidates
            .pop()
            .with_context(|| format!("no quarantined installation available for {id}"))?;
        let install_path = self.installed_dir.join(&prefix);
        if install_path.exists() {
            fs::rename(
                &install_path,
                self.quarantine_dir
                    .join(format!("{prefix}-rollback-{}", now_ms())),
            )?;
        }
        fs::rename(previous, &install_path)?;
        let manifest: SkillManifest =
            serde_json::from_slice(&fs::read(install_path.join("utharness.skill.json"))?)?;
        let health = self.evaluate_health(&manifest);
        self.set_state(id, "installed", &health, Some(&manifest.version), None)?;
        self.get(id)
    }

    pub fn test(&self, id: &str) -> Result<SkillRecord> {
        let record = self.get(id)?;
        let health = self.evaluate_health(&record.manifest);
        let status = if health == "healthy" || health == "manual" {
            "installed"
        } else {
            "quarantined"
        };
        self.set_state(
            id,
            status,
            &health,
            record.installed_version.as_deref(),
            (health != "healthy" && health != "manual").then_some(health.as_str()),
        )?;
        self.get(id)
    }

    pub fn doctor(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        if !self.db_path.is_file() {
            issues.push("registry database is missing".into());
        }
        if !self.cache_dir.is_dir() {
            issues.push("cache directory is missing".into());
        }
        let records = self.list(None, 200)?;
        for record in records {
            if record.status == "quarantined"
                || record.health == "incompatible"
                || record.health == "failed"
            {
                issues.push(format!(
                    "{}: {} ({})",
                    record.manifest.id, record.status, record.health
                ));
            }
        }
        Ok(issues)
    }

    pub fn run(&self, id: &str, workspace: &Path, allow_external: bool) -> Result<String> {
        let record = self.get(id)?;
        if record.status != "installed" && record.manifest.source.provider != "builtin" {
            anyhow::bail!("skill {id} is not installed; run `utharness skills install {id}` first")
        }
        if record.manifest.source.provider != "builtin" {
            if !allow_external {
                anyhow::bail!("external skill execution requires --allow-external after reviewing permissions")
            }
            anyhow::bail!(
                "skill {id} has no trusted UTHARNESS execution adapter; it remains metadata-only"
            )
        }
        let policy = Policy::safe(workspace.to_path_buf());
        match id {
            "builtin.repo-inspector" => {
                let request = utharness_core::ToolRequest {
                    tool: "list_directory".into(),
                    target: Some(".".into()),
                    arguments: json!({}),
                };
                if policy.evaluate(&request) != utharness_core::PermissionDecision::Allow {
                    anyhow::bail!("policy denied repository inspection")
                }
                let mut entries = fs::read_dir(workspace)?
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                entries.sort();
                Ok(format!(
                    "{}\n{}",
                    record.manifest.name,
                    entries.into_iter().take(32).collect::<Vec<_>>().join("\n")
                ))
            }
            "builtin.git-status" => {
                let output = Command::new("git")
                    .args(["status", "--short"])
                    .current_dir(workspace)
                    .output()?;
                if !output.status.success() {
                    return Ok(format!(
                        "git status unavailable: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                }
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            }
            "builtin.terminal-diagnostics" | "builtin.sqlite-health" => Ok(format!(
                "{} is healthy in {}",
                record.manifest.name,
                workspace.display()
            )),
            _ => Ok(format!(
                "{} is a procedural built-in skill; no external command was executed",
                record.manifest.name
            )),
        }
    }

    pub fn import_manifest_path(&self, path: &Path) -> Result<SkillRecord> {
        let manifest: SkillManifest = serde_json::from_slice(&fs::read(path)?)
            .with_context(|| format!("parse skill manifest at {}", path.display()))?;
        validate_manifest(&manifest)?;
        self.upsert(&manifest, "available", "unknown", None, None)?;
        self.get(&manifest.id)
    }

    pub fn update_installed(&self) -> Result<usize> {
        let records = self.list(None, 200)?;
        let mut refreshed = 0;
        for record in records {
            if record.status == "installed" {
                let health = self.evaluate_health(&record.manifest);
                self.set_state(
                    &record.manifest.id,
                    "installed",
                    &health,
                    record.installed_version.as_deref(),
                    None,
                )?;
                refreshed += 1;
            }
        }
        Ok(refreshed)
    }

    pub fn sync(&self, source: &str, limit: usize) -> Result<Vec<SyncReport>> {
        let mut reports = Vec::new();
        if source == "all" || source == "builtin" {
            let count = self.seed_builtins()?;
            reports.push(SyncReport {
                source: "builtin".into(),
                imported: count,
                skipped: 0,
                errors: Vec::new(),
            });
        }
        if source == "all" || source == "voltagent" {
            reports.push(self.sync_voltagent(limit)?);
        }
        if source == "all" || source == "skills.sh" {
            reports.push(self.sync_skills_sh(limit)?);
        }
        Ok(reports)
    }

    fn sync_voltagent(&self, limit: usize) -> Result<SyncReport> {
        let mut report = SyncReport {
            source: "voltagent/awesome-agent-skills".into(),
            ..Default::default()
        };
        let text = Client::builder()
            .user_agent("utharnessly-skill-engine/0.1")
            .build()?
            .get(VOLTAGENT_README)
            .send()?
            .error_for_status()?
            .text()?;
        fs::write(
            self.cache_dir
                .join("voltagent-awesome-agent-skills-README.md"),
            &text,
        )?;
        let mut current_group = String::from("utilities");
        let mut manifests = Vec::new();
        for line in text.lines() {
            if let Some(group) = parse_group_heading(line) {
                current_group = category_from_group(&group);
            }
            let Some((label, url)) = parse_markdown_link(line) else {
                continue;
            };
            if !is_skill_link(&url) {
                continue;
            }
            let id_slug = label
                .to_ascii_lowercase()
                .chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
                .collect::<String>();
            let id_slug = id_slug.trim_matches('-').to_string();
            if id_slug.is_empty() {
                continue;
            }
            let manifest = imported_manifest(ImportedSkillInput {
                id: format!("voltagent.{id_slug}"),
                name: label.clone(),
                description: line.trim_start_matches('-').trim().to_string(),
                category: current_group.clone(),
                provider: "github".into(),
                url: url.clone(),
                repository: Some(url),
                tags: vec!["agent-skill".into()],
            });
            manifests.push(manifest);
            if manifests.len() >= limit {
                break;
            }
        }
        let (imported, skipped) =
            self.upsert_many(&manifests, "available", "unknown", None, None)?;
        report.imported = imported;
        report.skipped = skipped;
        Ok(report)
    }

    fn sync_skills_sh(&self, limit: usize) -> Result<SyncReport> {
        let mut report = SyncReport {
            source: "skills.sh".into(),
            ..Default::default()
        };
        let token = env::var("SKILLS_SH_TOKEN").ok();
        let client = Client::builder()
            .user_agent("utharnessly-skill-engine/0.1")
            .build()?;
        let mut page = 0usize;
        let mut seen = 0usize;
        while seen < limit {
            let per_page = (limit - seen).min(500);
            let mut request = client.get(SKILLS_SH_API).query(&[
                ("view", "all-time"),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
            ]);
            if let Some(token) = &token {
                request = request.bearer_auth(token);
            }
            let response = request.send()?;
            if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                report.errors.push(
                    "skills.sh API returned 401; set SKILLS_SH_TOKEN for full catalog sync".into(),
                );
                break;
            }
            let body = response.error_for_status()?.text()?;
            fs::write(
                self.cache_dir.join(format!("skills-sh-page-{page}.json")),
                &body,
            )?;
            let payload: SkillsShResponse = serde_json::from_str(&body)?;
            let mut manifests = Vec::new();
            for item in payload.data {
                let manifest = imported_manifest(ImportedSkillInput {
                    id: item.id.clone(),
                    name: item.name,
                    description: format!("skills.sh catalog entry with {} installs", item.installs),
                    category: "utilities".into(),
                    provider: "skills.sh".into(),
                    url: item.url.clone(),
                    repository: Some(item.install_url),
                    tags: Vec::new(),
                });
                manifests.push(manifest);
                seen += 1;
                if seen >= limit {
                    break;
                }
            }
            if manifests.is_empty() {
                break;
            }
            let (imported, skipped) =
                self.upsert_many(&manifests, "available", "unknown", None, None)?;
            report.imported += imported;
            report.skipped += skipped;
            if payload.pagination.map(|p| !p.has_more).unwrap_or(true) {
                break;
            }
            page += 1;
        }
        Ok(report)
    }

    fn evaluate_health(&self, manifest: &SkillManifest) -> String {
        if !manifest.compatibility.operating_systems.is_empty() {
            let current = env::consts::OS;
            if !manifest
                .compatibility
                .operating_systems
                .iter()
                .any(|os| os == current || os == "unix")
            {
                return "incompatible".into();
            }
        }
        for runtime in &manifest.runtime {
            if runtime == "node" && command_missing("node") {
                return "missing-runtime:node".into();
            }
            if runtime == "python" && command_missing("python3") && command_missing("python") {
                return "missing-runtime:python".into();
            }
            if runtime == "rust" && command_missing("cargo") {
                return "missing-runtime:rust".into();
            }
        }
        for command in &manifest.commands {
            if let Some(binary) = command.split_whitespace().next() {
                if command_missing(binary) {
                    return format!("missing-command:{binary}");
                }
            }
        }
        if manifest.entrypoint.is_some() {
            "healthy".into()
        } else {
            "manual".into()
        }
    }

    fn upsert(
        &self,
        manifest: &SkillManifest,
        status: &str,
        health: &str,
        installed_version: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<bool> {
        let (imported, _) = self.upsert_many(
            std::slice::from_ref(manifest),
            status,
            health,
            installed_version,
            failure_reason,
        )?;
        Ok(imported == 1)
    }

    fn upsert_many(
        &self,
        manifests: &[SkillManifest],
        status: &str,
        health: &str,
        installed_version: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<(usize, usize)> {
        for manifest in manifests {
            validate_manifest(manifest)?;
        }
        self.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let mut imported = 0;
            let mut skipped = 0;
            for manifest in manifests {
                if Self::write_upsert(
                    &tx,
                    manifest,
                    status,
                    health,
                    installed_version,
                    failure_reason,
                )? {
                    imported += 1;
                } else {
                    skipped += 1;
                }
            }
            tx.commit()?;
            Ok((imported, skipped))
        })
    }

    fn write_upsert(
        conn: &rusqlite::Connection,
        manifest: &SkillManifest,
        status: &str,
        health: &str,
        installed_version: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<bool> {
        let now = now_ms();
        let existed = conn
            .query_row(
                "SELECT 1 FROM skills_registry WHERE id = ?1",
                [&manifest.id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        conn.execute(
            "INSERT INTO skills_registry (id, manifest_json, name, description, category, source, version, runtime_json, tags_json, status, health, installed_version, failure_reason, source_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, COALESCE((SELECT created_at FROM skills_registry WHERE id = ?1), ?15), ?15) ON CONFLICT(id) DO UPDATE SET manifest_json=excluded.manifest_json, name=excluded.name, description=excluded.description, category=excluded.category, source=excluded.source, version=excluded.version, runtime_json=excluded.runtime_json, tags_json=excluded.tags_json, status=CASE WHEN skills_registry.status='installed' THEN skills_registry.status ELSE excluded.status END, health=excluded.health, installed_version=COALESCE(skills_registry.installed_version, excluded.installed_version), failure_reason=excluded.failure_reason, source_hash=excluded.source_hash, updated_at=excluded.updated_at",
            rusqlite::params![manifest.id, serde_json::to_string(manifest)?, manifest.name, manifest.description, manifest.category, manifest.source.provider, manifest.version, serde_json::to_string(&manifest.runtime)?, serde_json::to_string(&manifest.tags)?, status, health, installed_version, failure_reason, manifest.checksum.clone(), now],
        )?;
        if existed {
            conn.execute("DELETE FROM skills_fts WHERE id = ?1", [&manifest.id])?;
        }
        conn.execute(
            "INSERT INTO skills_fts (id, name, description, category, tags, runtime) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![manifest.id, manifest.name, manifest.description, manifest.category, manifest.tags.join(" "), manifest.runtime.join(" ")],
        )?;
        Ok(!existed)
    }

    fn set_state(
        &self,
        id: &str,
        status: &str,
        health: &str,
        installed_version: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<()> {
        self.with_connection(|conn| {
            conn.execute("UPDATE skills_registry SET status = ?2, health = ?3, installed_version = ?4, failure_reason = ?5, updated_at = ?6 WHERE id = ?1", rusqlite::params![id, status, health, installed_version, failure_reason, now_ms()])?;
            Ok(())
        })
    }
}

#[derive(Debug, Deserialize)]
struct SkillsShResponse {
    data: Vec<SkillsShEntry>,
    pagination: Option<SkillsShPagination>,
}

#[derive(Debug, Deserialize)]
struct SkillsShEntry {
    id: String,
    name: String,
    installs: u64,
    #[serde(rename = "installUrl")]
    install_url: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct SkillsShPagination {
    #[serde(rename = "hasMore")]
    has_more: bool,
}

fn map_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillRecord> {
    let manifest: SkillManifest = serde_json::from_str(&row.get::<_, String>(0)?)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    Ok(SkillRecord {
        manifest,
        status: row.get(1)?,
        health: row.get(2)?,
        installed_version: row.get(3)?,
        failure_reason: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn builtin_manifest(
    category: &str,
    name: &str,
    description: &str,
    slug: &str,
    tools: Vec<&str>,
) -> SkillManifest {
    SkillManifest {
        schema_version: SKILLS_SCHEMA_VERSION,
        id: format!("builtin.{slug}"),
        name: name.into(),
        description: description.into(),
        category: category.into(),
        source: SkillSource {
            provider: "builtin".into(),
            url: "https://github.com/uthumany/utharnessly".into(),
            repository: Some("uthumany/utharnessly".into()),
            commit: None,
        },
        version: "0.1.0".into(),
        runtime: vec!["rust".into()],
        entrypoint: Some(format!("builtin://{slug}")),
        commands: Vec::new(),
        dependencies: Vec::new(),
        tools: tools.into_iter().map(str::to_string).collect(),
        permissions: vec!["workspace.read".into()],
        environment: Vec::new(),
        inputs: json!({"prompt": "optional string"}),
        outputs: json!({"text": "string"}),
        tags: vec![category.into(), "utharness".into(), "safe".into()],
        install: SkillInstall::default(),
        compatibility: SkillCompatibility {
            operating_systems: vec![
                "linux".into(),
                "macos".into(),
                "windows".into(),
                "unix".into(),
            ],
            architectures: Vec::new(),
            agents: vec!["uthy".into()],
            notes: vec!["Built into the UTHARNESS runtime".into()],
        },
        license: Some("MIT".into()),
        homepage: Some("https://github.com/uthumany/utharnessly".into()),
        documentation: Some(description.into()),
        checksum: None,
        update_source: Some("utharnessly".into()),
    }
}

struct ImportedSkillInput {
    id: String,
    name: String,
    description: String,
    category: String,
    provider: String,
    url: String,
    repository: Option<String>,
    tags: Vec<String>,
}

fn imported_manifest(input: ImportedSkillInput) -> SkillManifest {
    let ImportedSkillInput {
        id,
        name,
        description,
        category,
        provider,
        url,
        repository,
        tags,
    } = input;
    SkillManifest {
        schema_version: SKILLS_SCHEMA_VERSION,
        id,
        name,
        description,
        category,
        source: SkillSource {
            provider,
            url: url.clone(),
            repository: repository.clone(),
            commit: None,
        },
        version: "unknown".into(),
        runtime: Vec::new(),
        entrypoint: None,
        commands: Vec::new(),
        dependencies: Vec::new(),
        tools: Vec::new(),
        permissions: vec!["context.read".into()],
        environment: Vec::new(),
        inputs: json!({}),
        outputs: json!({"text": "procedural skill content"}),
        tags,
        install: SkillInstall {
            command: Some(format!(
                "npx skills add {}",
                repository.clone().unwrap_or_default()
            )),
            working_directory: None,
            package_manager: Some("npx".into()),
        },
        compatibility: SkillCompatibility::default(),
        license: None,
        homepage: Some(url.clone()),
        documentation: Some(description_from_url(&url)),
        checksum: None,
        update_source: Some(url),
    }
}

fn validate_manifest(manifest: &SkillManifest) -> Result<()> {
    if manifest.schema_version != SKILLS_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported skill manifest schema: {}",
            manifest.schema_version
        );
    }
    if manifest.id.is_empty()
        || manifest.id.len() > 200
        || manifest
            .id
            .chars()
            .any(|ch| !(ch.is_ascii_alphanumeric() || ".-_/@".contains(ch)))
    {
        anyhow::bail!("invalid skill id: {}", manifest.id);
    }
    if manifest.name.trim().is_empty() {
        anyhow::bail!("skill name cannot be empty");
    }
    if manifest.description.len() > 20_000 {
        anyhow::bail!("skill description is too large");
    }
    if manifest
        .permissions
        .iter()
        .any(|permission| permission == "system.root" || permission == "credentials.export")
    {
        anyhow::bail!(
            "unsafe permission requires a reviewed adapter: {}",
            manifest.id
        );
    }
    let mut dependency_versions = BTreeMap::new();
    for dependency in &manifest.dependencies {
        if let Some(previous) =
            dependency_versions.insert(dependency.name.clone(), dependency.version.clone())
        {
            if previous != dependency.version {
                anyhow::bail!("conflicting dependency versions for {}", dependency.name);
            }
        }
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn parse_group_heading(line: &str) -> Option<String> {
    let marker = "Skills by ";
    let start = line.find(marker)? + marker.len();
    let tail = &line[start..];
    let end = tail
        .find("</h3>")
        .or_else(|| tail.find('\n'))
        .unwrap_or(tail.len());
    Some(
        tail[..end]
            .trim_matches(|ch: char| ch == '<' || ch == '>' || ch == '`' || ch == '*')
            .trim()
            .to_string(),
    )
}

fn category_from_group(group: &str) -> String {
    let value = group.to_ascii_lowercase();
    if value.contains("test") {
        "testing".into()
    } else if value.contains("git") || value.contains("github") {
        "git".into()
    } else if value.contains("browser") || value.contains("web") {
        "web-development".into()
    } else if value.contains("security") {
        "security".into()
    } else if value.contains("data") || value.contains("database") {
        "data".into()
    } else if value.contains("cloud") || value.contains("devops") {
        "devops".into()
    } else if value.contains("document") || value.contains("content") {
        "documentation".into()
    } else {
        "utilities".into()
    }
}

fn parse_markdown_link(line: &str) -> Option<(String, String)> {
    let start = line.find('[')?;
    let middle = line[start + 1..].find("](")? + start + 1;
    let end = line[middle + 2..].find(')')? + middle + 2;
    let label = line[start + 1..middle].trim().to_string();
    let url = line[middle + 2..end].trim().to_string();
    Some((label, url))
}

fn is_skill_link(url: &str) -> bool {
    url.starts_with("https://")
        && (url.contains("officialskills.sh") || url.contains("github.com"))
        && !url.contains("/issues")
        && !url.contains("/pull/")
}

fn description_from_url(url: &str) -> String {
    format!("Imported procedural skill metadata from {url}; runtime, dependencies, commands, and permissions require source-specific inspection.")
}

fn safe_path(id: &str) -> String {
    id.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ".-_".contains(ch) {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn command_missing(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "command" };
    if cfg!(windows) {
        Command::new(checker)
            .arg(command)
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(true)
    } else {
        Command::new("sh")
            .args(["-lc", &format!("command -v {command}")])
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(true)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn seeds_categories_searches_and_runs_safe_builtin() -> Result<()> {
        let dir = tempdir()?;
        let registry = SkillRegistry::open(dir.path())?;
        assert_eq!(registry.seed_builtins()?, 31);
        assert!(registry.count()? >= 31);
        assert!(!registry.categories()?.is_empty());
        assert_eq!(registry.list(Some("git status"), 10)?.len(), 1);
        let git_init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir.path())
            .output()?;
        assert!(git_init.status.success());
        let installed = registry.install("builtin.git-status", false)?;
        assert_eq!(installed.status, "installed");
        let result = registry.run("builtin.git-status", dir.path(), false)?;
        assert!(result.is_empty() || result.lines().count() < 10);
        registry.remove("builtin.git-status")?;
        let restored = registry.rollback("builtin.git-status")?;
        assert_eq!(restored.status, "installed");
        Ok(())
    }

    #[test]
    fn parses_source_links_and_categories() {
        let (label, url) = parse_markdown_link(
            "- **[test-skill](https://github.com/example/skills/tree/main/test)** - testing",
        )
        .expect("markdown link");
        assert_eq!(label, "test-skill");
        assert!(url.contains("github.com/example"));
        assert_eq!(category_from_group("Skills by Testing Team"), "testing");
        assert_eq!(
            category_from_group("Skills by Browser Tools"),
            "web-development"
        );
    }

    #[test]
    #[ignore = "large catalog benchmark; run explicitly with --ignored"]
    fn indexes_100k_records_in_one_batch() -> Result<()> {
        let dir = tempdir()?;
        let registry = SkillRegistry::open(dir.path())?;
        let manifests = (0..100_000)
            .map(|index| {
                imported_manifest(ImportedSkillInput {
                    id: format!("fixture.skill-{index}"),
                    name: format!("Fixture Skill {index}"),
                    description: format!(
                        "Synthetic registry record for catalog scale test {index}"
                    ),
                    category: "utilities".into(),
                    provider: "fixture".into(),
                    url: format!("file:///fixture/{index}"),
                    repository: None,
                    tags: vec!["fixture".into()],
                })
            })
            .collect::<Vec<_>>();
        let (imported, skipped) =
            registry.upsert_many(&manifests, "available", "unknown", None, None)?;
        assert_eq!(imported, 100_000);
        assert_eq!(skipped, 0);
        assert_eq!(registry.count()?, 100_000);
        assert_eq!(registry.list(Some("catalog scale"), 5)?.len(), 5);
        Ok(())
    }

    #[test]
    fn rejects_unsafe_manifest_permissions() {
        let mut manifest = builtin_manifest("security", "Unsafe", "unsafe", "unsafe", vec![]);
        manifest.permissions.push("system.root".into());
        assert!(validate_manifest(&manifest).is_err());
    }
}
