use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::package_manager::PackageManager;
use crate::flatpak::FlatpakManager;

#[derive(Parser)]
#[clap(name = "birdnest")]
#[clap(about = "A unified package manager for PikaOS", long_about = "A unified package manager for PikaOS supporting pikman, apt, and flatpak.\n\nPikman can install packages from multiple distributions:\n  --aur: Install Arch packages (including from the AUR)\n  --fedora: Install Fedora packages\n  --alpine: Install Alpine packages\n\nUse 'pikman' subcommand for pikman-specific commands:\n  autoremove, enter, export, init, log, purge, run, upgrades, unexport")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install packages
    Install {
        /// Package names to install
        packages: Vec<String>,
        /// Use flatpak instead of system package manager
        #[clap(short, long)]
        flatpak: bool,
        /// Install Arch packages (including from the AUR) via pikman
        #[clap(long, conflicts_with = "fedora", conflicts_with = "alpine")]
        aur: bool,
        /// Install Fedora packages via pikman
        #[clap(long, conflicts_with = "aur", conflicts_with = "alpine")]
        fedora: bool,
        /// Install Alpine packages via pikman
        #[clap(long, conflicts_with = "aur", conflicts_with = "fedora")]
        alpine: bool,
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    /// Remove packages
    Remove {
        /// Package names to remove
        packages: Vec<String>,
        /// Use flatpak instead of system package manager
        #[clap(short, long)]
        flatpak: bool,
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
        /// Remove unused dependencies
        #[clap(short, long)]
        autoremove: bool,
    },
    /// Search for packages
    Search {
        /// Search query
        query: String,
        /// Search in flatpak repositories
        #[clap(short, long)]
        flatpak: bool,
    },
    /// Search for packages using pikman
    PikmanSearch {
        /// Search query
        query: String,
    },
    /// Update package lists
    Update {
        /// Update flatpak repositories
        #[clap(short, long)]
        flatpak: bool,
    },
    /// Upgrade installed packages
    Upgrade {
        /// Package names to upgrade (if empty, upgrade all)
        packages: Vec<String>,
        /// Use flatpak instead of system package manager
        #[clap(short, long)]
        flatpak: bool,
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    /// List installed packages
    List {
        /// Show only upgradable packages
        #[clap(short, long)]
        upgradable: bool,
        /// Use flatpak instead of system package manager
        #[clap(short, long)]
        flatpak: bool,
    },
    /// Show package information
    Show {
        /// Package name
        package: String,
        /// Use flatpak instead of system package manager
        #[clap(short, long)]
        flatpak: bool,
    },
    /// Show install dialog (internal use)
    InstallDialog {
        /// Package names to install
        packages: Vec<String>,
        /// Mark packages as flatpak
        #[clap(long)]
        flatpak: bool,
    },
    /// Show remove dialog (internal use)
    RemoveDialog {
        /// Package names to remove
        packages: Vec<String>,
        /// Mark packages as flatpak
        #[clap(long)]
        flatpak: bool,
    },
    /// Show conflict dialog (internal use)
    ConflictDialog {
        /// Package names that couldn't be removed (space-separated)
        packages: Vec<String>,
        /// Conflict message (passed via --message flag)
        #[clap(long)]
        message: Option<String>,
        /// Terminal output (passed via --output flag)
        #[clap(long)]
        output: Option<String>,
    },
    /// Clean package cache
    Clean {
        /// Clean flatpak cache
        #[clap(short, long)]
        flatpak: bool,
    },
    /// Show package manager status
    Status,
    /// Install flatpak packages
    FlatpakInstall {
        /// Flatpak package names to install
        packages: Vec<String>,
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    /// Search for flatpak packages
    FlatpakSearch {
        /// Search query
        query: String,
    },
    /// Update flatpak repositories
    FlatpakUpdate,
    /// Pikman-specific commands (autoremove, enter, export, init, log, purge, run, upgrades, unexport)
    Pikman {
        #[clap(subcommand)]
        subcommand: PikmanSubcommand,
    },
}

#[derive(Subcommand)]
pub enum PikmanSubcommand {
    /// Remove all unused packages
    Autoremove {
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    /// Enter the container instance for select package manager
    Enter {
        /// Container name
        name: String,
    },
    /// Export/Recreate a program's desktop entry from the container
    Export {
        /// Package name
        package: String,
        /// Container name
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Initialize a managed container
    Init {
        /// Container name
        name: String,
        /// Package manager type (arch, fedora, alpine)
        #[clap(short, long)]
        manager: Option<String>,
    },
    /// Show package manager logs
    Log,
    /// Fully purge a package
    Purge {
        /// Package names to purge
        packages: Vec<String>,
        /// Don't ask for confirmation
        #[clap(short, long)]
        yes: bool,
    },
    /// Run a command inside a managed container
    Run {
        /// Container name
        name: String,
        /// Command to run
        command: Vec<String>,
    },
    /// List the available upgrades
    Upgrades,
    /// Unexport/Remove a program's desktop entry
    Unexport {
        /// Package name
        package: String,
        /// Container name
        #[clap(short, long)]
        name: Option<String>,
    },
}

// SystemUpdateSubcommand removed - system updates handled by separate app

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Install { packages, flatpak, aur, fedora, alpine, yes } => {
                if flatpak {
                    FlatpakManager::new()?.install(&packages, yes)?;
                } else {
                    let distro = if aur {
                        Some("aur")
                    } else if fedora {
                        Some("fedora")
                    } else if alpine {
                        Some("alpine")
                    } else {
                        None
                    };
                    PackageManager::new()?.install(&packages, yes, distro)?;
                }
            }
            Commands::Remove { packages, flatpak, yes, autoremove } => {
                if flatpak {
                    FlatpakManager::new()?.remove(&packages, yes)?;
                } else {
                    PackageManager::new()?.remove(&packages, yes, autoremove)?;
                }
            }
            Commands::Search { query, flatpak } => {
                if flatpak {
                    FlatpakManager::new()?.search(&query)?;
                } else {
                    PackageManager::new()?.search(&query)?;
                }
            }
            Commands::PikmanSearch { query } => {
                PackageManager::new()?.pikman_search(&query)?;
            }
            Commands::Update { flatpak } => {
                if flatpak {
                    FlatpakManager::new()?.update()?;
                } else {
                    PackageManager::new()?.update()?;
                }
            }
            Commands::Upgrade { packages, flatpak, yes } => {
                if flatpak {
                    FlatpakManager::new()?.upgrade(&packages, yes)?;
                } else {
                    PackageManager::new()?.upgrade(&packages, yes)?;
                }
            }
            Commands::List { upgradable, flatpak } => {
                if flatpak {
                    FlatpakManager::new()?.list(upgradable)?;
                } else {
                    PackageManager::new()?.list(upgradable)?;
                }
            }
            Commands::Show { package, flatpak } => {
                if flatpak {
                    FlatpakManager::new()?.show(&package)?;
                } else {
                    PackageManager::new()?.show(&package)?;
                }
            }
            // SystemUpdate command removed - handled by separate app
            Commands::Clean { flatpak } => {
                if flatpak {
                    FlatpakManager::new()?.clean()?;
                } else {
                    PackageManager::new()?.clean()?;
                }
            }
            Commands::Status => {
                PackageManager::new()?.status()?;
            }
            Commands::FlatpakInstall { packages, yes } => {
                FlatpakManager::new()?.install(&packages, yes)?;
            }
            Commands::FlatpakSearch { query } => {
                FlatpakManager::new()?.search(&query)?;
            }
            Commands::FlatpakUpdate => {
                FlatpakManager::new()?.update()?;
            }
            Commands::Pikman { subcommand } => {
                let pkg_manager = PackageManager::new()?;
                match subcommand {
                    PikmanSubcommand::Autoremove { yes } => {
                        pkg_manager.pikman_autoremove(yes)?;
                    }
                    PikmanSubcommand::Enter { name } => {
                        pkg_manager.pikman_enter(&name)?;
                    }
                    PikmanSubcommand::Export { package, name } => {
                        pkg_manager.pikman_export(&package, name.as_deref())?;
                    }
                    PikmanSubcommand::Init { name, manager } => {
                        pkg_manager.pikman_init(&name, manager.as_deref())?;
                    }
                    PikmanSubcommand::Log => {
                        pkg_manager.pikman_log()?;
                    }
                    PikmanSubcommand::Purge { packages, yes } => {
                        pkg_manager.pikman_purge(&packages, yes)?;
                    }
                    PikmanSubcommand::Run { name, command } => {
                        pkg_manager.pikman_run(&name, &command)?;
                    }
                    PikmanSubcommand::Upgrades => {
                        pkg_manager.pikman_upgrades()?;
                    }
                    PikmanSubcommand::Unexport { package, name } => {
                        pkg_manager.pikman_unexport(&package, name.as_deref())?;
                    }
                }
            }
            Commands::InstallDialog { packages, flatpak } => {
                use crate::gui::install_dialog::InstallDialog;
                // Pass flatpak flag to the dialog
                InstallDialog::run_separate_window_with_flatpak_flag(packages, flatpak)?;
            }
            Commands::RemoveDialog { packages, flatpak } => {
                use crate::gui::remove_dialog::RemoveDialog;
                RemoveDialog::run_separate_window_with_flatpak_flag(packages, flatpak)?;
            }
            Commands::ConflictDialog { packages, message, output } => {
                use crate::gui::conflict_dialog::ConflictDialog;
                let conflict_msg = message.unwrap_or_else(|| "Unknown conflict".to_string());
                let terminal_output = output.unwrap_or_default();
                ConflictDialog::run_separate_window(packages, conflict_msg, terminal_output)?;
            }
        }
        Ok(())
    }
}

