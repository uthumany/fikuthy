use regex::{Captures, Regex};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use utharness_core::{PermissionDecision, PermissionMode, ToolRequest};

#[derive(Clone, Debug)]
pub struct Policy {
    pub mode: PermissionMode,
    pub workspace: PathBuf,
    pub allow_network: bool,
    pub allow_shell: bool,
}

impl Policy {
    pub fn safe(workspace: impl Into<PathBuf>) -> Self {
        Self {
            mode: PermissionMode::Safe,
            workspace: workspace.into(),
            allow_network: false,
            allow_shell: false,
        }
    }

    pub fn evaluate(&self, request: &ToolRequest) -> PermissionDecision {
        let readonly = matches!(
            request.tool.as_str(),
            "read_file"
                | "list_directory"
                | "search_files"
                | "glob"
                | "file_info"
                | "git_status"
                | "git_diff"
                | "git_log"
                | "code_search"
        );
        match self.mode {
            PermissionMode::Safe => {
                if readonly {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny
                }
            }
            PermissionMode::Ask => {
                if readonly {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Prompt
                }
            }
            PermissionMode::Trusted => PermissionDecision::Allow,
            PermissionMode::Custom => {
                if readonly {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Prompt
                }
            }
        }
    }

    pub fn validate_path(&self, requested: &Path) -> Result<PathBuf, String> {
        let candidate = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            self.workspace.join(requested)
        };
        let normalized = candidate
            .canonicalize()
            .map_err(|e| format!("cannot resolve path: {e}"))?;
        let root = self
            .workspace
            .canonicalize()
            .map_err(|e| format!("cannot resolve workspace: {e}"))?;
        if normalized == root || normalized.starts_with(&root) {
            Ok(normalized)
        } else {
            Err(format!("path escapes workspace: {}", requested.display()))
        }
    }

    pub fn redact(text: &str) -> String {
        static ASSIGNMENT: OnceLock<Regex> = OnceLock::new();
        static BEARER: OnceLock<Regex> = OnceLock::new();
        static TOKEN: OnceLock<Regex> = OnceLock::new();

        let assignment = ASSIGNMENT.get_or_init(|| {
            Regex::new(r#"(?i)\b([a-z0-9_]*(?:api[_-]?key|access[_-]?token|auth[_-]?token|token|secret|password|passwd|credential)[a-z0-9_-]*\s*[:=]\s*)('[^']*'|\"[^\"]*\"|[^\s,;}&]+)"#).expect("valid assignment redaction regex")
        });
        let bearer = BEARER.get_or_init(|| {
            Regex::new(r"(?i)\b(bearer\s+)[a-z0-9._~+/-]+=*").expect("valid bearer redaction regex")
        });
        let token = TOKEN.get_or_init(|| {
            Regex::new(r"\b(?:sk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9_]{16,}|github_pat_[A-Za-z0-9_]{16,}|xox[baprs]-[A-Za-z0-9-]{16,})\b").expect("valid token redaction regex")
        });

        let output = assignment.replace_all(text, |caps: &Captures<'_>| {
            format!("{}[REDACTED]", &caps[1])
        });
        let output = bearer.replace_all(&output, |caps: &Captures<'_>| {
            format!("{}[REDACTED]", &caps[1])
        });
        token.replace_all(&output, "[REDACTED]").into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn safe_policy_allows_only_read_tools() {
        let policy = Policy::safe(".");
        assert_eq!(
            policy.evaluate(&ToolRequest {
                tool: "read_file".into(),
                target: None,
                arguments: json!({})
            }),
            PermissionDecision::Allow
        );
        assert_eq!(
            policy.evaluate(&ToolRequest {
                tool: "shell".into(),
                target: None,
                arguments: json!({})
            }),
            PermissionDecision::Deny
        );
    }

    #[test]
    fn redacts_common_secret_assignments() {
        assert_eq!(Policy::redact("API_KEY=abc123 ok"), "API_KEY=[REDACTED] ok");
        assert_eq!(
            Policy::redact("password: \"hunter2\""),
            "password: [REDACTED]"
        );
        assert_eq!(
            Policy::redact("Authorization: Bearer abc.def-123"),
            "Authorization: Bearer [REDACTED]"
        );
        assert_eq!(
            Policy::redact("token=abc, next=true"),
            "token=[REDACTED], next=true"
        );
        assert_eq!(
            Policy::redact("github_pat_1234567890abcdefghijkl"),
            "[REDACTED]"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_that_escapes_workspace() {
        use std::os::unix::fs::symlink;
        let workspace = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), workspace.path().join("escape")).unwrap();
        let policy = Policy::safe(workspace.path());
        assert!(policy.validate_path(Path::new("escape")).is_err());
    }
}
