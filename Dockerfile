FROM rust:1.78-slim AS builder

WORKDIR /app

# System deps for openssl-sys (used by actix-web openssl feature)
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# Build application
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Runtime OpenSSL + certs for HTTPS calls
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/short-link-backend /usr/local/bin/short-link-backend
COPY config.yaml ./config.yaml

ENV RUST_LOG=info

CMD ["short-link-backend"]
