FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin backend-cli

FROM debian:bookworm-slim AS runner

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend-cli /usr/local/bin/backend-cli
COPY --from=builder /app/config ./config

ENV RUST_LOG=info \
    PORT=5150

EXPOSE 5150

CMD ["backend-cli", "start", "-b", "0.0.0.0"]
