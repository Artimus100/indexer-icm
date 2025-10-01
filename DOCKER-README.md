# Docker Deployment Guide

This guide covers deploying the indexer-icm services using Docker.

## Services

The project consists of 5 main Rust services:

1. **Core** (`core/`) - Main indexer service with Vixen configuration
2. **API** (`api/`) - REST API service  
3. **WebSocket** (`wss/`) - WebSocket server
4. **DB Processor** (`db-processor/`) - Database processing service
5. **KLines Engine** (`klines-engine/`) - KLines data processing

## Prerequisites

- Docker and Docker Compose installed
- Access to DigitalOcean managed Redis and PostgreSQL instances

## Environment Variables

The services are configured to use DigitalOcean managed databases:

- **Redis URL**: `rediss://default:AVNS_BkPlgZEK_rekF0Dgdc7@db-valkey-nyc3-58554-do-user-25396063-0.l.db.ondigitalocean.com:25061`
- **Database URL**: `postgresql://doadmin:AVNS_t3Xgg3yybcDW8r275h6@db-postgresql-nyc3-06292-do-user-25396063-0.k.db.ondigitalocean.com:25060/icm?sslmode=require`

## Quick Deployment

### Option 1: Docker Compose (Recommended)

```bash
# Build and run all services
docker-compose up -d

# Build and run only specific services
docker-compose up -d core api

# View logs
docker-compose logs -f core
```

### Option 2: Individual Docker Commands

```bash
# Run the build script
./deploy-docker.sh

# Or build manually
docker build -f core/Dockerfile -t indexer-core:latest .
docker build -f api/Dockerfile -t indexer-api:latest .
docker build -f wss/Dockerfile -t indexer-wss:latest .
docker build -f db-processor/Dockerfile -t indexer-db-processor:latest .
docker build -f klines-engine/Dockerfile -t indexer-klines-engine:latest .

# Run containers
docker run -d --name indexer-core -p 8080:8080 indexer-core:latest
docker run -d --name indexer-api -p 3000:3000 indexer-api:latest
docker run -d --name indexer-wss -p 8000:8000 indexer-wss:latest
docker run -d --name indexer-db-processor indexer-db-processor:latest
docker run -d --name indexer-klines-engine indexer-klines-engine:latest
```

## Service Ports

- **Core**: 8080
- **API**: 3000  
- **WebSocket**: 8000
- **DB Processor**: No external port (internal service)
- **KLines Engine**: No external port (internal service)

## Configuration

### Core Service

The core service requires a `Vixen.toml` configuration file. A sample is provided at the root of the project. Modify it according to your needs:

```toml
[log]
level = "info"

[redis]
url = "rediss://default:AVNS_BkPlgZEK_rekF0Dgdc7@db-valkey-nyc3-58554-do-user-25396063-0.l.db.ondigitalocean.com:25061"

[grpc]
endpoint = "https://mainnet.helius-rpc.com"

[filters]
# Add your filters here
```

## Local Development

To run with local PostgreSQL and Redis instead of managed services:

```bash
# Start local databases
docker-compose --profile local up postgres redis -d

# Update environment variables in docker-compose.yml to point to local services
# Then start your services
docker-compose up core api wss db-processor klines-engine
```

## Monitoring

### Health Checks

```bash
# Check service status
docker-compose ps

# View logs
docker-compose logs -f [service-name]

# Check individual container health
docker logs indexer-core
docker logs indexer-api
```

### Service Endpoints

- API Health: `http://localhost:3000/health` (if health endpoint exists)
- WebSocket: `ws://localhost:8000`
- Core metrics: Check logs for connection status

## Troubleshooting

### Build Issues

```bash
# Clean build
docker-compose build --no-cache

# Check individual service builds
docker build -f core/Dockerfile -t test-core .
```

### Connection Issues

1. Verify DigitalOcean database URLs are accessible
2. Check firewall settings for Redis/PostgreSQL ports
3. Validate SSL certificates for `rediss://` connections

### Resource Issues

```bash
# Check resource usage
docker stats

# Scale services
docker-compose up -d --scale db-processor=2
```

## Security Notes

- Database credentials are embedded in the configuration
- Use Docker secrets or environment files for production
- Ensure proper network isolation between services
- Keep Docker images updated

## Production Considerations

1. Use proper secrets management
2. Set up log aggregation
3. Configure monitoring and alerting
4. Implement proper backup strategies
5. Use multi-stage builds for smaller images
6. Consider using Kubernetes for orchestration