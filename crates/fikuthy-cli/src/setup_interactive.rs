use anyhow::Result;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::setup_system::{self, ComponentState, EnvironmentReport};

// Egyptian hieroglyphic icons
const ICON_SCAN: &str = "𓆗";
const ICON_LOAD: &str = "𓆘";
const ICON_INSTALL: &str = "𓆙";
const ICON_DONE: &str = "𓆚";
const ICON_WARN: &str = "𓃻";
const ICON_OK: &str = "𓆃";
const ICON_KEY: &str = "𓅿";
const ICON_MODEL: &str = "𓆀";
const ICON_PROVIDER: &str = "𓆁";
const ICON_CONFIG: &str = "𓆂";
const ICON_READY: &str = "𓆆";

const PROVIDERS: &[(&str, &str, &str, &str)] = &[
    ("openrouter", "OpenRouter", "OPENROUTER_API_KEY", "https://openrouter.ai/api/v1"),
    ("openai", "OpenAI", "OPENAI_API_KEY", "https://api.openai.com/v1"),
    ("anthropic", "Anthropic", "ANTHROPIC_API_KEY", "https://api.anthropic.com/v1"),
    ("nvidia", "NVIDIA NIM", "NVIDIA_API_KEY", "https://integrate.api.nvidia.com/v1"),
    ("groq", "Groq", "GROQ_API_KEY", "https://api.groq.com/openai/v1"),
    ("together", "Together AI", "TOGETHER_API_KEY", "https://api.together.xyz/v1"),
    ("deepseek", "DeepSeek", "DEEPSEEK_API_KEY", "https://api.deepseek.com/v1"),
    ("fireworks", "Fireworks AI", "FIREWORKS_API_KEY", "https://api.fireworks.ai/inference/v1"),
    ("ollama", "Ollama (local)", "", "http://127.0.0.1:11434/v1"),
    ("custom", "Custom OpenAI-compatible", "FIKUTHY_API_KEY", ""),
];

const MODELS: &[(&str, &[&str])] = &[
    ("openrouter", &["openrouter/auto", "meta-llama/llama-3.3-70b-instruct", "anthropic/claude-sonnet-4"]),
    ("openai", &["gpt-4o", "gpt-4o-mini", "o3-mini"]),
    ("anthropic", &["claude-sonnet-4-20250514", "claude-haiku-4-20250414"]),
    ("nvidia", &["nvidia/nemotron-3-super-120b-a12b", "meta/llama-3.2-90b-vision-instruct", "deepseek-ai/deepseek-v4-pro-0813"]),
    ("groq", &["llama-3.3-70b-versatile", "mixtral-8x7b-32768"]),
    ("together", &["meta-llama/Llama-3.3-70B-Instruct-Turbo", "deepseek-ai/DeepSeek-V3"]),
    ("deepseek", &["deepseek-chat", "deepseek-reasoner"]),
    ("fireworks", &["accounts/fireworks/models/llama-v3p3-70b-instruct"]),
    ("ollama", &["qwen2.5-coder:7b", "llama3.3:70b", "deepseek-coder-v2:16b"]),
    ("custom", &["default"]),
];

/// Run the full interactive setup flow.
pub fn run_interactive_setup() -> Result<()> {
    clear_screen();
    print_header();
    println!();

    // Phase 1: Scan environment with progress bar
    let report = scan_with_progress()?;
    println!();

    // Phase 2: Install missing dependencies
    install_missing_deps(&report)?;
    println!();

    // Phase 3: Select provider
    let provider = select_provider()?;
    println!();

    // Phase 4: Enter API key (if needed)
    let api_key = if provider.2.is_empty() {
        None
    } else {
        Some(enter_api_key(provider.2)?)
    };
    println!();

    // Phase 5: Select model
    let model = select_model(provider.0)?;
    println!();

    // Phase 6: Write configuration
    write_config_with_progress(provider.0, &model, api_key.as_deref())?;
    println!();

    // Done!
    print_completion(provider.1, &model);
    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().ok();
}

fn print_header() {
    println!("  ╔══════════════════════════════════════════════════╗");
    println!("  ║                                                  ║");
    println!("  ║       {ICON_READY}  F I K U T H Y   S E T U P  {ICON_READY}          ║");
    println!("  ║                                                  ║");
    println!("  ║         Agent Terminal · Interactive Setup        ║");
    println!("  ║                                                  ║");
    println!("  ╚══════════════════════════════════════════════════╝");
}

