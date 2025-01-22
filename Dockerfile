# Build stage
FROM rust:1.84.0-slim-bookworm as builder

# Combine all build dependencies installation and cleanup in one layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    wget \
    g++ \
    libstdc++-11-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /usr/src/app

# Copy only necessary files for build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Create a dummy main.rs to cache dependencies
RUN mkdir -p models && \
    wget -O models/silueta.onnx https://github.com/danielgatis/rembg/releases/download/v0.0.0/silueta.onnx && \
    cargo build --release --locked && \
    rm -rf target/release/deps/rembg_cpu_rust*

# Build the actual binary
COPY . .
RUN cargo build --release --locked && \
    strip target/release/rembg-cpu-rust

# Runtime stage
FROM debian:bookworm-slim

# Combine all runtime dependencies installation and cleanup in one layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libgomp1 \
    ca-certificates \
    libstdc++6 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# Copy only the necessary files from builder
COPY --from=builder /usr/src/app/target/release/rembg-cpu-rust /app/
COPY --from=builder /usr/src/app/models/silueta.onnx /app/models/

# Create a non-root user and set permissions
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app
USER appuser

# Configure the application
EXPOSE 80
ENV RUST_LOG=info \
    PORT=80

# Run the binary
CMD ["./rembg-cpu-rust"]
