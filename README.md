# rembg-cpu-rust

A fast background removal service written in Rust, using the U2Net model and ONNX Runtime for CPU inference.

## Features

- Fast background removal using U2Net model
- CPU-based inference using ONNX Runtime
- RESTful API endpoint for image processing
- Docker support for easy deployment
- Handles images of any size while maintaining aspect ratio
- Returns PNG images with transparency

## Prerequisites

- Rust 1.70 or higher
- Docker (optional, for containerized deployment)
- wget or curl (for downloading the model)

## Quick Start

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rembg-cpu-rust
cd rembg-cpu-rust
```

2. Download the U2Net model:
```bash
./scripts/download-model.sh
```

3. Build and run locally:
```bash
cargo run --release
```

The server will start at `http://localhost:8000`

## Docker Deployment

Build and run using Docker:

```bash
docker build -t rembg-cpu-rust .
docker run -p 8000:8000 rembg-cpu-rust
```

## API Usage

Remove background from an image:

```bash
curl -X POST -F "image=@/path/to/your/image.jpg" http://localhost:8000/api/rem-bg -o output.png
```

Or use the provided `api.http` file with REST Client extensions in VS Code/IntelliJ.

## Configuration

- Default port: 8000
- Max file size: 10MB
- Model path: `models/u2net.onnx`

## Project Structure

```
├── src/
│   ├── application/      # Application logic
│   ├── domain/          # Domain models and errors
│   ├── infrastructure/  # Server setup and configuration
│   └── presentation/    # API handlers
├── models/             # Model storage
├── scripts/           # Utility scripts
└── Dockerfile        # Docker configuration
```

## Performance Considerations

- Images are processed at 320x320 resolution for inference
- Original image dimensions are preserved in the output
- CPU-optimized for broad compatibility

## Error Handling

The service handles various error cases:
- Invalid image format
- Missing image file
- Model loading errors
- Processing errors

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [U2Net](https://github.com/xuebinqin/U-2-Net) - Original U2Net model
- [ONNX Runtime](https://onnxruntime.ai/) - Machine learning inference engine
- [Axum](https://github.com/tokio-rs/axum) - Web framework