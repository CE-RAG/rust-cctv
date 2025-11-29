# Stage 1: Build
FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app

# Install musl-tools for static compilation
RUN apt-get update \
    && apt-get install -y \
        musl-tools \
        pkg-config \
        libssl-dev \
        build-essential \
    && rm -rf /var/lib/apt/lists/*

# Add the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy only Cargo.toml and Cargo.lock first to leverage Docker cache
# This layer changes less often than source code
COPY Cargo.toml Cargo.lock ./

# Create dummy `lib.rs` file to prepare build dependencies only
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only. This layer is highly cacheable.
# If Cargo.toml and Cargo.lock haven't changed, this step will be skipped.
# RUN cargo fetch --locked --target x86_64-unknown-linux-gnu
RUN cargo fetch --locked
# Copy all source code
COPY src ./src

# Build the release binary with musl target
# --release for optimizations and smaller size
# --locked to ensure reproducible builds based on Cargo.lock
# --target for static linking with musl libc
RUN CARGO_INCREMENTAL=0 \
    OPENSSL_STATIC=1 \
    RUSTFLAGS="-C strip=debuginfo -C target-feature=+aes,+sse2,+ssse3" \
    cargo build --release --locked --target x86_64-unknown-linux-musl

# Stage 2: Runtime
FROM scratch

# Copy only the compiled binary from the builder stage
# COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/rust-cctv .
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rust-cctv /rust-cctv

# Expose application port
EXPOSE 8080

# Define the command to run your application
CMD ["/rust-cctv"]
