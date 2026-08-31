use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, process::Command};
use utharness_core::{PermissionDecision, PermissionMode, ToolRequest};
use utharness_security::Policy;

pub fn run_shell(workspace: &Path, command: &str, allow: bool) -> Result<()> {
    let root = fs::canonicalize(workspace)?;
    let policy = if allow {
        Policy {
            mode: PermissionMode::Trusted,
            workspace: root.clone(),
            allow_network: false,
            allow_shell: true,
        }
    } else {
        Policy::safe(root.clone())
    };
    let request = ToolRequest {
        tool: "shell".into(),
        target: Some(command.into()),
        arguments: json!({"cwd": root}),
    };
    if policy.evaluate(&request) != PermissionDecision::Allow {
        println!("Permission required: shell execution is blocked in SAFE mode.");
        println!(
            "Re-run with `--allow` after reviewing the command: {}",
            Policy::redact(command)
        );
        anyhow::bail!("tool denied by permission policy")
    }
    deny_destructive_command(command)?;
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .output()?;
    print!(
        "{}",
        Policy::redact(&String::from_utf8_lossy(&output.stdout))
    );
    eprint!(
        "{}",
        Policy::redact(&String::from_utf8_lossy(&output.stderr))
    );
    if !output.status.success() {
        anyhow::bail!("command exited with {}", output.status);
    }
    println!("\n[exit 0]");
    Ok(())
}

fn deny_destructive_command(command: &str) -> Result<()> {
    let normalized = command.to_ascii_lowercase();
    for denied in [
        "rm -rf",
        "rm -fr",
        "sudo ",
        "mkfs",
        "shutdown",
        "reboot",
        ":(){",
        "dd if=",
        "> /dev/",
        "chmod -r 777",
        "git reset --hard",
        "git clean -fd",
    ] {
        if normalized.contains(denied) {
            anyhow::bail!("command denied by safety denylist: {denied}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_variants_are_denied() {
        for command in [
            "rm -fr ./data",
            "git reset --hard",
            "DD IF=/dev/zero OF=disk",
        ] {
            assert!(deny_destructive_command(command).is_err(), "{command}");
        }
        assert!(deny_destructive_command("cargo test --workspace").is_ok());
    }
}
