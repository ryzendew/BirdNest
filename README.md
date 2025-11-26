# BirdNest

A unified package manager CLI for PikaOS written in Rust. BirdNest provides a single interface for managing packages from multiple sources: pikman, apt, and flatpak.

## Features

- **Unified Interface**: Manage packages from pikman, apt, and flatpak through one CLI
- **Package Operations**: Install, remove, search, update, and upgrade packages
- **System Updates**: Check for updates, list available updates, and apply them when ready
- **Smart Detection**: Automatically detects available package managers (pikman or apt)
- **Flatpak Support**: Full flatpak integration for application management
- **User-Friendly**: Colorized output and confirmation prompts

## Installation

### Building from Source

```bash
git clone <repository-url>
cd BirdNest
cargo build --release
sudo cp target/release/birdnest /usr/local/bin/
```

## Usage

### Install Packages

```bash
# Install system packages
birdnest install package1 package2

# Install flatpaks
birdnest install --flatpak app1 app2

# Install without confirmation
birdnest install -y package1
```

### Remove Packages

```bash
# Remove system packages
birdnest remove package1 package2

# Remove flatpaks
birdnest remove --flatpak app1

# Remove with autoremove
birdnest remove --autoremove package1
```

### Search for Packages

```bash
# Search system packages
birdnest search query

# Search flatpaks
birdnest search --flatpak query
```

### Update Package Lists

```bash
# Update system package lists
birdnest update

# Update flatpak repositories
birdnest update --flatpak
```

### Upgrade Packages

```bash
# Upgrade all packages
birdnest upgrade

# Upgrade specific packages
birdnest upgrade package1 package2

# Upgrade flatpaks
birdnest upgrade --flatpak
```

### List Packages

```bash
# List installed packages
birdnest list

# List upgradable packages
birdnest list --upgradable

# List flatpaks
birdnest list --flatpak
```

### Show Package Information

```bash
# Show package info
birdnest show package-name

# Show flatpak info
birdnest show --flatpak app-name
```
### Clean Cache

```bash
# Clean system package cache
birdnest clean

# Clean flatpak cache
birdnest clean --flatpak
```

### Status

```bash
# Show package manager status
birdnest status
```

## Configuration

Configuration is stored in `~/.config/birdnest/config.json`. The default configuration includes:

- `package_manager`: Auto-detection mode ("auto")
- `auto_confirm`: Automatically confirm operations (false)
- `flatpak_enabled`: Enable flatpak support (true)

## Requirements

- Rust 1.85 or later
- PikaOS (or compatible Linux distribution)
- pikman or apt (for system package management)
- flatpak (optional, for flatpak support)

## Architecture

- `src/main.rs`: Entry point
- `src/cli.rs`: CLI argument parsing and command routing
- `src/package_manager.rs`: Abstraction layer for pikman/apt
- `src/flatpak.rs`: Flatpak management
- `src/system_update.rs`: System update checking and management
- `src/config.rs`: Configuration management
- `src/utils.rs`: Utility functions for command execution and output

## License

MIT OR Apache-2.0






