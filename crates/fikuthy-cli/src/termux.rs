use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Serialize)]
pub struct TermuxPaths {
    pub prefix: PathBuf,
    pub bin: PathBuf,
    pub lib: PathBuf,
    pub share: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub repair: Option<String>,
}

pub fn is_termux() -> bool {
    env::var_os("TERMUX_VERSION").is_some()
        || env::var_os("PREFIX")
            .map(|value| value.to_string_lossy().contains("com.termux"))
            .unwrap_or(false)
        || Path::new("/data/data/com.termux/files/usr").is_dir()
}

pub fn paths() -> TermuxPaths {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let prefix = if is_termux() {
        env::var_os("PREFIX")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/data/data/com.termux/files/usr"))
    } else {
        PathBuf::from("/usr/local")
    };
    TermuxPaths {
        bin: prefix.join("bin"),
        lib: prefix.join("lib").join("fikuthy"),
        share: prefix.join("share").join("fikuthy"),
        prefix,
        config: home.join(".config").join("fikuthy"),
        data: home.join(".local").join("share").join("fikuthy"),
        cache: home.join(".cache").join("fikuthy"),
    }
}

pub fn setup() -> Result<TermuxPaths> {
    let locations = paths();
    for directory in [
        &locations.config,
        &locations.data,
        &locations.cache,
        &locations.data.join("projects"),
        &locations.data.join("memory"),
        &locations.data.join("sessions"),
        &locations.data.join("downloads"),
        &locations.data.join("logs"),
        &locations.data.join("skills"),
        &locations.data.join("mcp"),
    ] {
        fs::create_dir_all(directory)
            .with_context(|| format!("create Termux directory {}", directory.display()))?;
    }
    Ok(locations)
}

pub fn info() -> Result<Value> {
    let locations = paths();
    let (width, height) = terminal_size();
    let api = api_commands_available();
    let value = serde_json::json!({
        "platform": if is_termux() { "termux" } else { "non-termux" },
        "os": if command_exists("getprop") { "android" } else { env::consts::OS },
        "android_version": command_output("getprop", &["ro.build.version.release"]),
        "device_model": command_output("getprop", &["ro.product.model"]),
        "architecture": command_output("uname", &["-m"]).unwrap_or_else(|| env::consts::ARCH.into()),
        "prefix": locations.prefix,
        "shell": env::var("SHELL").unwrap_or_else(|_| "sh".into()),
        "termux_version": env::var("TERMUX_VERSION").ok(),
        "terminal": env::var("TERM").unwrap_or_else(|_| "unknown".into()),
        "terminal_size": { "columns": width, "rows": height },
        "color": color_capability(),
        "storage_link": home_storage_link(),
        "termux_api": !api.is_empty(),
        "termux_api_commands": api,
        "node": command_version("node", &["--version"]),
        "python": command_version("python", &["--version"]),
        "git": command_version("git", &["--version"]),
        "ssh": command_version("ssh", &["-V"]),
        "curl": command_version("curl", &["--version"]),
        "openssl": command_version("openssl", &["version"]),
        "disk": command_output("df", &["-h", "."]),
        "memory": memory_summary(),
        "dns": dns_status(),
        "network": network_status(),
        "paths": locations,
    });
    Ok(value)
}

