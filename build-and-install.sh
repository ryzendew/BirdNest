#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building BirdNest...${NC}"

# Build the project
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${YELLOW}Running as root, proceeding with installation...${NC}"
else
    echo -e "${YELLOW}Not running as root, will use sudo for installation...${NC}"
    SUDO="sudo"
fi

# Installation paths
BINARY_NAME="birdnest"
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
ICON_DIR="/usr/share/pixmaps"
DESKTOP_FILE="$DESKTOP_DIR/com.github.birdnest.desktop"

# Find PikaOS logo
PIKA_ICON=""
if [ -f "/usr/share/pixmaps/pika-logo.svg" ]; then
    PIKA_ICON="/usr/share/pixmaps/pika-logo.svg"
elif [ -f "/usr/share/pixmaps/pika-logo-duotone.svg" ]; then
    PIKA_ICON="/usr/share/pixmaps/pika-logo-duotone.svg"
elif [ -f "/usr/share/icons/desktop-base/scalable/emblems/emblem-pika.svg" ]; then
    PIKA_ICON="/usr/share/icons/desktop-base/scalable/emblems/emblem-pika.svg"
else
    echo -e "${YELLOW}Warning: PikaOS logo not found, using default icon${NC}"
    PIKA_ICON=""
fi

echo -e "${GREEN}Installing binary to $INSTALL_DIR...${NC}"
$SUDO cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
$SUDO chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo -e "${GREEN}Creating desktop entry...${NC}"

# Create desktop file
cat > /tmp/birdnest.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=BirdNest
GenericName=Package Manager
Comment=Unified package manager for PikaOS supporting pikman, apt, and flatpak
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=${PIKA_ICON:-application-x-executable}
Terminal=false
Categories=System;PackageManager;
Keywords=package;manager;pikman;apt;flatpak;
StartupNotify=true
EOF

$SUDO cp /tmp/birdnest.desktop "$DESKTOP_FILE"
$SUDO chmod 644 "$DESKTOP_FILE"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo -e "${GREEN}Updating desktop database...${NC}"
    $SUDO update-desktop-database "$DESKTOP_DIR"
fi

echo -e "${GREEN}Installation complete!${NC}"
echo -e "${GREEN}BirdNest has been installed to $INSTALL_DIR/$BINARY_NAME${NC}"
echo -e "${GREEN}Desktop entry created at $DESKTOP_FILE${NC}"
if [ -n "$PIKA_ICON" ]; then
    echo -e "${GREEN}Using PikaOS icon: $PIKA_ICON${NC}"
fi




