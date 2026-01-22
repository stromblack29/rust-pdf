# Rust PDF Compression Service

A high-performance microservice for compressing PDF documents, built with Rust. 
It provides both **gRPC** and **REST** APIs and uses aggressive image optimization tactics (downscaling + JPEG re-encoding) to achieve significant file size reductions (target 95% reduction for image-heavy PDFs).

## Features

- **High Performance**: Native Rust implementation using `lopdf` and `image` crates.
- **Dual API**:
  - **gRPC**: High-throughput inter-service communication via `tonic` (Port 50051).
  - **REST**: Easy integration via `axum`, with **Swagger UI** for testing (Port 3000).
- **concurrency**: Heavy compression tasks are offloaded to a thread pool to keep the API responsive (`spawn_blocking`).
- **OpenAPI Documentation**: Automatically generated Swagger docs.

## Quick Start

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- `protoc` (Protocol Buffers compiler)

### Running the Server
```bash
cargo run
```
The service will start on:
- **gRPC**: `[::1]:50051`
- **REST**: `http://127.0.0.1:3000`
- **Swagger UI**: [http://127.0.0.1:3000/swagger-ui/](http://127.0.0.1:3000/swagger-ui/)

## Testing

### Generate Test PDF
Create a large PDF (~2MB) with generated images to test compression:
```bash
cargo run --example generate_pdf
```

### Verify Compression
Run the compression logic on the generated PDF:
```bash
cargo run --example test_compression
```

## Architecture

- **`src/compression`**: Core logic. Parses PDF, finds images, downscales/compresses them, and rebuilds the PDF.
- **`src/grpc`**: gRPC service implementation defined in `proto/compression.proto`.
- **`src/api`**: REST/HTTP handlers. Includes `utoipa` definitions for Swagger.

## Documentation
- [Usage Walkthrough](WALKTHROUGH.md)
- [Implementation Plan](IMPLEMENTATION_PLAN.md)