pub fn checks() -> Vec<Check> {
    let locations = paths();
    let mut result = Vec::new();
    let termux = is_termux();
    result.push(check(
        "Termux",
        termux,
        if termux {
            "Termux environment detected"
        } else {
            "not running inside Termux"
        },
        Some("Install from a Termux shell so $PREFIX points to the Termux prefix"),
    ));
    let android = command_exists("getprop");
    result.push(check(
        "Android",
        android,
        &command_output("getprop", &["ro.build.version.release"])
            .unwrap_or_else(|| "unavailable".into()),
        None,
    ));
    result.push(check(
        "Architecture",
        command_exists("uname"),
        &command_output("uname", &["-m"]).unwrap_or_else(|| env::consts::ARCH.into()),
        None,
    ));
    result.push(check(
        "PREFIX",
        env::var_os("PREFIX").is_some() && locations.prefix.is_dir(),
        &locations.prefix.display().to_string(),
        Some("Start Termux normally so PREFIX is exported"),
    ));
    result.push(check(
        "Shell",
        env::var_os("SHELL").is_some() || command_exists("sh"),
        &env::var("SHELL").unwrap_or_else(|_| "sh".into()),
        None,
    ));
    let (columns, rows) = terminal_size();
    result.push(check(
        "Terminal",
        columns >= 40 && rows >= 12,
        &format!(
            "{columns} columns x {rows} rows; {}",
            env::var("TERM").unwrap_or_else(|_| "unknown".into())
        ),
        Some("Use portrait compact mode or rotate to landscape for a larger layout"),
    ));
    result.push(check(
        "ANSI/truecolor",
        env::var_os("NO_COLOR").is_none(),
        &color_capability(),
        Some("Unset NO_COLOR and use a truecolor-capable Termux terminal for the full palette"),
    ));
    result.push(check(
        "Storage sandbox",
        locations.config.parent().map(Path::is_dir).unwrap_or(false),
        &format!(
            "config={} data={} cache={}",
            locations.config.display(),
            locations.data.display(),
            locations.cache.display()
        ),
        Some("Run `fikuthy termux setup` to create user directories"),
    ));
    result.push(check(
        "Shared storage",
        home_storage_link(),
        if home_storage_link() {
            "~/storage is available"
        } else {
            "~/storage is not linked"
        },
        Some("Run `fikuthy termux storage enable` when shared storage is needed"),
    ));
    result.push(check(
        "Node.js",
        command_exists("node"),
        &command_version("node", &["--version"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install nodejs-lts` if using the source UI"),
    ));
    result.push(check(
        "Python",
        command_exists("python"),
        &command_version("python", &["--version"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install python` for Python-based tools"),
    ));
    result.push(check(
        "Git",
        command_exists("git"),
        &command_version("git", &["--version"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install git`"),
    ));
    result.push(check(
        "SSH",
        command_exists("ssh"),
        &command_version("ssh", &["-V"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install openssh` for remote workflows"),
    ));
    result.push(check(
        "curl",
        command_exists("curl"),
        &command_version("curl", &["--version"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install curl`"),
    ));
    result.push(check(
        "OpenSSL",
        command_exists("openssl"),
        &command_version("openssl", &["version"]).unwrap_or_else(|| "missing".into()),
        Some("Install with `pkg install openssl`"),
    ));
    result.push(check(
        "Termux:API",
        !api_commands_available().is_empty(),
        if api_commands_available().is_empty() {
            "optional extension unavailable"
        } else {
            "optional extension available"
        },
        Some("Install the matching Termux:API app and run `pkg install termux-api`"),
    ));
    result.push(check(
        "Disk",
        disk_has_space(),
        &command_output("df", &["-h", "."]).unwrap_or_else(|| "unavailable".into()),
        Some("Remove unused packages or cached archives with `pkg clean`"),
    ));
    result.push(check(
        "RAM",
        memory_available(),
        &memory_summary(),
        Some("Use compact TUI mode and avoid concurrent builds on low-memory devices"),
    ));
    result.push(check(
        "DNS/network",
        network_status() == "online",
        &format!("dns={} network={}", dns_status(), network_status()),
        Some("Check Wi-Fi/mobile data, DNS, and `curl https://github.com`"),
    ));
    result
}

pub fn install_keys() -> Result<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let directory = home.join(".termux");
    fs::create_dir_all(&directory)?;
    let path = directory.join("termux.properties");
    if path.exists() {
        let backup = directory.join(format!("termux.properties.fikuthy-{}", now_ms()));
        fs::copy(&path, backup)?;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let replacement =
        "extra-keys = [['ESC','CTRL','ALT','TAB','UP','DOWN','LEFT','RIGHT','ENTER']]";
    let mut lines = existing
        .lines()
        .filter(|line| !line.trim_start().starts_with("extra-keys"))
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.push(replacement.into());
    let mut file = fs::File::create(&path)?;
    writeln!(file, "{}", lines.join("\n"))?;
    if command_exists("termux-reload-settings") {
        let _ = Command::new("termux-reload-settings").status();
    }
    Ok(path)
}

pub fn enable_storage() -> Result<()> {
    if !command_exists("termux-setup-storage") {
        anyhow::bail!(
            "termux-setup-storage is unavailable; install/use the official Termux build first"
        )
    }
    let status = Command::new("termux-setup-storage").status()?;
    if !status.success() {
        anyhow::bail!("termux-setup-storage exited with {status}")
    }
    Ok(())
}

pub fn api_status_or_call(capability: Option<&str>, value: Option<&str>) -> Result<String> {
    let Some(capability) = capability else {
        let commands = api_commands_available();
        return Ok(if commands.is_empty() {
            "Termux:API unavailable (optional); core FIKUTHY remains usable.".into()
        } else {
            format!("Termux:API available: {}", commands.join(", "))
        });
    };
    let (command, args) = api_command(capability, value)?;
    if !command_exists(command) {
        anyhow::bail!("Termux:API capability `{capability}` requires `{command}`; install the matching termux-api package")
    }
    let output = Command::new(command).args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        anyhow::bail!("{command} failed: {stderr}")
    }
    Ok(if stdout.is_empty() { stderr } else { stdout })
}

pub fn update_guidance() -> String {
    if is_termux() {
        "Installation source: Termux pkg\nRun:\n  pkg update\n  pkg upgrade fikuthy\n\nFIKUTHY will not overwrite pkg-managed files.".into()
    } else {
        "Not running in Termux. Use the installation method that owns this executable (npm, PyPI, release archive, or source build).".into()
    }
}

fn api_command<'a>(capability: &'a str, value: Option<&'a str>) -> Result<(&'a str, Vec<String>)> {
    let command = match capability {
        "battery" => "termux-battery-status",
        "device" => "termux-telephony-deviceinfo",
        "clipboard" | "clipboard-get" => "termux-clipboard-get",
        "clipboard-set" => "termux-clipboard-set",
        "notification" | "notify" => "termux-notification",
        "tts" => "termux-tts-speak",
        "speech" => "termux-speech-to-text",
        "wifi" => "termux-wifi-connectioninfo",
        "storage" => "termux-storage-get",
        "share" => "termux-share",
        "vibrate" => "termux-vibrate",
        "dialog" => "termux-dialog",
        other => anyhow::bail!("unsupported Termux:API capability `{other}`; use battery, notification, clipboard, tts, speech, wifi, storage, share, vibrate, dialog, or device"),
    };
    let args = match capability {
        "clipboard-set" | "tts" | "storage" | "share" => {
            value.into_iter().map(str::to_string).collect()
        }
        "notify" | "notification" => value
            .map(|text| {
                vec![
                    "--title".into(),
                    "FIKUTHY".into(),
                    "--content".into(),
                    text.into(),
                ]
            })
            .unwrap_or_default(),
        "vibrate" => value
            .map(|text| vec!["-d".into(), text.into()])
            .unwrap_or_default(),
        "dialog" => value
            .map(|text| vec!["-t".into(), "FIKUTHY".into(), "-i".into(), text.into()])
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    Ok((command, args))
}

fn check(name: &str, ok: bool, detail: &str, repair: Option<&str>) -> Check {
    Check {
        name: name.into(),
        status: if ok { "pass".into() } else { "warning".into() },
        detail: detail.into(),
        repair: repair.map(str::to_string),
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .args(["-lc", &format!("command -v {command}")])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn command_version(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .trim()
    .to_string();
    (!text.is_empty()).then_some(text)
}

fn api_commands_available() -> Vec<String> {
    [
        ("battery", "termux-battery-status"),
        ("notification", "termux-notification"),
        ("clipboard", "termux-clipboard-get"),
        ("tts", "termux-tts-speak"),
        ("speech", "termux-speech-to-text"),
        ("wifi", "termux-wifi-connectioninfo"),
        ("storage", "termux-storage-get"),
        ("share", "termux-share"),
        ("vibrate", "termux-vibrate"),
        ("dialog", "termux-dialog"),
        ("device", "termux-telephony-deviceinfo"),
    ]
    .into_iter()
    .filter(|(_, command)| command_exists(command))
    .map(|(name, _)| name.into())
    .collect()
}

fn terminal_size() -> (u16, u16) {
    let output = Command::new("sh")
        .args(["-lc", "stty size 2>/dev/null || printf '0 0'"])
        .output();
    output
        .ok()
        .and_then(|output| {
            let values = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .filter_map(|value| value.parse::<u16>().ok())
                .collect::<Vec<_>>();
            (values.len() == 2).then(|| (values[1], values[0]))
        })
        .unwrap_or((0, 0))
}

fn color_capability() -> String {
    if env::var_os("NO_COLOR").is_some() {
        "monochrome".into()
    } else if env::var("COLORTERM")
        .map(|value| value.contains("truecolor"))
        .unwrap_or(false)
    {
        "truecolor".into()
    } else if env::var("TERM")
        .map(|value| value.contains("256color"))
        .unwrap_or(false)
    {
        "ansi256".into()
    } else {
        "ansi16".into()
    }
}

fn home_storage_link() -> bool {
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("storage").is_dir())
        .unwrap_or(false)
}

fn disk_has_space() -> bool {
    command_output("df", &["-Pk", "."])
        .and_then(|value| {
            value
                .lines()
                .nth(1)
                .and_then(|line| line.split_whitespace().nth(3)?.parse::<u64>().ok())
        })
        .map(|available_kb| available_kb >= 128 * 1024)
        .unwrap_or(false)
}

fn memory_summary() -> String {
    command_output("sh", &["-lc", "awk '/MemTotal|MemAvailable/ {print $1\"=\"$2\" kB\"}' /proc/meminfo | paste -sd ' ' -"])
        .unwrap_or_else(|| "unavailable".into())
}

fn memory_available() -> bool {
    command_output(
        "sh",
        &["-lc", "awk '/MemAvailable/ {print $2}' /proc/meminfo"],
    )
    .and_then(|value| value.parse::<u64>().ok())
    .map(|available_kb| available_kb >= 128 * 1024)
    .unwrap_or(true)
}

fn dns_status() -> String {
    if command_exists("getent")
        && Command::new("getent")
            .args(["hosts", "github.com"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    {
        "ok".into()
    } else if command_exists("nslookup") {
        if Command::new("nslookup")
            .arg("github.com")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            "ok".into()
        } else {
            "unavailable".into()
        }
    } else {
        "unverified".into()
    }
}

fn network_status() -> String {
    if command_exists("curl")
        && Command::new("curl")
            .args(["-fsSI", "--max-time", "3", "https://github.com"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    {
        "online".into()
    } else {
        "offline-or-blocked".into()
    }
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
