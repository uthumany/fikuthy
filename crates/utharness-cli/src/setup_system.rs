use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReport {
    pub os: String,
    pub architecture: String,
    pub shell: String,
    pub terminal: String,
    pub package_manager: Option<String>,
    pub components: Vec<ComponentStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStatus {
    pub id: &'static str,
    pub label: &'static str,
    pub state: ComponentState,
    pub required: bool,
    pub version: Option<String>,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ComponentState {
    Available,
    Missing,
    Broken,
    Optional,
}

const COMPONENTS: &[(&str, &str, &[&str], bool)] = &[
    ("git", "Git", &["--version"], true),
    ("curl", "curl", &["--version"], true),
    ("node", "Node.js", &["--version"], true),
    ("python3", "Python", &["--version"], false),
    ("uv", "uv", &["--version"], false),
    ("pip", "pip", &["--version"], false),
    ("npm", "npm", &["--version"], false),
    ("pnpm", "pnpm", &["--version"], false),
    ("bun", "Bun", &["--version"], false),
    ("deno", "Deno", &["--version"], false),
    ("sqlite3", "SQLite", &["--version"], false),
    ("rg", "ripgrep", &["--version"], false),
    ("ffmpeg", "ffmpeg", &["-version"], false),
    ("docker", "Docker", &["--version"], false),
    ("podman", "Podman", &["--version"], false),
    ("ssh", "SSH", &["-V"], false),
    ("gh", "GitHub CLI", &["--version"], false),
    ("rustc", "Rust", &["--version"], false),
    ("cargo", "Cargo", &["--version"], false),
    ("ollama", "Ollama", &["--version"], false),
];

pub fn scan_environment() -> EnvironmentReport {
    let package_manager = ["apt", "dnf", "pacman", "brew", "winget", "pkg"]
        .into_iter()
        .find(|name| executable_exists(name))
        .map(str::to_string);
    let components = COMPONENTS
        .iter()
        .map(|(id, label, args, required)| {
            scan_component(id, label, args, *required, package_manager.as_deref())
        })
        .collect();
    EnvironmentReport {
        os: env::consts::OS.into(),
        architecture: env::consts::ARCH.into(),
        shell: env::var("SHELL")
            .or_else(|_| env::var("COMSPEC"))
            .unwrap_or_else(|_| "unknown".into()),
        terminal: env::var("TERM_PROGRAM")
            .or_else(|_| env::var("TERM"))
            .unwrap_or_else(|_| "unknown".into()),
        package_manager,
        components,
    }
}

fn scan_component(
    id: &'static str,
    label: &'static str,
    args: &[&str],
    required: bool,
    manager: Option<&str>,
) -> ComponentStatus {
    if !executable_exists(id) {
        return ComponentStatus {
            id,
            label,
            state: if required {
                ComponentState::Missing
            } else {
                ComponentState::Optional
            },
            required,
            version: None,
            install_hint: install_hint(id, manager),
        };
    }
    match probe_version(id, args) {
        Ok(Some(raw)) => {
            let version = raw
                .lines()
                .next()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.chars().take(100).collect());
            ComponentStatus {
                id,
                label,
                state: ComponentState::Available,
                required,
                version,
                install_hint: None,
            }
        }
        _ => ComponentStatus {
            id,
            label,
            state: ComponentState::Broken,
            required,
            version: None,
            install_hint: install_hint(id, manager),
        },
    }
}

fn probe_version(command: &str, args: &[&str]) -> Result<Option<String>> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start {command}"))?;
    let Some(status) = child.wait_timeout(Duration::from_secs(2))? else {
        child.kill().ok();
        child.wait().ok();
        return Ok(None);
    };
    if !status.success() {
        return Ok(None);
    }
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut stream) = child.stdout.take() {
        stream.read_to_string(&mut stdout)?;
    }
    if let Some(mut stream) = child.stderr.take() {
        stream.read_to_string(&mut stderr)?;
    }
    Ok(Some(if stdout.trim().is_empty() {
        stderr
    } else {
        stdout
    }))
}

fn executable_exists(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|directory| {
        let candidate = directory.join(name);
        candidate.is_file() || (cfg!(windows) && directory.join(format!("{name}.exe")).is_file())
    })
}