/// Animated progress bar.
fn progress_bar(label: &str, icon: &str, duration_ms: u64) -> Result<()> {
    let width: usize = 30;
    let steps: usize = 100;
    let delay = duration_ms / steps as u64;

    print!("\r  {icon} {label} ");
    io::stdout().flush()?;

    for i in 0..=steps {
        let filled = (i * width) / 100;
        let empty = width - filled;
        let bar: String = "█".repeat(filled) + &"▒".repeat(empty);
        print!("\r  {icon} {label}  [{bar}] {i:3}%");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(delay));
    }
    println!("  {ICON_OK}");
    Ok(())
}

/// Scan environment with animated progress.
fn scan_with_progress() -> Result<EnvironmentReport> {
    println!("  {ICON_SCAN} scanning environment...");
    println!();

    // Animated scan phases
    let phases = [
        ("scanning system tools", 800),
        ("checking dependencies", 600),
        ("detecting package manager", 400),
        ("probing versions", 700),
        ("analyzing capabilities", 500),
    ];

    for (label, ms) in &phases {
        progress_bar(label, ICON_SCAN, *ms)?;
    }

    let report = setup_system::scan_environment();

    println!();
    println!("  {ICON_LOAD} loading data...");
    thread::sleep(Duration::from_millis(500));

    // Display scan results
    println!();
    println!("  ┌─ Scan Results ─────────────────────────────────┐");
    println!("  │  OS:       {:<37} │", report.os);
    println!("  │  Arch:     {:<37} │", report.architecture);
    println!("  │  Shell:    {:<37} │", report.shell);
    println!("  │  Terminal: {:<37} │", report.terminal);
    if let Some(pm) = &report.package_manager {
        println!("  │  Pkg Mgr:  {:<37} │", pm);
    }
    println!("  ├─────────────────────────────────────────────────┤");

    let mut available = 0;
    let mut missing = 0;
    for comp in &report.components {
        let (icon, status) = match comp.state {
            ComponentState::Available => {
                available += 1;
                (ICON_OK, "OK")
            }
            ComponentState::Missing if comp.required => {
                missing += 1;
                (ICON_WARN, "MISSING")
            }
            ComponentState::Missing => {
                (ICON_WARN, "optional")
            }
            ComponentState::Broken => {
                missing += 1;
                (ICON_WARN, "BROKEN")
            }
            ComponentState::Optional => (ICON_OK, "optional"),
        };
        let version_str = comp.version.as_deref().unwrap_or("—");
        println!("  │  {icon} {:<14} {:<10} {:<15} │", comp.label, status, version_str);
    }
    println!("  ├─────────────────────────────────────────────────┤");
    println!("  │  {ICON_OK} Available: {available}  {ICON_WARN} Missing: {missing}              │");
    println!("  └─────────────────────────────────────────────────┘");

    Ok(report)
}

/// Install missing required dependencies.
fn install_missing_deps(report: &EnvironmentReport) -> Result<()> {
    let missing: Vec<_> = report
        .components
        .iter()
        .filter(|c| c.required && c.state != ComponentState::Available)
        .collect();

    if missing.is_empty() {
        println!("  {ICON_OK} All required dependencies are installed!");
        return Ok(());
    }

    println!("  {ICON_INSTALL} installing missing dependencies...");
    println!();

    for comp in &missing {
        if let Some(hint) = &comp.install_hint {
            print!("  {ICON_INSTALL} Installing {}... ", comp.label);
            io::stdout().flush()?;

            // Parse the install command
            let parts: Vec<&str> = hint.split_whitespace().collect();
            if parts.is_empty() {
                println!("{ICON_WARN} no install command");
                continue;
            }

            // Run the install command
            let result = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            match result {
                Ok(status) if status.success() => {
                    // Animated install progress
                    progress_bar(&format!("{} installed", comp.label), ICON_INSTALL, 600)?;
                }
                _ => {
                    println!("{ICON_WARN} failed — run manually: {hint}");
                }
            }
        } else {
            println!("  {ICON_WARN} {} — no auto-install available", comp.label);
        }
    }

    Ok(())
}

/// Interactive provider selection.
fn select_provider() -> Result<&'static (&'static str, &'static str, &'static str, &'static str)> {
    println!("  {ICON_PROVIDER} Select LLM Provider:");
    println!();

    for (i, (id, name, _, _)) in PROVIDERS.iter().enumerate() {
        let icon = if *id == "ollama" { ICON_OK } else { ICON_KEY };
        println!("    {:2}. {icon}  {}", i + 1, name);
    }
    println!();

    print!("  Enter choice (1-{}): ", PROVIDERS.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input
        .trim()
        .parse()
        .unwrap_or(1)
        .min(PROVIDERS.len())
        .max(1);

    let selected = &PROVIDERS[choice - 1];
    println!("  {ICON_OK} Selected: {}", selected.1);
    Ok(selected)
}

