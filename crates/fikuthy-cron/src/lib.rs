use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// A scheduled cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: CronExpression,
    pub prompt: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: u32,
    pub delivery_target: Option<String>,
}

/// Simple cron expression (5-field: min hour dom month dow).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExpression {
    pub minute: String,
    pub hour: String,
    pub day_of_month: String,
    pub month: String,
    pub day_of_week: String,
}

impl CronExpression {
    pub fn parse(expr: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            anyhow::bail!("cron expression must have 5 fields, got {}", parts.len());
        }
        Ok(Self {
            minute: parts[0].into(),
            hour: parts[1].into(),
            day_of_month: parts[2].into(),
            month: parts[3].into(),
            day_of_week: parts[4].into(),
        })
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.minute, self.hour, self.day_of_month, self.month, self.day_of_week
        )
    }
}

/// Natural-language to cron expression conversion.
pub fn parse_natural_schedule(input: &str) -> anyhow::Result<CronExpression> {
    let lower = input.trim().to_lowercase();

    // Common patterns.
    if lower.contains("every minute") || lower == "* * * * *" {
        return CronExpression::parse("* * * * *");
    }
    if lower.contains("every hour") || lower.contains("hourly") {
        return CronExpression::parse("0 * * * *");
    }
    if lower.contains("daily") || lower.contains("every day") {
        return CronExpression::parse("0 9 * * *"); // 9am default
    }
    if lower.contains("every morning") {
        return CronExpression::parse("0 9 * * *");
    }
    if lower.contains("every night") || lower.contains("nightly") {
        return CronExpression::parse("0 22 * * *");
    }
    if lower.contains("weekly") || lower.contains("every week") {
        return CronExpression::parse("0 9 * * 1"); // Monday 9am
    }
    if lower.contains("monthly") || lower.contains("every month") {
        return CronExpression::parse("0 9 1 * *"); // 1st of month
    }

    // "every N minutes/hours/days" pattern.
    if let Some(caps) = parse_interval(&lower) {
        return Ok(caps);
    }

    // Try raw cron expression.
    CronExpression::parse(input)
}

fn parse_interval(input: &str) -> Option<CronExpression> {
    let words: Vec<&str> = input.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if *word == "every" && i + 2 < words.len() {
            if let Ok(n) = words[i + 1].parse::<u32>() {
                let unit = words[i + 2];
                match unit {
                    "minute" | "minutes" | "min" | "mins" => {
                        if n == 0 || n > 59 {
                            return None;
                        }
                        return CronExpression::parse(&format!("*/{} * * * *", n)).ok();
                    }
                    "hour" | "hours" | "hr" | "hrs" => {
                        if n == 0 || n > 23 {
                            return None;
                        }
                        return CronExpression::parse(&format!("0 */{} * * *", n)).ok();
                    }
                    "day" | "days" => {
                        if n == 0 || n > 31 {
                            return None;
                        }
                        return CronExpression::parse(&format!("0 0 */{} * *", n)).ok();
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Cron job store — persists jobs to a JSON file.
pub struct CronStore {
    path: PathBuf,
}

impl CronStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load_all(&self) -> anyhow::Result<Vec<CronJob>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&data).unwrap_or_default())
    }

    pub fn save_all(&self, jobs: &[CronJob]) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(jobs)?;
        std::fs::create_dir_all(self.path.parent().unwrap_or(&self.path))?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn add(&self, job: CronJob) -> anyhow::Result<()> {
        let mut jobs = self.load_all()?;
        jobs.push(job);
        self.save_all(&jobs)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        let mut jobs = self.load_all()?;
        jobs.retain(|j| j.id != id);
        self.save_all(&jobs)
    }

    pub fn get(&self, id: &str) -> anyhow::Result<CronJob> {
        let jobs = self.load_all()?;
        jobs.into_iter()
            .find(|j| j.id == id)
            .ok_or_else(|| anyhow::anyhow!("cron job {} not found", id))
    }

    pub fn update(&self, job: &CronJob) -> anyhow::Result<()> {
        let mut jobs = self.load_all()?;
        if let Some(existing) = jobs.iter_mut().find(|j| j.id == job.id) {
            *existing = job.clone();
        }
        self.save_all(&jobs)
    }
}

/// Create a new cron job from a natural-language prompt.
pub fn create_job(name: &str, schedule_input: &str, prompt: &str) -> anyhow::Result<CronJob> {
    let schedule = parse_natural_schedule(schedule_input)?;
    Ok(CronJob {
        id: Uuid::new_v4().to_string()[..8].to_string(),
        name: name.into(),
        schedule,
        prompt: prompt.into(),
        enabled: true,
        created_at: Utc::now(),
        last_run: None,
        next_run: None,
        run_count: 0,
        delivery_target: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_cron_expression() {
        let expr = CronExpression::parse("*/5 * * * *").unwrap();
        assert_eq!(expr.minute, "*/5");
        assert_eq!(expr.hour, "*");
    }

    #[test]
    fn parse_natural_language() {
        let expr = parse_natural_schedule("every 30 minutes").unwrap();
        assert_eq!(expr.minute, "*/30");

        let expr = parse_natural_schedule("daily").unwrap();
        assert_eq!(expr.hour, "9");

        let expr = parse_natural_schedule("every hour").unwrap();
        assert_eq!(expr.minute, "0");
        assert_eq!(expr.hour, "*");

        let expr = parse_natural_schedule("weekly").unwrap();
        assert_eq!(expr.day_of_week, "1");
    }

    #[test]
    fn cron_store_round_trip() {
        let dir = tempdir().unwrap();
        let store = CronStore::new(dir.path().join("cron.json"));

        let job = create_job("test", "every hour", "check status").unwrap();
        let id = job.id.clone();
        store.add(job).unwrap();

        let loaded = store.get(&id).unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.prompt, "check status");
        assert!(loaded.enabled);

        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);

        store.remove(&id).unwrap();
        assert!(store.load_all().unwrap().is_empty());
    }
}
