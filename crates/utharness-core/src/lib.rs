use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub type Id = Uuid;

pub fn new_id() -> Id {
    Uuid::now_v7()
}

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("validation failed: {0}")]
    Validation(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Paused,
    Complete,
    Failed,
    Archived,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Archived => "archived",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Planning,
    Running,
    Waiting,
    Paused,
    Complete,
    Failed,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Queued => "queued",
            Self::Planning => "planning",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        })
    }
}

impl TaskStatus {
    pub fn transition(&self, next: TaskStatus) -> Result<TaskStatus, CoreError> {
        let allowed = matches!(
            (self, &next),
            (Self::Queued, Self::Planning)
                | (Self::Queued, Self::Cancelled)
                | (Self::Planning, Self::Running)
                | (Self::Planning, Self::Failed)
                | (Self::Planning, Self::Cancelled)
                | (Self::Running, Self::Waiting)
                | (Self::Running, Self::Paused)
                | (Self::Running, Self::Complete)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::Cancelled)
                | (Self::Waiting, Self::Running)
                | (Self::Waiting, Self::Paused)
                | (Self::Waiting, Self::Cancelled)
                | (Self::Paused, Self::Running)
                | (Self::Paused, Self::Cancelled)
                | (Self::Failed, Self::Queued)
        );
        if allowed {
            Ok(next)
        } else {
            Err(CoreError::InvalidTransition {
                from: self.to_string(),
                to: next.to_string(),
            })
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Id,
    pub canonical_path: String,
    pub display_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_opened_at: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: Id,
    pub workspace_id: Id,
    pub title: String,
    pub status: SessionStatus,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub cwd: String,
    pub theme: String,
    pub draft_input: String,
    pub scroll_offset: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
    pub version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub session_id: Id,
    pub parent_id: Option<Id>,
    pub role: MessageRole,
    pub content: String,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub sequence: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub session_id: Option<Id>,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub retry_count: i64,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: Id,
    pub workspace_id: Option<Id>,
    pub session_id: Option<Id>,
    pub scope: String,
    pub kind: String,
    pub content: String,
    pub source: String,
    pub importance: f64,
    pub metadata: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Id,
    pub session_id: Id,
    pub task_id: Option<Id>,
    pub label: String,
    pub git_revision: Option<String>,
    pub state: Value,
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub id: Id,
    pub aggregate_type: String,
    pub aggregate_id: Id,
    pub sequence: i64,
    pub event_type: String,
    pub payload: Value,
    pub trace_id: Id,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool: String,
    pub target: Option<String>,
    pub arguments: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    Safe,
    Ask,
    Trusted,
    Custom,
}

impl fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Safe => "SAFE",
            Self::Ask => "ASK",
            Self::Trusted => "TRUSTED",
            Self::Custom => "CUSTOM",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Prompt,
    Deny,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: Option<String>,
    pub model: String,
    pub local: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub remediation: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_machine_rejects_invalid_transitions() {
        assert!(TaskStatus::Queued.transition(TaskStatus::Complete).is_err());
        assert_eq!(
            TaskStatus::Running
                .transition(TaskStatus::Complete)
                .unwrap(),
            TaskStatus::Complete
        );
    }

    #[test]
    fn ids_are_unique() {
        assert_ne!(new_id(), new_id());
    }
}
