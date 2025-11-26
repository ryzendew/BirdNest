use anyhow::Result;
use colored::*;
use std::process::{Command, Stdio};

pub fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    
    print!("{} [y/N]: ", prompt.yellow().bold());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

fn is_root() -> bool {
    std::env::var("USER").unwrap_or_default() == "root" || 
    unsafe { libc::geteuid() == 0 }
}

fn check_sudo_available() -> Result<()> {
    // Try to run sudo --version to check if it's available
    let output = Command::new("sudo")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    
    if output.is_err() {
        anyhow::bail!("sudo is not available. Please install sudo or run as root.");
    }
    
    Ok(())
}

pub fn run_command(cmd: &str, args: &[&str], sudo: bool) -> Result<String> {
    let output = if sudo {
        // Skip sudo if already root
        if is_root() {
            Command::new(cmd)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        } else {
            // Check if sudo is available
            check_sudo_available()?;
            Command::new("sudo")
                .arg(cmd)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        }
    } else {
        Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_command_interactive(cmd: &str, args: &[&str], sudo: bool) -> Result<()> {
    let status = if sudo {
        // Skip sudo if already root
        if is_root() {
            Command::new(cmd)
                .args(args)
                .status()?
        } else {
            // Check if we're in a GUI environment (DISPLAY or WAYLAND_DISPLAY set)
            let is_gui = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
            
            if is_gui {
                // Use pkexec for GUI applications (shows polkit GUI password dialog)
                print_info("Elevated privileges required, using pkexec...");
                
                let mut pkexec_cmd = Command::new("pkexec");
                pkexec_cmd.arg(cmd);
                pkexec_cmd.args(args);
                
                // Preserve DISPLAY environment variable for GUI password dialogs
                if let Ok(display) = std::env::var("DISPLAY") {
                    pkexec_cmd.env("DISPLAY", display);
                }
                
                // Preserve XAUTHORITY if set (for X11 GUI password dialogs)
                if let Ok(xauth) = std::env::var("XAUTHORITY") {
                    pkexec_cmd.env("XAUTHORITY", xauth);
                }
                
                // Preserve WAYLAND_DISPLAY for Wayland
                if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
                    pkexec_cmd.env("WAYLAND_DISPLAY", wayland);
                }
                
                pkexec_cmd.status()?
            } else {
                // Fallback to sudo for non-GUI environments
                check_sudo_available()?;
                print_info("Elevated privileges required, using sudo...");
                
                let mut sudo_cmd = Command::new("sudo");
                sudo_cmd.arg(cmd);
                sudo_cmd.args(args);
                
                sudo_cmd.status()?
            }
        }
    } else {
        Command::new(cmd)
            .args(args)
            .status()?
    };

    if !status.success() {
        // Check if it's a password cancellation
        if status.code() == Some(1) {
            anyhow::bail!("Authentication failed or cancelled. Please try again.");
        }
        anyhow::bail!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

#[allow(dead_code)]
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

