#!/bin/bash

# PumpDotFun Systemd Setup Script
# This script sets up systemd services for the PumpDotFun application

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_USER="pumpdotfun"
APP_GROUP="pumpdotfun"
APP_DIR="/opt/pumpdotfun"
SYSTEMD_DIR="/etc/systemd/system"
CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${GREEN}PumpDotFun Systemd Setup Script${NC}"
echo "================================"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}" 
   exit 1
fi

# Create application user and group
echo -e "${YELLOW}Creating application user and group...${NC}"
if ! id "$APP_USER" &>/dev/null; then
    useradd --system --shell /bin/false --home-dir "$APP_DIR" --create-home "$APP_USER"
    echo -e "${GREEN}Created user: $APP_USER${NC}"
else
    echo -e "${YELLOW}User $APP_USER already exists${NC}"
fi

# Create application directory
echo -e "${YELLOW}Setting up application directory...${NC}"
mkdir -p "$APP_DIR"
chown -R "$APP_USER:$APP_GROUP" "$APP_DIR"
echo -e "${GREEN}Application directory ready: $APP_DIR${NC}"

# Copy source code and configuration
echo -e "${YELLOW}Copying source code and configuration...${NC}"
# Go to parent directory to copy the whole project
cd "$CURRENT_DIR/.."
cp -r . "$APP_DIR/"
# Copy Vixen.toml to core directory
cp "$APP_DIR/core/Vixen.toml" "$APP_DIR/core/" 2>/dev/null || echo -e "${YELLOW}Vixen.toml not found in core directory, please ensure it exists${NC}"
chown -R "$APP_USER:$APP_GROUP" "$APP_DIR"
echo -e "${GREEN}Source code and configuration copied${NC}"

# Copy systemd service files
echo -e "${YELLOW}Installing systemd service files...${NC}"
cp "$CURRENT_DIR"/*.service "$SYSTEMD_DIR/"
cp "$CURRENT_DIR"/*.target "$SYSTEMD_DIR/"
chmod 644 "$SYSTEMD_DIR"/pumpdotfun-*.service
chmod 644 "$SYSTEMD_DIR"/pumpdotfun.target
echo -e "${GREEN}Systemd service files installed${NC}"

# Reload systemd
echo -e "${YELLOW}Reloading systemd daemon...${NC}"
systemctl daemon-reload
echo -e "${GREEN}Systemd daemon reloaded${NC}"

# Enable services
echo -e "${YELLOW}Enabling services...${NC}"
systemctl enable pumpdotfun-core.service
systemctl enable pumpdotfun-db-processor.service
systemctl enable pumpdotfun-klines-engine.service
systemctl enable pumpdotfun-wss.service
systemctl enable pumpdotfun-api.service
systemctl enable pumpdotfun.target
echo -e "${GREEN}Services enabled${NC}"

echo ""
echo -e "${GREEN}Setup completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Ensure Rust and Cargo are installed for the $APP_USER user"
echo ""
echo "2. Copy your Vixen.toml configuration to $APP_DIR/core/Vixen.toml"
echo ""
echo "3. Update environment variables in service files if needed:"
echo "   - Database URL in db-processor and klines-engine services"
echo "   - Redis URL in db-processor, klines-engine, and wss services" 
echo "   - API ports in api and wss services"
echo ""
echo "4. Start the services:"
echo "   systemctl start pumpdotfun.target"
echo ""
echo -e "${YELLOW}Service management commands:${NC}"
echo "Start all services:    systemctl start pumpdotfun.target"
echo "Stop all services:     systemctl stop pumpdotfun.target"
echo "Restart all services:  systemctl restart pumpdotfun.target"
echo "Check status:          systemctl status pumpdotfun.target"
echo "View logs:             journalctl -u pumpdotfun-core -f"
echo ""
echo -e "${YELLOW}Individual service commands:${NC}"
echo "systemctl start pumpdotfun-core"
echo "systemctl start pumpdotfun-db-processor"
echo "systemctl start pumpdotfun-klines-engine"
echo "systemctl start pumpdotfun-wss"
echo "systemctl start pumpdotfun-api"
