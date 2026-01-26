use axum::{
    extract::{Multipart, Query},
    routing::{post, get},
    Router,
    response::{IntoResponse, Response},
    body::Body,
};
use axum::http::{StatusCode, header};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::compression::{compress_pdf_with_config, CompressionConfig};
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(
    paths(
        compress_handler_multipart,
    ),
    components(
        schemas(CompressionQueryParams, CompressionStats)
    ),
    tags(
        (name = "compression", description = "PDF Compression API - Target 90% size reduction")
    )
)]
pub struct ApiDoc;

use axum::extract::DefaultBodyLimit;

pub fn app_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/compress", post(compress_handler_multipart))
        .route("/health", get(|| async { "OK" }))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit
}

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct CompressionQueryParams {
    /// JPEG quality (1-100). Lower = smaller file. Default: 30 for 90% reduction
    #[serde(default = "default_quality")]
    pub quality: u8,
    /// Maximum dimension in pixels. Default: 600 for 90% reduction
    #[serde(default = "default_max_dim")]
    pub max_dimension: u32,
    /// Remove metadata from PDF. Default: true
    #[serde(default = "default_remove_metadata")]
    pub remove_metadata: bool,
}

fn default_quality() -> u8 { 30 }
fn default_max_dim() -> u32 { 600 }
fn default_remove_metadata() -> bool { true }

impl Default for CompressionQueryParams {
    fn default() -> Self {
        Self {
            quality: 30,
            max_dimension: 600,
            remove_metadata: true,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CompressionStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f32,
}

/// Compress a PDF file uploaded via multipart form
#[utoipa::path(
    post,
    path = "/compress",
    params(CompressionQueryParams),
    request_body(content = String, description = "PDF file", content_type = "multipart/form-data"), 
    responses(
        (status = 200, description = "Compressed PDF with statistics in headers", body = String, content_type = "application/pdf"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
async fn compress_handler_multipart(
    Query(params): Query<CompressionQueryParams>,
    mut multipart: Multipart
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Basic multipart handling: look for a field named "file" or just take the first file
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let original_size = data.len() as u64;
            
            // Create compression config from query params
            let config = CompressionConfig {
                jpeg_quality: params.quality,
                max_dimension: params.max_dimension,
                remove_metadata: params.remove_metadata,
            };
            
            // Offload to blocking thread
            let compressed = tokio::task::spawn_blocking(move || {
                compress_pdf_with_config(&data, config)
            }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let compressed_size = compressed.len() as u64;
            let compression_ratio = if original_size > 0 {
                ((original_size - compressed_size) as f32 / original_size as f32) * 100.0
            } else {
                0.0
            };

            // Return as PDF with compression statistics in headers
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"compressed.pdf\"")
                .header("X-Original-Size", original_size.to_string())
                .header("X-Compressed-Size", compressed_size.to_string())
                .header("X-Compression-Ratio", format!("{:.2}", compression_ratio))
                .body(Body::from(compressed))
                .unwrap());
        }
    }

    Err((StatusCode::BAD_REQUEST, "No file field found".to_string()))
}
