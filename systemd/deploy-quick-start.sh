#!/bin/bash

# Quick deployment script for PumpDotFun services
# This script provides a quick way to deploy and manage the services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_DIR="/opt/pumpdotfun"

echo -e "${GREEN}PumpDotFun Quick Deployment Script${NC}"
echo "=================================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}" 
   exit 1
fi

# Function to show service status
show_status() {
    echo -e "${BLUE}Service Status:${NC}"
    systemctl status pumpdotfun.target --no-pager || true
    echo ""
    echo -e "${BLUE}Individual Services:${NC}"
    for service in pumpdotfun-core pumpdotfun-db-processor pumpdotfun-klines-engine pumpdotfun-wss pumpdotfun-api; do
        status=$(systemctl is-active $service 2>/dev/null || echo "inactive")
        if [ "$status" = "active" ]; then
            echo -e "  $service: ${GREEN}$status${NC}"
        else
            echo -e "  $service: ${RED}$status${NC}"
        fi
    done
}

# Function to show logs
show_logs() {
    echo -e "${BLUE}Recent logs from all services:${NC}"
    journalctl -u pumpdotfun-core -u pumpdotfun-db-processor -u pumpdotfun-klines-engine -u pumpdotfun-wss -u pumpdotfun-api --since "10 minutes ago" --no-pager
}

# Check command line arguments
case "${1:-}" in
    "start")
        echo -e "${YELLOW}Starting all PumpDotFun services...${NC}"
        systemctl start pumpdotfun.target
        echo -e "${GREEN}Services started${NC}"
        show_status
        ;;
    "stop")
        echo -e "${YELLOW}Stopping all PumpDotFun services...${NC}"
        systemctl stop pumpdotfun.target
        echo -e "${GREEN}Services stopped${NC}"
        show_status
        ;;
    "restart")
        echo -e "${YELLOW}Restarting all PumpDotFun services...${NC}"
        systemctl restart pumpdotfun.target
        echo -e "${GREEN}Services restarted${NC}"
        show_status
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "follow")
        echo -e "${YELLOW}Following logs (Ctrl+C to stop)...${NC}"
        journalctl -u pumpdotfun-core -u pumpdotfun-db-processor -u pumpdotfun-klines-engine -u pumpdotfun-wss -u pumpdotfun-api -f
        ;;
    "update-config")
        if [ ! -f "core/Vixen.toml" ]; then
            echo -e "${RED}Vixen.toml not found in core directory${NC}"
            exit 1
        fi
        echo -e "${YELLOW}Updating Vixen.toml configuration...${NC}"
        cp core/Vixen.toml "$APP_DIR/core/"
        chown pumpdotfun:pumpdotfun "$APP_DIR/core/Vixen.toml"
        echo -e "${GREEN}Configuration updated${NC}"
        echo -e "${YELLOW}Restarting core service...${NC}"
        systemctl restart pumpdotfun-core
        ;;
    *)
        echo -e "${BLUE}Usage: $0 {start|stop|restart|status|logs|follow|update-config}${NC}"
        echo ""
        echo -e "${YELLOW}Commands:${NC}"
        echo "  start        - Start all services"
        echo "  stop         - Stop all services"
        echo "  restart      - Restart all services"
        echo "  status       - Show service status"
        echo "  logs         - Show recent logs"
        echo "  follow       - Follow logs in real-time"
        echo "  update-config - Update Vixen.toml and restart core service"
        echo ""
        show_status
        ;;
esac
