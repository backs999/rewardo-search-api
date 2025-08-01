# Stage 1: Build the application
# Using the official Rust image for building
FROM rust:latest AS builder

WORKDIR /app

# Copy the Cargo files for dependency resolution
COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src

COPY src/ src/

# Build the application
RUN cargo build --release

# Stage 2: Create the runtime image
# Using Debian slim which is:
# - Lightweight while still using glibc (compatible with standard Rust binaries)
# - Includes /bin/bash for shell access
# - Has package manager (apt) for installing additional dependencies
# - Suitable for running Rust applications
# - Regularly updated for security
FROM debian:stable-slim

WORKDIR /app

# Install SSL certificates and required runtime dependencies for Rust applications
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/rewardo-search-api .

# Ensure the binary is executable
RUN chmod +x ./rewardo-search-api

# Expose the port the server listens on
EXPOSE 8086

# Environment variable for controlling log levels
# Example values: debug, info, warn, error
# Default is 'info' if not specified
ENV RUST_LOG=info

# Set the entry point to run the application
CMD ["./rewardo-search-api"]