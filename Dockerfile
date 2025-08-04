FROM debian:stable-slim

WORKDIR /app

# Install SSL certificates and required runtime dependencies for Rust applications
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the artifact from the local filesystem
COPY ./rewardo-search-api-latest ./rewardo-search-api-latest

# Copy the .env file
COPY ./.env ./.env

# Ensure the binary is executable
RUN chmod +x ./rewardo-search-api-latest

# Expose the port the server listens on
EXPOSE 8086

# Set the entry point to run the application with environment variables from .env
CMD ["./rewardo-search-api-latest"]