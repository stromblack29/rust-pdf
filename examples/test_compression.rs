use rustpdf::compression::compress_pdf;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_path = "Policy EasyCare VISA[LTR] Online_G9022138.pdf";
    let output_path = "Policy_compressed.pdf";
    
    let input_data = fs::read(input_path)?;
    println!("Original size: {} bytes", input_data.len());
    
    let start = Instant::now();
    let compressed_data = compress_pdf(&input_data)?;
    let duration = start.elapsed();
    
    println!("Compressed size: {} bytes", compressed_data.len());
    println!("Time taken: {:?}", duration);
    println!("Compression Ratio: {:.2}%", (1.0 - (compressed_data.len() as f64 / input_data.len() as f64)) * 100.0);
    
    fs::write(output_path, compressed_data)?;
    println!("Saved to {}", output_path);
    
    Ok(())
}
