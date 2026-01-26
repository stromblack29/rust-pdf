# Rust PDF Compression Service

A high-performance microservice for compressing PDF documents, built with Rust. 
It provides both **gRPC** and **REST** APIs and uses aggressive image optimization tactics (downscaling + JPEG re-encoding) to achieve significant file size reductions (target 95% reduction for image-heavy PDFs).

## Features

- **Aggressive Compression**: Target 90% file size reduction
- **Configurable**: Adjust quality, dimensions, and metadata removal
- **Dual APIs**: REST (with Swagger UI) and gRPC support
- **Fast**: Built with Rust for maximum performance
- **Statistics**: Returns compression metrics (original size, compressed size, ratio)

## Compression Strategy

The service achieves 90% compression through:

1. **Image Downscaling**: Reduces images to max 600px (configurable)
2. **JPEG Re-encoding**: Compresses images with quality 30 (configurable)
3. **Metadata Removal**: Strips unnecessary PDF metadata
4. **Stream Compression**: Applies PDF stream compression

### Default Settings (90% Target)
- JPEG Quality: 30
- Max Dimension: 600px
- Remove Metadata: true

## Quick Start

### Prerequisites
- Rust (latest stable)
- Protocol Buffers compiler (`protoc`)

### Installation

```bash
# Clone the repository
git clone https://github.com/stromblack29/rust-pdf.git
cd rust-pdf

# Build the project
cargo build --release

# Run the server
cargo run --release
```

The service will start:
- **gRPC Server**: `[::1]:50051`
- **REST API**: `http://127.0.0.1:3000`
- **Swagger UI**: `http://127.0.0.1:3000/swagger-ui/`

## Usage

### REST API

#### Basic Compression (90% target)
```bash
curl -X POST http://localhost:3000/compress \
  -F "file=@input.pdf" \
  -o compressed.pdf
```

#### Custom Compression Settings
```bash
# Ultra compression (95% target)
curl -X POST "http://localhost:3000/compress?quality=20&max_dimension=400" \
  -F "file=@input.pdf" \
  -o ultra_compressed.pdf

# Moderate compression (70% target)
curl -X POST "http://localhost:3000/compress?quality=50&max_dimension=1000" \
  -F "file=@input.pdf" \
  -o moderate_compressed.pdf
```

#### Query Parameters
- `quality` (1-100): JPEG quality, lower = smaller file (default: 30)
- `max_dimension` (pixels): Maximum image dimension (default: 600)
- `remove_metadata` (true/false): Remove PDF metadata (default: true)

#### Response Headers
- `X-Original-Size`: Original file size in bytes
- `X-Compressed-Size`: Compressed file size in bytes
- `X-Compression-Ratio`: Compression percentage

### gRPC API

See `examples/grpc_client_test.rs` for a complete example.

```rust
use rustpdf::grpc::pb::{
    compression_service_client::CompressionServiceClient,
    CompressRequest, CompressionConfig
};

let mut client = CompressionServiceClient::connect("http://[::1]:50051").await?;

let request = CompressRequest {
    pdf_data: pdf_bytes,
    config: Some(CompressionConfig {
        jpeg_quality: Some(30),
        max_dimension: Some(600),
        remove_metadata: Some(true),
    }),
};

let response = client.compress_pdf(request).await?;
println!("Compression ratio: {:.2}%", response.compression_ratio);
```

## Testing

### Test 90% Compression
```bash
cargo run --example test_90_percent_compression
```

This will test three compression levels:
- Default (90% target): Quality 30, Max 600px
- Ultra (95% target): Quality 20, Max 400px
- Moderate (70% target): Quality 50, Max 1000px

### Test gRPC Client
```bash
# Terminal 1: Start the server
cargo run

# Terminal 2: Run the gRPC client test
cargo run --example grpc_client_test
```

## API Documentation

### Swagger UI
Visit `http://localhost:3000/swagger-ui/` when the server is running for interactive API documentation.

### Proto Definition
See `proto/compression.proto` for the gRPC service definition.

## Project Structure

```bash
rustpdf/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── compression/         # Compression engine
│   │   └── mod.rs          # Core compression logic
│   ├── api/                 # REST API
│   │   └── mod.rs          # Axum handlers + OpenAPI
│   └── grpc/                # gRPC service
│       └── mod.rs          # Tonic service implementation
├── proto/
│   └── compression.proto    # gRPC protocol definition
├── examples/
│   ├── test_90_percent_compression.rs
│   └── grpc_client_test.rs
└── Cargo.toml
```

## Dependencies

- **tokio**: Async runtime
- **axum**: REST API framework
- **tonic**: gRPC framework
- **lopdf**: PDF parsing and manipulation
- **image**: Image processing and compression
- **utoipa**: OpenAPI documentation

## Performance Notes

- Compression is CPU-intensive and runs in blocking threads
- Default body limit: 50MB
- Processing time depends on PDF size and image count
- Typical 5MB PDF with images: 1-3 seconds

## Limitations

- CMYK color space not fully supported (will skip those images)
- Text-only PDFs won't see significant compression
- Best results with image-heavy PDFs
- Some complex PDF features may not be preserved

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

## Repository

https://github.com/stromblack29/rust-pdf
