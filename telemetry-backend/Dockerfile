# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/telemetry-backend /app/telemetry-backend

# Create data directory
RUN mkdir -p /data

ENV DATABASE_URL=sqlite:///data/telemetry.db
ENV PORT=8080

EXPOSE 8080

VOLUME ["/data"]

CMD ["/app/telemetry-backend"]
