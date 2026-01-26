use rustpdf::grpc::pb::{compression_service_client::CompressionServiceClient, CompressRequest, CompressionConfig};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Testing gRPC Compression Service ===\n");

    // Connect to the gRPC server
    let mut client = CompressionServiceClient::connect("http://[::1]:50051").await?;
    println!("Connected to gRPC server at [::1]:50051\n");

    // Read test PDF
    let input_path = "examples/Policy EasyCare VISA[LTR] Online_G9022138.pdf";
    
    if !std::path::Path::new(input_path).exists() {
        println!("Test PDF not found. Please place a PDF file at: {}", input_path);
        println!("Make sure the server is running: cargo run");
        return Ok(());
    }

    let pdf_data = fs::read(input_path)?;
    let original_size = pdf_data.len();
    
    println!("Original file: {}", input_path);
    println!("Original size: {} bytes ({:.2} MB)", original_size, original_size as f64 / 1_048_576.0);
    println!();

    // Test with default 90% compression settings
    println!("Sending compression request with default settings (90% target)...");
    let request = tonic::Request::new(CompressRequest {
        pdf_data: pdf_data.clone(),
        config: Some(CompressionConfig {
            jpeg_quality: Some(30),
            max_dimension: Some(600),
            remove_metadata: Some(true),
        }),
    });

    let response = client.compress_pdf(request).await?;
    let result = response.into_inner();
    
    println!("\n=== Compression Results ===");
    println!("Original size: {} bytes ({:.2} MB)", result.original_size, result.original_size as f64 / 1_048_576.0);
    println!("Compressed size: {} bytes ({:.2} MB)", result.compressed_size, result.compressed_size as f64 / 1_048_576.0);
    println!("Compression ratio: {:.2}%", result.compression_ratio);
    
    // Save the compressed PDF
    let output_path = "examples/grpc_compressed_output.pdf";
    fs::write(output_path, &result.compressed_pdf_data)?;
    println!("\nCompressed PDF saved to: {}", output_path);

    Ok(())
}
