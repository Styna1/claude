# ── Stage 1: Build ──
FROM rust:1.83-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/

# Build release binary
RUN cargo build --release

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/agario-clone .

# Copy static frontend files
COPY static/ static/

# Create data directory for SQLite
RUN mkdir -p data

EXPOSE 3000

ENV RUST_LOG=info

CMD ["./agario-clone"]
