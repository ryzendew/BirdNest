use anyhow::Result;
use crate::utils::{self, confirm, run_command, run_command_interactive};

pub struct FlatpakManager;

impl FlatpakManager {
    pub fn new() -> Result<Self> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::new() called");
        use std::process::Command;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Checking if flatpak is installed...");
        if Command::new("flatpak").arg("--version").output().is_err() {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] ERROR: flatpak is not installed");
            anyhow::bail!("flatpak is not installed");
        }
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] flatpak is installed, FlatpakManager created successfully");
        Ok(Self)
    }

    pub fn install(&self, packages: &[String], yes: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::install() called with {} packages, yes={}", packages.len(), yes);
        if packages.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] ERROR: No packages specified");
            anyhow::bail!("No packages specified");
        }

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Packages to install: {:?}", packages);

        if !yes && !confirm(&format!("Install {} flatpak(s)?", packages.len()))? {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Installation cancelled by user");
            utils::print_info("Installation cancelled");
            return Ok(());
        }

        let mut args = vec!["install", "-y"];
        args.extend(packages.iter().map(|s| s.as_str()));
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak {}", args.join(" "));
        run_command_interactive("flatpak", &args, false)?;

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Installation completed successfully");
        utils::print_success(&format!("Successfully installed {} flatpak(s)", packages.len()));
        Ok(())
    }

    pub fn remove(&self, packages: &[String], yes: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::remove() called with {} packages, yes={}", packages.len(), yes);
        if packages.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] ERROR: No packages specified");
            anyhow::bail!("No packages specified");
        }

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Packages to remove: {:?}", packages);

        if !yes && !confirm(&format!("Remove {} flatpak(s)?", packages.len()))? {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Removal cancelled by user");
            utils::print_info("Removal cancelled");
            return Ok(());
        }

        let mut args = vec!["uninstall", "-y"];
        args.extend(packages.iter().map(|s| s.as_str()));
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak {}", args.join(" "));
        run_command_interactive("flatpak", &args, false)?;

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Removal completed successfully");
        utils::print_success(&format!("Successfully removed {} flatpak(s)", packages.len()));
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::search() called with query: '{}'", query);
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak search {}", query);
        let output = run_command("flatpak", &["search", query], false)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Search completed, output length: {} bytes", output.len());
        print!("{}", output);
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::update() called");
        utils::print_info("Updating flatpak repositories...");
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak update --noninteractive");
        run_command_interactive("flatpak", &["update", "--noninteractive"], false)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Repository update completed successfully");
        utils::print_success("Flatpak repositories updated");
        Ok(())
    }

    pub fn upgrade(&self, packages: &[String], yes: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::upgrade() called with {} packages, yes={}", packages.len(), yes);
        if !yes && !confirm("Upgrade flatpaks?")? {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Upgrade cancelled by user");
            utils::print_info("Upgrade cancelled");
            return Ok(());
        }

        if packages.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Upgrading all flatpaks, executing: flatpak update -y");
            run_command_interactive("flatpak", &["update", "-y"], false)?;
        } else {
            let mut args = vec!["update", "-y"];
            args.extend(packages.iter().map(|s| s.as_str()));
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Upgrading specific packages, executing: flatpak {}", args.join(" "));
            run_command_interactive("flatpak", &args, false)?;
        }

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Upgrade completed successfully");
        utils::print_success("Flatpaks upgraded");
        Ok(())
    }

    pub fn list(&self, upgradable: bool) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::list() called, upgradable={}", upgradable);
        let output = if upgradable {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Listing upgradable packages, executing: flatpak update --dry-run");
            run_command("flatpak", &["update", "--dry-run"], false)?
        } else {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] Listing all packages, executing: flatpak list");
            run_command("flatpak", &["list"], false)?
        };

        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] List completed, output length: {} bytes", output.len());
        print!("{}", output);
        Ok(())
    }

    pub fn show(&self, package: &str) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::show() called for package: '{}'", package);
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak info {}", package);
        let output = run_command("flatpak", &["info", package], false)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Show completed, output length: {} bytes", output.len());
        print!("{}", output);
        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] FlatpakManager::clean() called");
        utils::print_info("Cleaning flatpak cache...");
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Executing command: flatpak uninstall --unused -y");
        run_command_interactive("flatpak", &["uninstall", "--unused", "-y"], false)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] Clean completed successfully");
        utils::print_success("Flatpak cache cleaned");
        Ok(())
    }
}

