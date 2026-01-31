# Multi-stage build for minimal image size
FROM rust:alpine3.23 AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static perl make

# Create a new empty project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./
COPY build.rs ./

# Copy source code
COPY src ./src

# Build for release with static linking
RUN cargo build --release

# Runtime stage
FROM alpine:3.23.3

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Create directory for serving files and custom index
RUN mkdir -p /srv /index && chown appuser:appuser /srv /index

WORKDIR /srv

# Copy the binary from builder
COPY --from=builder /app/target/release/scratch-server /usr/local/bin/scratch-server

# Switch to non-root user
USER appuser

# Environment variables (defaults)
ENV THREADS=12 \
    IP=0.0.0.0 \
    SILENT=false \
    CORS=false \
    COMPRESSION=false

# Expose the default port
EXPOSE 7878

# Entry point script that converts env vars to CLI args
ENTRYPOINT ["/bin/sh", "-c", "\
    ARGS=\"--port 7878 --threads ${THREADS} --ip ${IP}\"; \
    [ \"${SILENT}\" = \"true\" ] && ARGS=\"${ARGS} --silent\"; \
    [ \"${CORS}\" = \"true\" ] && ARGS=\"${ARGS} --cors\"; \
    [ \"${COMPRESSION}\" = \"true\" ] && ARGS=\"${ARGS} --compression\"; \
    [ -n \"${AUTH}\" ] && ARGS=\"${ARGS} --auth ${AUTH}\"; \
    [ -n \"${CERT}\" ] && ARGS=\"${ARGS} --cert /certs/${CERT}\"; \
    [ -n \"${CERT_PASS}\" ] && ARGS=\"${ARGS} --certpass ${CERT_PASS}\"; \
    [ -f /index/index.html ] && ARGS=\"${ARGS} --index /index/index.html\"; \
    exec scratch-server ${ARGS}"]
