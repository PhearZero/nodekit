use std::process::Command;
use anyhow::{Result, bail};

pub fn upgrade_algorand() -> Result<()> {
    let (cmd, args) = if command_exists("apt-get") {
        ("sudo", vec!["apt-get", "update", "&&", "sudo", "apt-get", "install", "--only-upgrade", "-y", "algorand"])
    } else if command_exists("dnf") {
        ("sudo", vec!["dnf", "update", "-y", "--refresh", "algorand"])
    } else {
        bail!("No supported package manager found (apt-get or dnf required)")
    };

    // For apt-get with && we need to run it through sh
    let status = if cmd == "sudo" && args.contains(&"&&") {
        Command::new("sh")
            .arg("-c")
            .arg("sudo apt-get update && sudo apt-get install --only-upgrade -y algorand")
            .status()?
    } else {
        Command::new(cmd)
            .args(&args)
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        bail!("Upgrade command failed with status: {}", status)
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
