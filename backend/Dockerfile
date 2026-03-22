FROM rust:1.88-slim AS builder

WORKDIR /app

ENV OPENSSL_NO_VENDOR=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        pkg-config \
        libssl-dev \
        perl \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt
COPY . .

ARG ENABLE_DRIFT=false
RUN if [ "$ENABLE_DRIFT" = "true" ]; then \
        rustup install 1.76.0-x86_64-unknown-linux-gnu \
        && cargo build --release --features drift --bin backend-cli; \
    else \
        cargo build --release --bin backend-cli; \
    fi

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
