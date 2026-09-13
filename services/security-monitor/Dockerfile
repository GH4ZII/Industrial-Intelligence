FROM rust:1-bookworm AS builder
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libpcap-dev libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libpcap0.8 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/security-monitor /usr/local/bin/security-monitor
CMD ["security-monitor", "--demo"]
