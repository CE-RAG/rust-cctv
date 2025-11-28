# Stage 1: Build
FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install SSL libs (needed for HTTP requests)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/rust-cctv .

# Expose port
EXPOSE 8080

# Run the app
CMD ["./rust-cctv"]