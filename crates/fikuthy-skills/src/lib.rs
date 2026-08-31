use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Skill lifecycle states — mirrors hermes-agent's curator model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillLifecycle {
    /// Just created, not yet validated.
    Draft,
    /// Validated and available for use.
    Active,
    /// Used successfully at least once.
    Proven,
    /// Not used recently; curator may archive.
    Stale,
    /// Removed from active use but kept for rollback.
    Archived,
}

/// A skill created by the agent from a successful autonomous run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub lifecycle: SkillLifecycle,
    pub tools: Vec<String>,
    pub permissions: Vec<String>,
    pub prompt_template: String,
    pub success_criteria: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub source: SkillSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillSource {
    /// Built-in skill shipped with fikuthy.
    Builtin,
    /// Created by the agent from a successful task.
    AgentCreated { session_id: String, task: String },
    /// Imported from Skills Hub or external source.
    Imported { url: String },
    /// User-authored skill.
    User,
}

/// Tracks skill usage for the learning loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUsageRecord {
    pub skill_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub duration_ms: u64,
    pub output_summary: String,
}

/// The curator reviews skills periodically and manages their lifecycle.
pub struct SkillCurator {
    skills_dir: PathBuf,
    stale_threshold_days: i64,
    archive_threshold_days: i64,
}

impl SkillCurator {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills_dir,
            stale_threshold_days: 30,
            archive_threshold_days: 90,
        }
    }

    /// Run a curation pass: transition skills through lifecycle states.
    pub fn curate(&self) -> anyhow::Result<Vec<CurationAction>> {
        let mut actions = Vec::new();
        let skills = self.load_all_skills()?;
        let now = Utc::now();

        for skill in &skills {
            match skill.lifecycle {
                SkillLifecycle::Active | SkillLifecycle::Proven => {
                    if let Some(last_used) = skill.last_used {
                        let days_since = (now - last_used).num_days();
                        if days_since >= self.archive_threshold_days {
                            actions.push(CurationAction::Archive {
                                skill_id: skill.id.clone(),
                                reason: format!("unused for {} days", days_since),
                            });
                        } else if days_since >= self.stale_threshold_days {
                            actions.push(CurationAction::MarkStale {
                                skill_id: skill.id.clone(),
                                reason: format!("unused for {} days", days_since),
                            });
                        }
                    }
                }
                SkillLifecycle::Stale => {
                    if let Some(last_used) = skill.last_used {
                        let days_since = (now - last_used).num_days();
                        if days_since >= self.archive_threshold_days {
                            actions.push(CurationAction::Archive {
                                skill_id: skill.id.clone(),
                                reason: format!("stale for {} days", days_since),
                            });
                        }
                    }
                }
                SkillLifecycle::Draft => {
                    // Auto-activate drafts that have been used successfully.
                    if skill.success_count > 0 {
                        actions.push(CurationAction::Activate {
                            skill_id: skill.id.clone(),
                            reason: "has successful uses".into(),
                        });
                    }
                }
                SkillLifecycle::Archived => {}
            }
        }

        Ok(actions)
    }

    fn load_all_skills(&self) -> anyhow::Result<Vec<SkillManifest>> {
        let mut skills = Vec::new();
        if !self.skills_dir.is_dir() {
            return Ok(skills);
        }
        for entry in std::fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(skill) = serde_json::from_str::<SkillManifest>(&std::fs::read_to_string(&path)?)
                {
                    skills.push(skill);
                }
            }
        }
        Ok(skills)
    }
}

#[derive(Debug)]
pub enum CurationAction {
    Activate {
        skill_id: String,
        reason: String,
    },
    MarkStale {
        skill_id: String,
        reason: String,
    },
    Archive {
        skill_id: String,
        reason: String,
    },
}

