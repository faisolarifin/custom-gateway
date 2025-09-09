# Multi-stage build untuk Rust webhook-gateway
FROM rust:1.89-alpine AS builder
WORKDIR /app

# Install dependencies untuk compile Rust di Alpine
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Copy Cargo.toml dan Cargo.lock untuk cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy main untuk cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm src/main.rs

# Copy source code
COPY src ./src

# Build binary
RUN cargo build --release

# Stage 2: Minimal runtime image
FROM alpine:3.18
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata libgcc
ENV TZ=Asia/Jakarta

# Copy binary dari builder stage
COPY --from=builder /app/target/release/webhook-gateway ./webhook-gateway

# Copy config example (akan di-override saat deployment)
COPY config.yaml.example ./config.yaml

# Create log directory
RUN mkdir -p log

# Expose port
EXPOSE 8080

# Entry point
CMD ["./webhook-gateway"]
