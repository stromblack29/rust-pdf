# Walkthrough - Rust PDF Compression Service

The service is fully implemented in Rust using `tonic` (gRPC), `axum` (REST), `lopdf`, and `image` crates. It has been successfully compiled and verified.

## Features
- **gRPC API**: `CompressPdf` method on port `50051` with configurable compression settings.
- **REST API**: `/compress` endpoint on port `3000` with query parameters for configuration.
- **Swagger UI**: Accessible at `http://127.0.0.1:3000/swagger-ui/`.
- **90% Compression Target**: Aggressive compression with quality 30, max 600px dimensions.
- **Configurable Settings**: Adjust JPEG quality, max dimensions, and metadata removal.
- **Compression Statistics**: Returns original size, compressed size, and compression ratio.

## How to Run
1. Ensure Rust and Protoc are in your PATH (installed via `winget` during setup).
2. Run the server:
   ```powershell
   cargo run
   ```

## Verifying the Service

### 1. REST API & Swagger
Open your browser to:
[http://localhost:3000/swagger-ui/](http://localhost:3000/swagger-ui/)

You can use the "Try it out" feature to upload a PDF.

### 2. gRPC API
The proto definition is located at `proto/compression.proto`. You can use a client like `grpcurl`:
```bash
grpcurl -plaintext -proto proto/compression.proto -d '{"pdf_data": "..."}' localhost:50051 compression.CompressionService/CompressPdf
```
*(Note: Sending binary data via command line JSON is tricky; programmatic access is recommended).*

## Testing Compression

### Test 90% Compression Target
Run the comprehensive compression test with multiple quality levels:
```powershell
cargo run --example test_90_percent_compression
```

This will test three compression profiles:
- **Default (90% target)**: Quality 30, Max 600px
- **Ultra (95% target)**: Quality 20, Max 400px  
- **Moderate (70% target)**: Quality 50, Max 1000px

Output files will be saved in the `examples/` directory with compression statistics.

### Test gRPC Client
Test the gRPC API with a sample PDF:

1. Start the server in one terminal:
   ```powershell
   cargo run
   ```

2. Run the gRPC client test in another terminal:
   ```powershell
   cargo run --example grpc_client_test
   ```

This will compress a PDF via gRPC and display compression statistics.

### REST API Testing via Swagger UI
1. Open `http://localhost:3000/swagger-ui/`
2. Navigate to the `/compress` endpoint
3. Click "Try it out"
4. Upload a PDF file
5. Optionally adjust query parameters:
   - `quality`: 1-100 (default: 30)
   - `max_dimension`: pixels (default: 600)
   - `remove_metadata`: true/false (default: true)
6. Execute and download the compressed PDF

### REST API Testing via cURL
```powershell
# Default 90% compression
curl -X POST http://localhost:3000/compress -F "file=@input.pdf" -o compressed.pdf

# Custom settings - ultra compression
curl -X POST "http://localhost:3000/compress?quality=20&max_dimension=400" -F "file=@input.pdf" -o ultra.pdf

# Custom settings - moderate compression
curl -X POST "http://localhost:3000/compress?quality=50&max_dimension=1000" -F "file=@input.pdf" -o moderate.pdf
```

> [!NOTE]
> Real-world PDFs with high-resolution images will see significant compression. Text-only PDFs won't compress much. Best results are achieved with image-heavy PDFs (photos, scans, etc.).

## Code Structure
- `src/lib.rs`: Exports modules.
- `src/main.rs`: Entry point spawning gRPC and Axum servers.
- `src/compression/mod.rs`: Core compression logic with configurable settings:
  - `CompressionConfig`: Configuration struct for quality, dimensions, metadata
  - `compress_pdf()`: Default compression (90% target)
  - `compress_pdf_with_config()`: Custom compression settings
  - `remove_metadata()`: Strips PDF metadata for size reduction
  - `process_image_object()`: Image extraction, resizing, and JPEG re-encoding
- `src/grpc/mod.rs`: gRPC service implementation with compression statistics.
- `src/api/mod.rs`: REST handlers with query parameters and OpenAPI docs.
- `proto/compression.proto`: gRPC protocol definition with compression config.

## Compression Configuration

### CompressionConfig Structure
```rust
pub struct CompressionConfig {
    pub jpeg_quality: u8,        // 1-100, lower = smaller file
    pub max_dimension: u32,      // Max pixels for images
    pub remove_metadata: bool,   // Strip PDF metadata
}
```

### Default Settings (90% Target)
- JPEG Quality: 30
- Max Dimension: 600px
- Remove Metadata: true

### Recommended Settings by Use Case

**Web Display (90-95% reduction)**
- Quality: 20-30
- Max Dimension: 400-600px
- Remove Metadata: true

**Print Quality (60-70% reduction)**
- Quality: 60-70
- Max Dimension: 1200-1500px
- Remove Metadata: true

**Archive (40-50% reduction)**
- Quality: 80-85
- Max Dimension: 2000px
- Remove Metadata: false

## Git Repository
Initialized and linked to `https://github.com/stromblack29/rust-pdf.git`.
- `.gitignore` configured.
- Initial commit created.
