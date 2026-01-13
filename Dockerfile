
FROM rust:1.92.0-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    libsoapysdr-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml ./

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --bin api --features api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    libsoapysdr0.8 \
    curl \
    iptables \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*

# Install Tailscale
RUN curl -fsSL https://tailscale.com/install.sh | sh

WORKDIR /app

COPY --from=builder /app/target/release/api /usr/local/bin/api

# Create startup script
RUN echo '#!/bin/sh\n\
tailscaled --tun=userspace-networking --socks5-server=localhost:1055 &\n\
tailscale up --authkey=${TAILSCALE_AUTHKEY} --hostname=sdr-db-api\n\
exec api\n\
' > /app/start.sh && chmod +x /app/start.sh

ENV RUST_LOG=info,api=debug
ENV API_PORT=3000

EXPOSE 3000

CMD ["/app/start.sh"]
