use tonic::{Request, Response, Status};
use crate::compression::{compress_pdf_with_config, CompressionConfig};

// Import the generated proto code
pub mod pb {
    tonic::include_proto!("compression");
}

use pb::compression_service_server::CompressionService;
pub use pb::compression_service_server::CompressionServiceServer;
use pb::{CompressRequest, CompressResponse};

#[derive(Debug, Default)]
pub struct HelperService;

#[tonic::async_trait]
impl CompressionService for HelperService {
    async fn compress_pdf(
        &self,
        request: Request<CompressRequest>,
    ) -> Result<Response<CompressResponse>, Status> {
        let req = request.into_inner();
        let pdf_data = req.pdf_data;
        let original_size = pdf_data.len() as u64;

        println!("Received compression request: {} bytes", original_size);

        // Parse configuration from request or use defaults
        let config = if let Some(proto_config) = req.config {
            CompressionConfig {
                jpeg_quality: proto_config.jpeg_quality.unwrap_or(30) as u8,
                max_dimension: proto_config.max_dimension.unwrap_or(600),
                remove_metadata: proto_config.remove_metadata.unwrap_or(true),
            }
        } else {
            CompressionConfig::default()
        };

        // Call the compression logic
        // This is CPU intensive, so we spawn_blocking
        let compressed_data = match tokio::task::spawn_blocking(move || {
            compress_pdf_with_config(&pdf_data, config)
        }).await {
             Ok(Ok(data)) => data,
             Ok(Err(e)) => return Err(Status::internal(format!("Compression failed: {}", e))),
             Err(e) => return Err(Status::internal(format!("Join error: {}", e))),
        };

        let compressed_size = compressed_data.len() as u64;
        let compression_ratio = if original_size > 0 {
            ((original_size - compressed_size) as f32 / original_size as f32) * 100.0
        } else {
            0.0
        };

        println!("Compression finished: {} bytes -> {} bytes ({:.2}% reduction)", 
                 original_size, compressed_size, compression_ratio);

        Ok(Response::new(CompressResponse {
            compressed_pdf_data: compressed_data.into(),
            original_size,
            compressed_size,
            compression_ratio,
        }))
    }
}
