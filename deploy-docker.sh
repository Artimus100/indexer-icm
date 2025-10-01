#!/bin/bash

# Deployment script for indexer-icm services

set -e

echo "🚀 Building and deploying indexer-icm services..."

# Function to build individual service
build_service() {
    local service=$1
    echo "📦 Building $service service..."
    docker build -f ${service}/Dockerfile -t indexer-${service}:latest .
}

# Function to run individual service
run_service() {
    local service=$1
    echo "🏃 Running $service service..."
    docker run -d --name indexer-${service} \
        --restart unless-stopped \
        -p ${2}:${2} \
        indexer-${service}:latest
}

# Build all services
echo "Building all services..."
build_service core
build_service api  
build_service wss
build_service db-processor
build_service klines-engine

echo "✅ All services built successfully!"

# Option to run services individually or with docker-compose
echo ""
echo "To run services:"
echo "1. Use docker-compose: docker-compose up -d"
echo "2. Or run individual containers:"
echo "   - Core: docker run -d --name indexer-core -p 8080:8080 indexer-core:latest"
echo "   - API: docker run -d --name indexer-api -p 3000:3000 indexer-api:latest"
echo "   - WSS: docker run -d --name indexer-wss -p 8000:8000 indexer-wss:latest"
echo "   - DB Processor: docker run -d --name indexer-db-processor indexer-db-processor:latest"
echo "   - KLines Engine: docker run -d --name indexer-klines-engine indexer-klines-engine:latest"

echo ""
echo "🎉 Deployment ready!"