# DB Processor service Dockerfile
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

# Build the DB processor service
RUN cargo build --release --bin db-processor

# Runtime stage - use same base as builder for GLIBC compatibility
FROM rustlang/rust:nightly-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/db-processor /usr/local/bin/db-processor

# Set environment variables
ENV REDIS_URL=rediss://default:AVNS_BkPlgZEK_rekF0Dgdc7@db-valkey-nyc3-58554-do-user-25396063-0.l.db.ondigitalocean.com:25061
ENV DATABASE_URL=postgresql://doadmin:AVNS_t3Xgg3yybcDW8r275h6@db-postgresql-nyc3-06292-do-user-25396063-0.k.db.ondigitalocean.com:25060/icm?sslmode=require
ENV RUST_LOG=info

CMD ["db-processor"]