use rustpdf::grpc;
use rustpdf::api;
use std::net::SocketAddr;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    println!("Starting Rust PDF Compression Service...");

    // gRPC Server setup
    let grpc_addr = "[::1]:50051".parse()?;
    let grpc_service = grpc::HelperService::default();
    
    // REST API setup
    let app = api::app_router();
    let rest_addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("gRPC listening on {}", grpc_addr);
    println!("REST listening on http://{}", rest_addr);
    println!("Swagger UI available at http://{}/swagger-ui/", rest_addr);

    // Run both servers
    // Note: In production you might want better graceful shutdown coordination
    
    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc::CompressionServiceServer::new(grpc_service))
        .serve(grpc_addr);

    let rest_server = axum::serve(
        tokio::net::TcpListener::bind(rest_addr).await?, 
        app
    );

    tokio::select! {
        result = grpc_server => {
             println!("gRPC server failed: {:?}", result);
        },
        result = rest_server => {
             println!("REST server failed: {:?}", result);
        }
    }

    Ok(())
}
