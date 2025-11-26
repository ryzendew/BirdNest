use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod package_manager;
mod flatpak;
mod utils;
mod gui;

use cli::Cli;

fn main() -> Result<()> {
    eprintln!("[DEBUG] BirdNest starting...");
    eprintln!("[DEBUG] Arguments: {:?}", std::env::args().collect::<Vec<String>>());
    
    let args: Vec<String> = std::env::args().collect();
    
    // If no arguments provided, launch GUI
    if args.len() == 1 {
        eprintln!("[DEBUG] No CLI arguments, launching GUI...");
        match gui::run() {
            Ok(_) => {
                eprintln!("[DEBUG] GUI exited successfully");
        Ok(())
            }
            Err(e) => {
                eprintln!("[ERROR] GUI failed: {:?}", e);
                Err(e.into())
            }
        }
    } else {
        eprintln!("[DEBUG] CLI arguments provided, using CLI mode...");
        let cli = Cli::parse();
        eprintln!("[DEBUG] CLI parsed successfully, running command...");
        match cli.run() {
            Ok(_) => {
                eprintln!("[DEBUG] CLI command completed successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("[ERROR] CLI command failed: {:?}", e);
                Err(e)
            }
        }
    }
}

