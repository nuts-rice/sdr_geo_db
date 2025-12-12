# Multi-stage build for SDR DB API server
# Stage 1: Build the Rust binary
FROM rust:1.92-slim as builder

# Install build dependencies for PostgreSQL, Diesel, and SoapySDR
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libsoapysdr-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the API binary with release optimizations
RUN cargo build --release --bin api --features api

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    libsoapysdr0.8 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/api /usr/local/bin/api

# Set environment variables
ENV RUST_LOG=info,api=debug
ENV API_PORT=3000

# Expose the API port
EXPOSE 3000

# Run the API server
CMD ["api"]
