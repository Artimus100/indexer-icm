#!/bin/bash

# PumpDotFun Deployment Script
# This script builds and deploys the application to the production directory

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_DIR="/opt/pumpdotfun"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${GREEN}PumpDotFun Deployment Script${NC}"
echo "============================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}" 
   exit 1
fi

# Build the application
echo -e "${YELLOW}Building application in release mode...${NC}"
cd "$PROJECT_DIR"
cargo build --release
echo -e "${GREEN}Build completed${NC}"

# Stop services if running
echo -e "${YELLOW}Stopping services...${NC}"
systemctl stop pumpdotfun.target || true
echo -e "${GREEN}Services stopped${NC}"

# Create target directory
mkdir -p "$APP_DIR/target/release"

# Copy binaries
echo -e "${YELLOW}Deploying binaries...${NC}"
cp "$PROJECT_DIR/target/release/core" "$APP_DIR/target/release/"
cp "$PROJECT_DIR/target/release/api" "$APP_DIR/target/release/"
cp "$PROJECT_DIR/target/release/db-processor" "$APP_DIR/target/release/"
cp "$PROJECT_DIR/target/release/klines-engine" "$APP_DIR/target/release/"
cp "$PROJECT_DIR/target/release/wss" "$APP_DIR/target/release/"
echo -e "${GREEN}Binaries deployed${NC}"

# Copy configuration files
echo -e "${YELLOW}Deploying configuration files...${NC}"
cp "$PROJECT_DIR/core/Vixen.toml" "$APP_DIR/" || echo -e "${YELLOW}Vixen.toml not found, skipping${NC}"

# Copy .env file if it doesn't exist
if [ ! -f "$APP_DIR/.env" ]; then
    if [ -f "$PROJECT_DIR/.env" ]; then
        cp "$PROJECT_DIR/.env" "$APP_DIR/"
        echo -e "${GREEN}.env file copied${NC}"
    else
        echo -e "${YELLOW}No .env file found. Please create one based on .env.example${NC}"
    fi
fi

# Set permissions
echo -e "${YELLOW}Setting permissions...${NC}"
chown -R pumpdotfun:pumpdotfun "$APP_DIR"
chmod +x "$APP_DIR/target/release/"*
echo -e "${GREEN}Permissions set${NC}"

# Start services
echo -e "${YELLOW}Starting services...${NC}"
systemctl start pumpdotfun.target
echo -e "${GREEN}Services started${NC}"

# Check status
echo -e "${YELLOW}Checking service status...${NC}"
sleep 5
systemctl status pumpdotfun.target --no-pager

echo ""
echo -e "${GREEN}Deployment completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Service status commands:${NC}"
echo "Check all services: systemctl status pumpdotfun.target"
echo "View logs:          journalctl -u pumpdotfun-core -f"
echo "Stop services:      systemctl stop pumpdotfun.target"
echo "Restart services:   systemctl restart pumpdotfun.target"
