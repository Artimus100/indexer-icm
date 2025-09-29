# PumpDotFun Systemd Configuration

This directory contains systemd service files and scripts to manage the PumpDotFun application services on a Linux server.

## Services

The following services are configured:

1. **pumpdotfun-core** - Core service that handles the main business logic
2. **pumpdotfun-db-processor** - Database processor service
3. **pumpdotfun-klines-engine** - K-lines engine service
4. **pumpdotfun-wss** - WebSocket service
5. **pumpdotfun-api** - REST API service

## Files

- `pumpdotfun-*.service` - Individual systemd service files
- `pumpdotfun.target` - Target file to manage all services together
- `setup.sh` - Script to install and configure systemd services
- `deploy.sh` - Script to build and deploy the application
- `.env.example` - Example environment configuration file

## Prerequisites

Before setting up the services, ensure you have:

1. **PostgreSQL** installed and running
2. **Redis** installed and running
3. Rust toolchain installed
4. Root access to the server

## Installation

1. **Run the setup script as root:**
   ```bash
   sudo ./setup.sh
   ```

2. **Build your application:**
   ```bash
   cargo build --release
   ```

3. **Deploy the application:**
   ```bash
   sudo ./deploy.sh
   ```

4. **Configure environment variables:**
   ```bash
   sudo cp .env.example /opt/pumpdotfun/.env
   sudo nano /opt/pumpdotfun/.env
   ```

## Service Management

### Start all services:
```bash
sudo systemctl start pumpdotfun.target
```

### Stop all services:
```bash
sudo systemctl stop pumpdotfun.target
```

### Restart all services:
```bash
sudo systemctl restart pumpdotfun.target
```

### Check status of all services:
```bash
sudo systemctl status pumpdotfun.target
```

### Individual service management:
```bash
sudo systemctl start pumpdotfun-core
sudo systemctl status pumpdotfun-api
sudo systemctl restart pumpdotfun-wss
```

## Logging

View logs for all services:
```bash
sudo journalctl -u pumpdotfun-*
```

View logs for a specific service:
```bash
sudo journalctl -u pumpdotfun-core -f
```

View logs with timestamps:
```bash
sudo journalctl -u pumpdotfun-api --since "1 hour ago"
```

## Configuration

### Service Dependencies

The services are configured with the following dependency order:
1. `pumpdotfun-core` - Starts first (depends on network)
2. `pumpdotfun-db-processor` - Depends on core, redis, postgresql
3. `pumpdotfun-klines-engine` - Depends on core, redis, postgresql
4. `pumpdotfun-wss` - Depends on core, redis
5. `pumpdotfun-api` - Depends on core, postgresql

### Environment Variables

Each service can be configured through environment variables. Common variables include:

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `RUST_LOG` - Logging level (debug, info, warn, error)
- `API_PORT` - Port for the API service
- `WSS_PORT` - Port for the WebSocket service

### Security

The services are configured with security hardening:

- Run as non-root user (`pumpdotfun`)
- Private temporary directories
- Protected system directories
- Limited file access
- Resource limits (memory, file descriptors)

## Troubleshooting

### Service won't start:
1. Check the service status: `sudo systemctl status pumpdotfun-core`
2. Check the logs: `sudo journalctl -u pumpdotfun-core`
3. Verify dependencies are running (PostgreSQL, Redis)
4. Check file permissions in `/opt/pumpdotfun`

### High memory usage:
- Adjust `MemoryMax` in service files
- Monitor with: `sudo systemctl status pumpdotfun-*`

### Permission issues:
```bash
sudo chown -R pumpdotfun:pumpdotfun /opt/pumpdotfun
sudo chmod +x /opt/pumpdotfun/target/release/*
```

### Database connection issues:
1. Verify PostgreSQL is running: `sudo systemctl status postgresql`
2. Check database URL in `/opt/pumpdotfun/.env`
3. Test connection manually

### Redis connection issues:
1. Verify Redis is running: `sudo systemctl status redis`
2. Check Redis URL in `/opt/pumpdotfun/.env`
3. Test connection: `redis-cli ping`

## Directory Structure

```
/opt/pumpdotfun/
├── target/release/          # Compiled binaries
│   ├── core
│   ├── api
│   ├── db-processor
│   ├── klines-engine
│   └── wss
├── .env                     # Environment configuration
└── Vixen.toml              # Core service configuration
```

## Monitoring

You can monitor the services using:

```bash
# Check if all services are active
sudo systemctl is-active pumpdotfun-*

# Monitor resource usage
sudo systemctl status pumpdotfun.target

# Watch logs in real-time
sudo journalctl -u pumpdotfun-* -f
```
