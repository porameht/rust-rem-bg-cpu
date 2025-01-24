FROM rust:1.84.0-slim-bookworm as builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    wget \
    g++ \
    libstdc++-11-dev \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN mkdir -p models && \
    wget -O models/silueta.onnx https://github.com/danielgatis/rembg/releases/download/v0.0.0/silueta.onnx && \
    cargo build --release --locked && \
    rm -rf target/release/deps/rembg_cpu_rust*

COPY . .
RUN cargo build --release --locked && \
    strip target/release/rembg-cpu-rust

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libgomp1 \
    ca-certificates \
    libstdc++6 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/rembg-cpu-rust /app/
COPY --from=builder /usr/src/app/models/silueta.onnx /app/models/

RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 8000
ENV RUST_LOG=info \
    PORT=8000

CMD ["./rembg-cpu-rust"]