fn install_hint(component: &str, manager: Option<&str>) -> Option<String> {
    let package = match component {
        "node" => "nodejs",
        "rg" => "ripgrep",
        "python3" => "python3",
        other => other,
    };
    match manager {
        Some("apt") => Some(format!("sudo apt install -y {package}")),
        Some("dnf") => Some(format!("sudo dnf install -y {package}")),
        Some("pacman") => Some(format!("sudo pacman -S --needed {package}")),
        Some("brew") => Some(format!("brew install {package}")),
        Some("winget") => Some(format!("winget search {package}")),
        Some("pkg") => Some(format!("pkg install {package}")),
        _ => None,
    }
}

pub fn home() -> Result<PathBuf> {
    if let Some(value) = env::var_os("UTHARNESS_HOME") {
        return Ok(PathBuf::from(value));
    }
    let base = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .context("HOME or USERPROFILE is required")?;
    Ok(PathBuf::from(base).join(".utharness"))
}

pub fn secrets_path() -> Result<PathBuf> {
    Ok(home()?.join("secrets.env"))
}
pub fn config_path() -> Result<PathBuf> {
    Ok(home()?.join("config.yaml"))
}

pub fn persist_secret(name: &str, value: &str) -> Result<PathBuf> {
    persist_secret_at(&secrets_path()?, name, value)
}

pub fn persist_secret_at(path: &Path, name: &str, value: &str) -> Result<PathBuf> {
    if !valid_secret_name(name) {
        anyhow::bail!("invalid secret variable name");
    }
    if value.trim().is_empty() || value.contains(['\n', '\r', '\0']) {
        anyhow::bail!("API key must be non-empty and contain no line breaks");
    }
    let parent = path.parent().context("secrets path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut entries = read_secret_entries(path)?;
    entries.retain(|(key, _)| key != name);
    entries.push((name.into(), value.into()));
    let temporary = parent.join(".secrets.env.tmp");
    let body = entries
        .into_iter()
        .map(|(key, value)| format!("{key}={}\n", shell_quote(&value)))
        .collect::<String>();
    fs::write(&temporary, body.as_bytes()).context("failed to write temporary secrets file")?;
    set_private_permissions(&temporary)?;
    fs::rename(&temporary, path).context("failed to atomically save secrets")?;
    set_private_permissions(path)?;
    Ok(path.to_path_buf())
}

pub fn load_secrets() -> Result<()> {
    load_secrets_from(&secrets_path()?)
}

pub fn load_secrets_from(path: &Path) -> Result<()> {
    for (name, value) in read_secret_entries(path)? {
        if env::var_os(&name).is_none() {
            env::set_var(name, value);
        }
    }
    Ok(())
}

fn read_secret_entries(path: &Path) -> Result<Vec<(String, String)>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).context("failed to read secrets file")?;
    raw.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let (key, value) = line.split_once('=').context("invalid secrets.env entry")?;
            let key = key.trim();
            if !valid_secret_name(key) {
                anyhow::bail!("invalid variable name in secrets.env");
            }
            Ok((key.into(), shell_unquote(value.trim())))
        })
        .collect()
}

fn valid_secret_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        && name.as_bytes()[0].is_ascii_uppercase()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn shell_unquote(value: &str) -> String {
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        value[1..value.len() - 1].replace("'\\''", "'")
    } else {
        value.into()
    }
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn write_global_config(
    mode: &str,
    provider: &str,
    model: &str,
    workspace_config: &Path,
) -> Result<PathBuf> {
    let path = config_path()?;
    fs::create_dir_all(path.parent().context("config path has no parent")?)?;
    let quote = |value: &str| serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into());
    let body = format!(
        "schema_version: 1\nmode: {}\nprovider: {}\nmodel: {}\nworkspace_config: {}\nsecrets_file: {}\n",
        quote(mode), quote(provider), quote(model), quote(&workspace_config.display().to_string()), quote(&secrets_path()?.display().to_string())
    );
    fs::write(&path, body)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn secret_file_is_private_and_round_trips_quotes() {
        let directory = tempdir().unwrap();
        let secrets = directory.path().join("secrets.env");
        persist_secret_at(&secrets, "TEST_SECRET_KEY_1", "secret'with quote").unwrap();
        // Read back and verify round-trip through file
        let entries = read_secret_entries(&secrets).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "TEST_SECRET_KEY_1");
        assert_eq!(entries[0].1, "secret'with quote");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&secrets)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn rejects_invalid_secret_variable_names() {
        let directory = tempdir().unwrap();
        let secrets = directory.path().join("secrets.env");
        assert!(persist_secret_at(&secrets, "", "secret").is_err());
        assert!(persist_secret_at(&secrets, "1INVALID", "secret").is_err());
        assert!(persist_secret_at(&secrets, "mixed_Case", "secret").is_err());
    }
}
