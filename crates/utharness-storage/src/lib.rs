use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use utharness_core::{
    new_id, now_ms, Checkpoint, Id, MemoryRecord, Message, MessageRole, Session, SessionStatus,
    Task, TaskStatus, Workspace,
};

const MIGRATION: &str = include_str!("../../../migrations/0001_initial.sql");

#[derive(Clone, Debug)]
pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let storage = Self {
            path: path.as_ref().to_path_buf(),
        };
        storage.with_connection(|conn| {
            conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA busy_timeout = 5000; PRAGMA trusted_schema = OFF;")?;
            conn.execute_batch(MIGRATION)?;
            conn.execute("INSERT OR IGNORE INTO app_meta(key, value, updated_at) VALUES ('schema_version', '1', ?1)", params![now_ms()])?;
            Ok(())
        })?;
        Ok(storage)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = Connection::open(&self.path)
            .with_context(|| format!("open SQLite database at {}", self.path.display()))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000; PRAGMA trusted_schema = OFF;",
        )?;
        f(&conn)
    }

    pub fn ensure_workspace(&self, path: &Path) -> Result<Workspace> {
        let canonical = std::fs::canonicalize(path)
            .with_context(|| format!("workspace does not exist: {}", path.display()))?;
        let canonical_path = canonical.to_string_lossy().to_string();
        let display_name = canonical
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("workspace")
            .to_string();
        self.with_connection(|conn| {
            let now = now_ms();
            let id: String = conn.query_row("SELECT id FROM workspaces WHERE canonical_path = ?1", params![canonical_path], |row| row.get(0)).optional()?.unwrap_or_else(|| new_id().to_string());
            conn.execute("INSERT INTO workspaces(id, canonical_path, display_name, created_at, updated_at, last_opened_at) VALUES (?1, ?2, ?3, ?4, ?4, ?4) ON CONFLICT(canonical_path) DO UPDATE SET updated_at=excluded.updated_at, last_opened_at=excluded.last_opened_at", params![id, canonical_path, display_name, now])?;
            Ok(Workspace { id: Id::parse_str(&id)?, canonical_path, display_name, created_at: now, updated_at: now, last_opened_at: Some(now) })
        })
    }

    pub fn create_session(
        &self,
        workspace: &Workspace,
        title: &str,
        cwd: &Path,
    ) -> Result<Session> {
        let session = Session {
            id: new_id(),
            workspace_id: workspace.id,
            title: title.to_string(),
            status: SessionStatus::Active,
            provider_id: None,
            model_id: None,
            cwd: cwd.to_string_lossy().to_string(),
            theme: "utharness-carbon".into(),
            draft_input: String::new(),
            scroll_offset: 0,
            created_at: now_ms(),
            updated_at: now_ms(),
            closed_at: None,
            version: 0,
        };
        self.with_connection(|conn| {
            conn.execute("INSERT INTO sessions(id, workspace_id, title, status, cwd, theme, draft_input, scroll_offset, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)", params![session.id.to_string(), session.workspace_id.to_string(), session.title, session.status.to_string(), session.cwd, session.theme, session.draft_input, session.scroll_offset, session.created_at, session.updated_at, session.version])?;
            Ok(session)
        })
    }

    pub fn list_sessions(&self, workspace_id: Id) -> Result<Vec<Session>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id, workspace_id, title, status, provider_id, model_id, cwd, theme, draft_input, scroll_offset, created_at, updated_at, closed_at, version FROM sessions WHERE workspace_id = ?1 ORDER BY updated_at DESC")?;
            let rows = stmt.query_map(params![workspace_id.to_string()], |row| {
                Ok(Session { id: Id::parse_str(&row.get::<_, String>(0)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?, workspace_id: Id::parse_str(&row.get::<_, String>(1)?).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?, title: row.get(2)?, status: parse_session_status(&row.get::<_, String>(3)?), provider_id: row.get(4)?, model_id: row.get(5)?, cwd: row.get(6)?, theme: row.get(7)?, draft_input: row.get(8)?, scroll_offset: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?, closed_at: row.get(12)?, version: row.get(13)? })
            })?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn append_message(
        &self,
        session_id: Id,
        role: MessageRole,
        content: &str,
    ) -> Result<Message> {
        self.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let sequence: i64 = tx.query_row("SELECT COALESCE(MAX(sequence), 0) + 1 FROM messages WHERE session_id = ?1", params![session_id.to_string()], |row| row.get(0))?;
            let message = Message { id: new_id(), session_id, parent_id: None, role, content: content.to_string(), status: "complete".into(), created_at: now_ms(), completed_at: Some(now_ms()), sequence };
            tx.execute("INSERT INTO messages(id, session_id, role, content, status, created_at, completed_at, sequence) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![message.id.to_string(), message.session_id.to_string(), message.role.to_string(), message.content, message.status, message.created_at, message.completed_at, message.sequence])?;
            tx.execute("UPDATE sessions SET updated_at = ?1, version = version + 1 WHERE id = ?2", params![now_ms(), session_id.to_string()])?;
            tx.commit()?;
            Ok(message)
        })
    }

    pub fn messages(&self, session_id: Id) -> Result<Vec<Message>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id, session_id, parent_id, role, content, status, created_at, completed_at, sequence FROM messages WHERE session_id = ?1 ORDER BY sequence")?;
            let rows = stmt.query_map(params![session_id.to_string()], |row| {
                Ok(Message { id: parse_id(row.get::<_, String>(0)?)?, session_id: parse_id(row.get::<_, String>(1)?)?, parent_id: row.get::<_, Option<String>>(2)?.map(parse_id).transpose()?, role: parse_role(&row.get::<_, String>(3)?), content: row.get(4)?, status: row.get(5)?, created_at: row.get(6)?, completed_at: row.get(7)?, sequence: row.get(8)? })
            })?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn create_task(
        &self,
        session_id: Option<Id>,
        title: &str,
        description: &str,
    ) -> Result<Task> {
        let task = Task {
            id: new_id(),
            session_id,
            title: title.into(),
            description: description.into(),
            status: TaskStatus::Queued,
            retry_count: 0,
            created_at: now_ms(),
            started_at: None,
            completed_at: None,
            version: 0,
        };
        self.with_connection(|conn| {
            conn.execute("INSERT INTO tasks(id, session_id, title, description, status, retry_count, created_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![task.id.to_string(), task.session_id.map(|v| v.to_string()), task.title, task.description, task.status.to_string(), task.retry_count, task.created_at, task.version])?;
            Ok(task)
        })
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id, session_id, title, description, status, retry_count, created_at, started_at, completed_at, version FROM tasks ORDER BY created_at DESC")?;
            let rows = stmt.query_map([], |row| Ok(Task { id: parse_id(row.get::<_, String>(0)?)?, session_id: row.get::<_, Option<String>>(1)?.map(parse_id).transpose()?, title: row.get(2)?, description: row.get(3)?, status: parse_task_status(&row.get::<_, String>(4)?), retry_count: row.get(5)?, created_at: row.get(6)?, started_at: row.get(7)?, completed_at: row.get(8)?, version: row.get(9)? }))?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn add_memory(
        &self,
        workspace_id: Option<Id>,
        session_id: Option<Id>,
        scope: &str,
        kind: &str,
        content: &str,
        source: &str,
    ) -> Result<MemoryRecord> {
        let memory = MemoryRecord {
            id: new_id(),
            workspace_id,
            session_id,
            scope: scope.into(),
            kind: kind.into(),
            content: content.into(),
            source: source.into(),
            importance: 0.5,
            metadata: json!({}),
            created_at: now_ms(),
            updated_at: now_ms(),
        };
        self.with_connection(|conn| {
            conn.execute("INSERT INTO memories(id, workspace_id, session_id, scope, kind, content, source, importance, metadata_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)", params![memory.id.to_string(), memory.workspace_id.map(|v| v.to_string()), memory.session_id.map(|v| v.to_string()), memory.scope, memory.kind, memory.content, memory.source, memory.importance, memory.metadata.to_string(), memory.created_at, memory.updated_at])?;
            Ok(memory)
        })
    }

    pub fn search_memory(
        &self,
        workspace_id: Option<Id>,
        query: &str,
    ) -> Result<Vec<MemoryRecord>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT m.id, m.workspace_id, m.session_id, m.scope, m.kind, m.content, m.source, m.importance, m.metadata_json, m.created_at, m.updated_at FROM memory_fts f JOIN memories m ON m.rowid = f.rowid WHERE f.memory_fts MATCH ?1 AND (?2 IS NULL OR m.workspace_id = ?2) AND m.deleted_at IS NULL ORDER BY bm25(memory_fts) LIMIT 20")?;
            let rows = stmt.query_map(params![query, workspace_id.map(|v| v.to_string())], |row| Ok(MemoryRecord { id: parse_id(row.get::<_, String>(0)?)?, workspace_id: row.get::<_, Option<String>>(1)?.map(parse_id).transpose()?, session_id: row.get::<_, Option<String>>(2)?.map(parse_id).transpose()?, scope: row.get(3)?, kind: row.get(4)?, content: row.get(5)?, source: row.get(6)?, importance: row.get(7)?, metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or(Value::Object(Default::default())), created_at: row.get(9)?, updated_at: row.get(10)? }))?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn create_checkpoint(
        &self,
        session_id: Id,
        task_id: Option<Id>,
        label: &str,
        state: &Value,
    ) -> Result<Checkpoint> {
        let checkpoint = Checkpoint {
            id: new_id(),
            session_id,
            task_id,
            label: label.into(),
            git_revision: None,
            state: state.clone(),
            created_at: now_ms(),
        };
        self.with_connection(|conn| { conn.execute("INSERT INTO checkpoints(id, session_id, task_id, label, state_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![checkpoint.id.to_string(), checkpoint.session_id.to_string(), checkpoint.task_id.map(|v| v.to_string()), checkpoint.label, checkpoint.state.to_string(), checkpoint.created_at])?; Ok(checkpoint) })
    }

    pub fn record_event(
        &self,
        aggregate_type: &str,
        aggregate_id: Id,
        event_type: &str,
        payload: &Value,
        trace_id: Id,
    ) -> Result<Id> {
        self.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;
            let sequence: i64 = tx.query_row("SELECT COALESCE(MAX(sequence), 0) + 1 FROM runtime_events WHERE aggregate_type = ?1 AND aggregate_id = ?2", params![aggregate_type, aggregate_id.to_string()], |row| row.get(0))?;
            let id = new_id();
            tx.execute("INSERT INTO runtime_events(id, aggregate_type, aggregate_id, sequence, event_type, payload_json, trace_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![id.to_string(), aggregate_type, aggregate_id.to_string(), sequence, event_type, payload.to_string(), trace_id.to_string(), now_ms()])?;
            tx.commit()?;
            Ok(id)
        })
    }

    pub fn recent_events(&self, limit: usize) -> Result<Vec<String>> {
        let limit = limit.min(100) as i64;
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT event_type, payload_json, created_at FROM runtime_events ORDER BY created_at DESC LIMIT ?1")?;
            let rows = stmt.query_map(params![limit], |row| {
                let event_type: String = row.get(0)?;
                let payload: String = row.get(1)?;
                let created_at: i64 = row.get(2)?;
                Ok(format!("{created_at}  {event_type}  {payload}"))
            })?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn recent_memories(
        &self,
        workspace_id: Option<Id>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>> {
        let limit = limit.min(100) as i64;
        self.with_connection(|conn| {
            let mut stmt = conn.prepare("SELECT id, workspace_id, session_id, scope, kind, content, source, importance, metadata_json, created_at, updated_at FROM memories WHERE deleted_at IS NULL AND (?1 IS NULL OR workspace_id = ?1) ORDER BY updated_at DESC LIMIT ?2")?;
            let rows = stmt.query_map(params![workspace_id.map(|v| v.to_string()), limit], |row| {
                Ok(MemoryRecord {
                    id: parse_id(row.get::<_, String>(0)?)?,
                    workspace_id: row.get::<_, Option<String>>(1)?.map(parse_id).transpose()?,
                    session_id: row.get::<_, Option<String>>(2)?.map(parse_id).transpose()?,
                    scope: row.get(3)?,
                    kind: row.get(4)?,
                    content: row.get(5)?,
                    source: row.get(6)?,
                    importance: row.get(7)?,
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or(Value::Object(Default::default())),
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        })
    }

    pub fn integrity_check(&self) -> Result<String> {
        self.with_connection(|conn| {
            Ok(conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?)
        })
    }
}

fn parse_id(value: String) -> rusqlite::Result<Id> {
    Id::parse_str(&value).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
}
fn parse_role(value: &str) -> MessageRole {
    match value {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        _ => MessageRole::Tool,
    }
}
fn parse_session_status(value: &str) -> SessionStatus {
    match value {
        "paused" => SessionStatus::Paused,
        "complete" => SessionStatus::Complete,
        "failed" => SessionStatus::Failed,
        "archived" => SessionStatus::Archived,
        _ => SessionStatus::Active,
    }
}
fn parse_task_status(value: &str) -> TaskStatus {
    match value {
        "planning" => TaskStatus::Planning,
        "running" => TaskStatus::Running,
        "waiting" => TaskStatus::Waiting,
        "paused" => TaskStatus::Paused,
        "complete" => TaskStatus::Complete,
        "failed" => TaskStatus::Failed,
        "cancelled" => TaskStatus::Cancelled,
        _ => TaskStatus::Queued,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn persists_sessions_messages_and_memory_search() -> Result<()> {
        let dir = tempdir()?;
        let db = Storage::open(dir.path().join("state.db"))?;
        let workspace = db.ensure_workspace(dir.path())?;
        let session = db.create_session(&workspace, "Test session", dir.path())?;
        db.append_message(session.id, MessageRole::User, "remember the auth policy")?;
        db.add_memory(
            Some(workspace.id),
            Some(session.id),
            "project",
            "note",
            "Authentication uses ASK mode",
            "test",
        )?;
        assert_eq!(db.messages(session.id)?.len(), 1);
        assert_eq!(
            db.search_memory(Some(workspace.id), "authentication")?
                .len(),
            1
        );
        assert_eq!(db.integrity_check()?, "ok");
        Ok(())
    }
}
