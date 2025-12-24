FROM rust:1.92-slim as builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libsoapysdr-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --bin api --features api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    libsoapysdr0.8 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api /usr/local/bin/api

ENV RUST_LOG=info,api=debug
ENV API_PORT=3000

EXPOSE 3000

CMD ["api"]
