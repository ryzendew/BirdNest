use anyhow::Result;
use crate::utils::{self, confirm, run_command, run_command_interactive};
use crate::config::Config;

pub struct PackageManager {
    manager: PackageManagerType,
    #[allow(dead_code)]
    config: Config,
}

#[derive(Debug, Clone)]
enum PackageManagerType {
    Pikman,
    Apt,
}

impl PackageManager {
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        let manager = Self::detect_manager()?;
        
        Ok(Self { manager, config })
    }

    fn detect_manager() -> Result<PackageManagerType> {
        use std::process::Command;
        
        // Check for pikman first (PikaOS specific)
        if Command::new("pikman").arg("--version").output().is_ok() {
            return Ok(PackageManagerType::Pikman);
        }
        
        // Fallback to apt
        if Command::new("apt").arg("--version").output().is_ok() {
            return Ok(PackageManagerType::Apt);
        }
        
        anyhow::bail!("No supported package manager found (pikman or apt)");
    }

    pub fn install(&self, packages: &[String], yes: bool, distro: Option<&str>) -> Result<()> {
        if packages.is_empty() {
            anyhow::bail!("No packages specified");
        }

        if !yes && !confirm(&format!("Install {} package(s)?", packages.len()))? {
            utils::print_info("Installation cancelled");
            return Ok(());
        }

        match &self.manager {
            PackageManagerType::Pikman => {
                let mut args = vec!["install"];
                
                // Add distro-specific flags
                match distro {
                    Some("aur") => args.push("--aur"),
                    Some("fedora") => args.push("--fedora"),
                    Some("alpine") => args.push("--alpine"),
                    _ => {}
                }
                
                args.extend(packages.iter().map(|s| s.as_str()));
                if yes {
                    args.push("-y");
                }
                run_command_interactive("pikman", &args, false)?;
            }
            PackageManagerType::Apt => {
                // Distro flags only work with pikman
                if distro.is_some() {
                    anyhow::bail!("Distro-specific flags (--aur, --fedora, --alpine) only work with pikman");
                }
                let mut args = vec!["install", "-y"];
                args.extend(packages.iter().map(|s| s.as_str()));
                run_command_interactive("apt", &args, true)?;
            }
        }

        utils::print_success(&format!("Successfully installed {} package(s)", packages.len()));
        Ok(())
    }

    pub fn remove(&self, packages: &[String], yes: bool, autoremove: bool) -> Result<()> {
        if packages.is_empty() {
            anyhow::bail!("No packages specified");
        }

        if !yes && !confirm(&format!("Remove {} package(s)?", packages.len()))? {
            utils::print_info("Removal cancelled");
            return Ok(());
        }

        match &self.manager {
            PackageManagerType::Pikman => {
                let mut args = vec!["remove"];
                args.extend(packages.iter().map(|s| s.as_str()));
                if yes {
                    args.push("-y");
                }
                if autoremove {
                    args.push("--autoremove");
                }
                run_command_interactive("pikman", &args, false)?;
            }
            PackageManagerType::Apt => {
                let mut args = vec!["remove", "-y"];
                args.extend(packages.iter().map(|s| s.as_str()));
                if autoremove {
                    args.push("--autoremove");
                }
                run_command_interactive("apt", &args, true)?;
            }
        }

        utils::print_success(&format!("Successfully removed {} package(s)", packages.len()));
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<()> {
        // Always use apt for search
        let output = run_command("apt", &["search", query], false)?;
        print!("{}", output);
        Ok(())
    }

    pub fn pikman_search(&self, query: &str) -> Result<()> {
        // Directly use pikman for search, regardless of detected manager
        let output = run_command("pikman", &["search", query], false)?;
        print!("{}", output);
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        utils::print_info("Updating package lists...");
        
        match &self.manager {
            PackageManagerType::Pikman => {
                run_command_interactive("pikman", &["update"], false)?;
            }
            PackageManagerType::Apt => {
                run_command_interactive("apt", &["update"], true)?;
            }
        }

        utils::print_success("Package lists updated");
        Ok(())
    }

    pub fn upgrade(&self, packages: &[String], yes: bool) -> Result<()> {
        if !yes && !confirm("Upgrade packages?")? {
            utils::print_info("Upgrade cancelled");
            return Ok(());
        }

        match &self.manager {
            PackageManagerType::Pikman => {
                if packages.is_empty() {
                    let mut args = vec!["upgrade"];
                    if yes {
                        args.push("-y");
                    }
                    run_command_interactive("pikman", &args, false)?;
                } else {
                    let mut args = vec!["upgrade"];
                    args.extend(packages.iter().map(|s| s.as_str()));
                    if yes {
                        args.push("-y");
                    }
                    run_command_interactive("pikman", &args, false)?;
                }
            }
            PackageManagerType::Apt => {
                if packages.is_empty() {
                    let args = vec!["upgrade", "-y"];
                    run_command_interactive("apt", &args, true)?;
                } else {
                    let mut args = vec!["install", "--upgrade", "-y"];
                    args.extend(packages.iter().map(|s| s.as_str()));
                    run_command_interactive("apt", &args, true)?;
                }
            }
        }

        utils::print_success("Packages upgraded");
        Ok(())
    }

    pub fn list(&self, upgradable: bool) -> Result<()> {
        let output = match &self.manager {
            PackageManagerType::Pikman => {
                if upgradable {
                    run_command("pikman", &["list", "--upgradable"], false)?
                } else {
                    run_command("pikman", &["list", "--installed"], false)?
                }
            }
            PackageManagerType::Apt => {
                if upgradable {
                    run_command("apt", &["list", "--upgradable"], false)?
                } else {
                    run_command("dpkg", &["-l"], false)?
                }
            }
        };

        print!("{}", output);
        Ok(())
    }

    pub fn show(&self, package: &str) -> Result<()> {
        let output = match &self.manager {
            PackageManagerType::Pikman => {
                run_command("pikman", &["show", package], false)?
            }
            PackageManagerType::Apt => {
                run_command("apt", &["show", package], false)?
            }
        };

        print!("{}", output);
        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        utils::print_info("Cleaning package cache...");
        
        match &self.manager {
            PackageManagerType::Pikman => {
                run_command_interactive("pikman", &["clean"], false)?;
            }
            PackageManagerType::Apt => {
                run_command_interactive("apt", &["clean"], true)?;
                run_command_interactive("apt", &["autoclean"], true)?;
            }
        }

        utils::print_success("Cache cleaned");
        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        let manager_name = match &self.manager {
            PackageManagerType::Pikman => "pikman",
            PackageManagerType::Apt => "apt",
        };

        println!("Package Manager: {}", manager_name);
        
        // Check for updates
        self.update()?;
        
        // Show upgradable packages
        println!("\nUpgradable packages:");
        self.list(true)?;
        
        Ok(())
    }

    // Pikman-specific commands
    pub fn pikman_autoremove(&self, yes: bool) -> Result<()> {
        if !yes && !confirm("Remove all unused packages?")? {
            utils::print_info("Autoremove cancelled");
            return Ok(());
        }

        let mut args = vec!["autoremove"];
        if yes {
            args.push("-y");
        }
        run_command_interactive("pikman", &args, false)?;
        utils::print_success("Unused packages removed");
        Ok(())
    }

    pub fn pikman_enter(&self, name: &str) -> Result<()> {
        run_command_interactive("pikman", &["enter", name], false)?;
        Ok(())
    }

    pub fn pikman_export(&self, package: &str, name: Option<&str>) -> Result<()> {
        let mut args = vec!["export", package];
        if let Some(n) = name {
            args.push("--name");
            args.push(n);
        }
        run_command_interactive("pikman", &args, false)?;
        utils::print_success(&format!("Desktop entry exported for {}", package));
        Ok(())
    }

    pub fn pikman_init(&self, name: &str, manager: Option<&str>) -> Result<()> {
        let mut args = vec!["init", name];
        if let Some(mgr) = manager {
            args.push("--manager");
            args.push(mgr);
        }
        run_command_interactive("pikman", &args, false)?;
        utils::print_success(&format!("Container {} initialized", name));
        Ok(())
    }

    pub fn pikman_log(&self) -> Result<()> {
        let output = run_command("pikman", &["log"], false)?;
        print!("{}", output);
        Ok(())
    }

    pub fn pikman_purge(&self, packages: &[String], yes: bool) -> Result<()> {
        if packages.is_empty() {
            anyhow::bail!("No packages specified");
        }

        if !yes && !confirm(&format!("Fully purge {} package(s)?", packages.len()))? {
            utils::print_info("Purge cancelled");
            return Ok(());
        }

        let mut args = vec!["purge"];
        args.extend(packages.iter().map(|s| s.as_str()));
        if yes {
            args.push("-y");
        }
        run_command_interactive("pikman", &args, false)?;
        utils::print_success(&format!("Successfully purged {} package(s)", packages.len()));
        Ok(())
    }

    pub fn pikman_run(&self, name: &str, command: &[String]) -> Result<()> {
        if command.is_empty() {
            anyhow::bail!("No command specified");
        }

        let mut args = vec!["run", name];
        args.extend(command.iter().map(|s| s.as_str()));
        run_command_interactive("pikman", &args, false)?;
        Ok(())
    }

    pub fn pikman_upgrades(&self) -> Result<()> {
        let output = run_command("pikman", &["upgrades"], false)?;
        print!("{}", output);
        Ok(())
    }

    pub fn pikman_unexport(&self, package: &str, name: Option<&str>) -> Result<()> {
        let mut args = vec!["unexport", package];
        if let Some(n) = name {
            args.push("--name");
            args.push(n);
        }
        run_command_interactive("pikman", &args, false)?;
        utils::print_success(&format!("Desktop entry removed for {}", package));
        Ok(())
    }
}

