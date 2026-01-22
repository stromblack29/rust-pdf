use axum::{
    extract::Multipart,
    routing::{post, get},
    Router,
    response::{IntoResponse, Response},
    Json,
    body::Body,
};
use axum::http::{StatusCode, header};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::compression::compress_pdf;

#[derive(OpenApi)]
#[openapi(
    paths(
        compress_handler_multipart,
    ),
    tags(
        (name = "compression", description = "PDF Compression API")
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

/// Compress a PDF file uploaded via multipart form
#[utoipa::path(
    post,
    path = "/compress",
    request_body(content = String, description = "PDF file", content_type = "multipart/form-data"), 
    responses(
        (status = 200, description = "Compressed PDF", body = String, content_type = "application/pdf"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
async fn compress_handler_multipart(mut multipart: Multipart) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Basic multipart handling: look for a field named "file" or just take the first file
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
            // Offload to blocking thread
            let compressed = tokio::task::spawn_blocking(move || {
                compress_pdf(&data)
            }).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Return as PDF
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"compressed.pdf\"")
                .body(Body::from(compressed))
                .unwrap());
        }
    }

    Err((StatusCode::BAD_REQUEST, "No file field found".to_string()))
}