/// Auto-creates a skill from a successful autonomous run.
pub fn create_skill_from_task(
    task: &str,
    steps: &[CompletedStep],
    session_id: &str,
) -> SkillManifest {
    let id = format!("agent.{}", Uuid::new_v4().to_string()[..8].to_string());
    let tools: Vec<String> = steps.iter().map(|s| s.tool.clone()).collect();
    let prompt_template = format!(
        "Execute the following task using the same tool sequence:\n{}\n\nSteps:\n{}",
        task,
        steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {} → {}", i + 1, s.tool, s.target))
            .collect::<Vec<_>>()
            .join("\n")
    );

    SkillManifest {
        id,
        name: format!("Auto: {}", task.chars().take(50).collect::<String>()),
        description: format!("Auto-generated skill from task: {}", task),
        category: "agent-created".into(),
        version: "0.1.0".into(),
        lifecycle: SkillLifecycle::Draft,
        tools,
        permissions: vec!["workspace.read".into()],
        prompt_template,
        success_criteria: "All steps complete without error".into(),
        tags: vec!["auto-generated".into(), "agent-created".into()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_used: None,
        use_count: 0,
        success_count: 0,
        failure_count: 0,
        source: SkillSource::AgentCreated {
            session_id: session_id.into(),
            task: task.into(),
        },
    }
}

pub struct CompletedStep {
    pub tool: String,
    pub target: String,
    pub success: bool,
}

/// Skill store — manages skill persistence.
pub struct SkillStore {
    dir: PathBuf,
}

impl SkillStore {
    pub fn new(dir: PathBuf) -> Self {
        std::fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    pub fn save(&self, skill: &SkillManifest) -> anyhow::Result<PathBuf> {
        let path = self.dir.join(format!("{}.json", skill.id));
        let json = serde_json::to_string_pretty(skill)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    pub fn load(&self, id: &str) -> anyhow::Result<SkillManifest> {
        let path = self.dir.join(format!("{}.json", id));
        let json = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn list(&self) -> anyhow::Result<Vec<SkillManifest>> {
        let mut skills = Vec::new();
        if !self.dir.is_dir() {
            return Ok(skills);
        }
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(skill) = serde_json::from_str(&std::fs::read_to_string(&path)?) {
                    skills.push(skill);
                }
            }
        }
        Ok(skills)
    }

    pub fn record_usage(&self, skill_id: &str, success: bool) -> anyhow::Result<()> {
        let mut skill = self.load(skill_id)?;
        skill.use_count += 1;
        if success {
            skill.success_count += 1;
        } else {
            skill.failure_count += 1;
        }
        skill.last_used = Some(Utc::now());
        skill.updated_at = Utc::now();

        // Auto-promote lifecycle on success.
        if skill.lifecycle == SkillLifecycle::Draft && skill.success_count > 0 {
            skill.lifecycle = SkillLifecycle::Active;
        }
        if skill.lifecycle == SkillLifecycle::Active && skill.success_count >= 3 {
            skill.lifecycle = SkillLifecycle::Proven;
        }

        self.save(&skill)?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        let path = self.dir.join(format!("{}.json", id));
        if path.exists() {
            // Move to quarantine instead of deleting.
            let quarantine = self.dir.join("quarantine");
            std::fs::create_dir_all(&quarantine)?;
            std::fs::rename(&path, quarantine.join(format!("{}.json", id)))?;
        }
        Ok(())
    }

    pub fn rollback(&self, id: &str) -> anyhow::Result<()> {
        let quarantine = self.dir.join("quarantine").join(format!("{}.json", id));
        if quarantine.exists() {
            std::fs::rename(&quarantine, self.dir.join(format!("{}.json", id)))?;
            Ok(())
        } else {
            anyhow::bail!("no quarantined skill found for {}", id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn skill_lifecycle_transitions() {
        let dir = tempdir().unwrap();
        let store = SkillStore::new(dir.path().to_path_buf());

        let skill = create_skill_from_task(
            "list files in workspace",
            &[CompletedStep {
                tool: "list_directory".into(),
                target: ".".into(),
                success: true,
            }],
            "test-session",
        );
        assert_eq!(skill.lifecycle, SkillLifecycle::Draft);

        store.save(&skill).unwrap();

        // Record successful usage — should auto-promote to Active.
        store.record_usage(&skill.id, true).unwrap();
        let loaded = store.load(&skill.id).unwrap();
        assert_eq!(loaded.lifecycle, SkillLifecycle::Active);
        assert_eq!(loaded.use_count, 1);
        assert_eq!(loaded.success_count, 1);

        // Record more successes — should promote to Proven after 3.
        store.record_usage(&skill.id, true).unwrap();
        store.record_usage(&skill.id, true).unwrap();
        let loaded = store.load(&skill.id).unwrap();
        assert_eq!(loaded.lifecycle, SkillLifecycle::Proven);
    }

    #[test]
    fn quarantine_and_rollback() {
        let dir = tempdir().unwrap();
        let store = SkillStore::new(dir.path().to_path_buf());

        let skill = create_skill_from_task(
            "test task",
            &[CompletedStep {
                tool: "read_file".into(),
                target: "test.txt".into(),
                success: true,
            }],
            "session-1",
        );
        let id = skill.id.clone();
        store.save(&skill).unwrap();

        store.remove(&id).unwrap();
        assert!(store.load(&id).is_err());

        store.rollback(&id).unwrap();
        assert!(store.load(&id).is_ok());
    }

    #[test]
    fn curator_detects_stale_skills() {
        let dir = tempdir().unwrap();
        let store = SkillStore::new(dir.path().to_path_buf());
        let curator = SkillCurator::new(dir.path().to_path_buf());

        let mut skill = create_skill_from_task(
            "old task",
            &[CompletedStep {
                tool: "read_file".into(),
                target: "x".into(),
                success: true,
            }],
            "session-old",
        );
        skill.lifecycle = SkillLifecycle::Active;
        skill.last_used = Some(Utc::now() - chrono::Duration::days(35));
        store.save(&skill).unwrap();

        let actions = curator.curate().unwrap();
        assert!(!actions.is_empty());
        match &actions[0] {
            CurationAction::MarkStale { skill_id, .. } => assert_eq!(skill_id, &skill.id),
            _ => panic!("expected MarkStale action"),
        }
    }
}
