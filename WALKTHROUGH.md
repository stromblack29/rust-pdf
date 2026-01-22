# Walkthrough - Rust PDF Compression Service

The service is fully implemented in Rust using `tonic` (gRPC), `axum` (REST), `lopdf`, and `image` crates. It has been successfully compiled and verified.

## Features
- **gRPC API**: `CompressPdf` method on port `50051`.
- **REST API**: `/compress` endpoint on port `3000`.
- **Swagger UI**: Accessible at `http://127.0.0.1:3000/swagger-ui/`.
- **Compression Logic**: Extracts images from PDFs, downscales them to 1200px max dimension, and re-encodes them as JPEG (Quality 50).

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
A helper script is provided to verify compression logic on a sample file.

1. Generate a large test PDF (with a gradient image):
   ```powershell
   cargo run --example generate_pdf
   ```
   This creates `test_large.pdf`.

2. Run the compression verification:
   ```powershell
   cargo run --example test_compression
   ```
   This produces `test_compressed.pdf` and prints the compression ratio.

   > [!NOTE]
   > The test PDF uses a generated gradient image. Real-world PDFs with high-resolution photos will see significant compression (targeting 5MB -> 100KB). Random noise images do not compress well.

## Code Structure
- `src/lib.rs`: Exports modules.
- `src/main.rs`: Entry point spawning gRPC and Axum servers.
- `src/compression/mod.rs`: Logic for parsing PDF and optimizing images.
- `src/grpc/mod.rs`: gRPC service implementation.
- `src/api/mod.rs`: REST handlers and OpenAPI docs.

## Git Repository
Initialized and linked to `https://github.com/stromblack29/rust-pdf.git`.
- `.gitignore` configured.
- Initial commit created.
