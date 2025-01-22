# Build stage
FROM rust:1.84.0 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Download the model file
RUN mkdir -p models && \
    wget -O models/u2net.onnx https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx

# Build dependencies - this is the caching Docker layer!
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

# Install OpenSSL, ONNX Runtime, and other dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    libgomp1 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -ms /bin/bash appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/rembg-cpu-rust /app/rembg-cpu-rust
COPY --from=builder /usr/src/app/models/u2net.onnx /app/models/u2net.onnx

# Use the non-root user
RUN chown -R appuser:appuser /app
USER appuser

# Expose port 80
EXPOSE 80

# Set environment variables
ENV RUST_LOG=info
ENV PORT=80
ENV LD_LIBRARY_PATH=/usr/lib

# Run the binary
CMD ["./rembg-cpu-rust"]
