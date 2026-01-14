# Multi-stage Dockerfile for KS Forward Video Processor

# Stage 1: Build
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Runtime
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    && update-ca-certificates

# Create non-root user
RUN addgroup -S appuser && adduser -S appuser -G appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/schRust /app/schRust

# Create cache directory
RUN mkdir -p /app/transcript_cache && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD schRust --health || exit 1

ENTRYPOINT ["/app/schRust"]
CMD ["run"]
