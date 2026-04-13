# Stage 1: Build
FROM rust:1.85-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY migration/Cargo.toml ./migration/

# Create dummy main.rs to build dependencies
RUN mkdir -p api/src migration/src && \
    echo "fn main() {}" > api/src/main.rs && \
    echo "pub fn dummy() {}" > api/src/lib.rs && \
    echo "pub fn dummy() {}" > migration/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf api/src migration/src

# Copy actual source code
COPY api/src ./api/src
COPY migration/src ./migration/src

# Build the application
RUN touch api/src/main.rs api/src/lib.rs migration/src/lib.rs && \
    cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/api /usr/local/bin/taskflow

# Copy seed.sql for manual seeding
COPY seed.sql /app/seed.sql

# Create non-root user
RUN addgroup -S taskflow && adduser -S taskflow -G taskflow
USER taskflow

EXPOSE 8080

CMD ["taskflow"]
