# WebSocket service Dockerfile
FROM rustlang/rust:nightly-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    build-essential \
    cmake \
    git \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace Cargo.toml first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members for dependency resolution
COPY core/ ./core/
COPY api/ ./api/
COPY wss/ ./wss/
COPY db-processor/ ./db-processor/
COPY klines-engine/ ./klines-engine/
COPY store/ ./store/

# Build the WSS service
RUN cargo build --release --bin wss

# Runtime stage - use same base as builder for GLIBC compatibility
FROM rustlang/rust:nightly-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/wss /usr/local/bin/wss

# Set environment variables
ENV REDIS_URL=rediss://default:AVNS_BkPlgZEK_rekF0Dgdc7@db-valkey-nyc3-58554-do-user-25396063-0.l.db.ondigitalocean.com:25061
ENV RUST_LOG=info

EXPOSE 8000

CMD ["wss"]