/// Enter API key with masked input.
fn enter_api_key(var_name: &str) -> Result<String> {
    println!("  {ICON_KEY} Enter API Key ({var_name}):");
    println!("  {ICON_KEY} (input is masked — paste your key and press Enter)");
    println!();

    print!("  {ICON_KEY} API Key: ");
    io::stdout().flush()?;

    // Read with masking
    let key = read_masked_input()?;

    if key.trim().is_empty() {
        println!("  {ICON_WARN} No key entered — you can set {var_name} later");
        return Ok(String::new());
    }

    // Validate key format (basic check)
    if key.len() < 10 {
        println!("  {ICON_WARN} Key seems too short — please verify");
    } else {
        println!("  {ICON_OK} Key received ({})", mask_key(&key));
    }

    // Save to secrets file
    let home = setup_system::home()?;
    let secrets_path = home.join("secrets.env");
    setup_system::persist_secret_at(&secrets_path, var_name, &key)?;

    Ok(key)
}

/// Read input with masking (shows asterisks).
fn read_masked_input() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    // Trim the newline
    Ok(input.trim().to_string())
}

/// Mask API key for display: show first 6 and last 4 chars.
fn mask_key(key: &str) -> String {
    if key.len() <= 12 {
        return "*".repeat(key.len());
    }
    format!("{}...{}", &key[..6], &key[key.len() - 4..])
}

/// Interactive model selection.
fn select_model(provider_id: &str) -> Result<String> {
    println!("  {ICON_MODEL} Select Model:");
    println!();

    let models = MODELS
        .iter()
        .find(|(id, _)| *id == provider_id)
        .map(|(_, m)| *m)
        .unwrap_or(&["default"]);

    for (i, model) in models.iter().enumerate() {
        println!("    {:2}. {ICON_MODEL}  {}", i + 1, model);
    }
    println!("    {}. {ICON_MODEL}  Enter custom model", models.len() + 1);
    println!();

    print!("  Enter choice (1-{}): ", models.len() + 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().unwrap_or(1);

    let model = if choice > 0 && choice <= models.len() {
        models[choice - 1].to_string()
    } else if choice == models.len() + 1 {
        print!("  {ICON_MODEL} Enter model ID: ");
        io::stdout().flush()?;
        let mut custom = String::new();
        io::stdin().read_line(&mut custom)?;
        custom.trim().to_string()
    } else {
        models[0].to_string()
    };

    println!("  {ICON_OK} Selected: {model}");
    Ok(model)
}

/// Write configuration with animated progress.
fn write_config_with_progress(
    provider_id: &str,
    model: &str,
    api_key: Option<&str>,
) -> Result<()> {
    println!("  {ICON_CONFIG} writing configuration...");
    println!();

    progress_bar("creating workspace config", ICON_CONFIG, 800)?;
    progress_bar("saving provider settings", ICON_CONFIG, 600)?;
    progress_bar("configuring model", ICON_CONFIG, 400)?;

    // Write fikuthy.json
    let config = serde_json::json!({
        "schemaVersion": 1,
        "mode": "full",
        "provider": provider_id,
        "model": model,
        "permissionMode": "safe",
        "tools": ["workspace_read", "git_inspection", "skills", "memory"],
        "ui": {
            "banner": true,
            "bannerMode": "full",
            "icons": "unicode"
        }
    });

    let config_path = std::env::current_dir()?.join("fikuthy.json");
    std::fs::write(
        &config_path,
        format!("{}\n", serde_json::to_string_pretty(&config)?),
    )?;

    // Write global config
    setup_system::write_global_config("full", provider_id, model, &config_path)?;

    progress_bar("finalizing setup", ICON_CONFIG, 500)?;

    println!();
    println!("  {ICON_OK} Configuration saved to {}", config_path.display());

    Ok(())
}

/// Print completion message.
fn print_completion(provider_name: &str, model: &str) {
    println!();
    println!("  ╔══════════════════════════════════════════════════╗");
    println!("  ║                                                  ║");
    println!("  ║       {ICON_READY}  F I K U T H Y   R E A D Y  {ICON_READY}          ║");
    println!("  ║                                                  ║");
    println!("  ╚══════════════════════════════════════════════════╝");
    println!();
    println!("  {ICON_OK} Provider: {provider_name}");
    println!("  {ICON_OK} Model:    {model}");
    println!();
    println!("  {ICON_READY} Next steps:");
    println!("    fikuthy chat \"hello\"           Start chatting");
    println!("    fikuthy autonomous \"task\"      Run autonomous agent");
    println!("    fikuthy doctor                  Check diagnostics");
    println!("    fikuthy tui                     Open terminal UI");
    println!();
}
