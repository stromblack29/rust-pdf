use tonic::{Request, Response, Status};
use crate::compression::compress_pdf;

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

        println!("Received compression request: {} bytes", pdf_data.len());

        // Call the compression logic
        // This is CPU intensive, so we might want to spawn_blocking if we were doing it directly here.
        // But since we are inside an async fn, we definitely should.
        
        let compressed_data = match tokio::task::spawn_blocking(move || {
            compress_pdf(&pdf_data)
        }).await {
             Ok(Ok(data)) => data,
             Ok(Err(e)) => return Err(Status::internal(format!("Compression failed: {}", e))),
             Err(e) => return Err(Status::internal(format!("Join error: {}", e))),
        };

        println!("Compression request finished: {} bytes", compressed_data.len());

        Ok(Response::new(CompressResponse {
            compressed_pdf_data: compressed_data.into(),
        }))
    }
}
