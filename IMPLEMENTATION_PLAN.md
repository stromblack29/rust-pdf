# Implementation Plan - Rust PDF Compression Service

This service provides a high-performance API for compressing PDF documents. It exposes both gRPC and REST (OpenAPI/Swagger) endpoints.
The core compression engine uses native Rust libraries (`lopdf` and `image`) to parse PDF structures and aggressively re-compress embedded images to achieve target file size reductions.

## User Review Required
> [!IMPORTANT]
> **Compression Strategy**: Achieving 95% compression (5MB -> 100KB) is aggressive and lossy.
> The implementation will focus on:
> 1. Downsampling images to standard screen resolution (e.g., 72-96 DPI).
> 2. Converting images to efficent formats (JPEG) with lower quality settings.
> 3. Stripping unused objects.
>
> **Note**: Pure Rust PDF manipulation is complex. We will use `lopdf` to modify the document structure. Support for all PDF features (e.g., specific color spaces like CMYK) might be limited compared to tools like Ghostscript.

## Proposed Changes

### Project Structure
Organize as a Cargo workspace or single binary with modules.
`src/`
 - `main.rs`: Entry point, sets up servers.
 - `compression/`: Logic for PDF manipulation.
 - `grpc/`: Tonic gRPC service implementation.
 - `api/`: Axum handlers and OpenAPI definition.
 - `proto/`: `.proto` definitions.

### Dependencies
- `tokio`: Async runtime.
- `axum`: REST API.
- `tonic`, `prost`: gRPC.
- `utoipa`, `utoipa-swagger-ui`: OpenAPI.
- `lopdf`: PDF parsing and modification.
- `image`: Image processing (resizing, re-encoding).
- `rayion`: Parallel processing for images (if CPU bound).

### Component: Compression Engine (`src/compression/mod.rs`)
#### [NEW] `compression/mod.rs`
- `compress_pdf(input: &[u8]) -> Result<Vec<u8>>`: Main entry point.
- Iterates through PDF Objects.
- Identifies `XObject` / `Subtype Image`.
- Decodes image stream using `lopdf` filters.
- Uses `image` crate to load, resize (e.g., max dimension 800px), and re-encode as JPEG (Quality 50).
- Replaces original stream with new compressed stream.
- Saves document with cross-reference table reconstruction (using `lopdf`'s `save_to`).

### Component: gRPC Service (`src/grpc/mod.rs`)
#### [NEW] `proto/compression.proto`
- Define `CompressionService` with `CompressPdf` RPC.
- Request: `bytes pdf_data`, `CompressionConfig config`.
- Response: `bytes compressed_pdf_data`.

#### [NEW] `src/grpc/server.rs`
- Implement the Tonic service trait.
- Call `compression::compress_pdf`.

### Component: REST API (`src/api/mod.rs`)
#### [NEW] `src/api/routes.rs`
- `POST /compress`: Multipart upload or raw bytes body.
- Returns compressed PDF.
- Annotate with `#[utoipa::path]` for Swagger.

#### [NEW] `src/api/docs.rs`
- `ApiDoc` struct deriving `OpenApi`.

### Component: Main (`src/main.rs`)
- Initialize tracing/logging.
- Spawn gRPC server on one port (e.g., 50051).
- Spawn Axum server on another port (e.g., 3000).
- Handle graceful shutdown.

## Verification Plan

### Automated Tests
- Unit tests for `compress_pdf` using a sample PDF (need to generate or mock one).
- Integration tests for gRPC client.
- Integration test for REST endpoint using `reqwest`.

### Manual Verification
- Run server.
- Use `grpcurl` or a custom client to test gRPC.
- Open `http://localhost:3000/swagger-ui` to test REST API.
- Verify output PDF size and readability.
