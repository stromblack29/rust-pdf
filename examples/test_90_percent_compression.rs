use rustpdf::compression::{compress_pdf_with_config, CompressionConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Testing 90% Compression Target ===\n");

    // Test with the existing PDF
    let input_path = "examples/Policy EasyCare VISA[LTR] Online_G9022138.pdf";
    
    if !std::path::Path::new(input_path).exists() {
        println!("Test PDF not found. Please place a PDF file at: {}", input_path);
        return Ok(());
    }

    let input_data = fs::read(input_path)?;
    let original_size = input_data.len();
    
    println!("Original file: {}", input_path);
    println!("Original size: {} bytes ({:.2} MB)", original_size, original_size as f64 / 1_048_576.0);
    println!();

    // Test 1: Default aggressive compression (90% target)
    println!("Test 1: Default Aggressive Compression (Quality 30, Max 600px)");
    let config1 = CompressionConfig::default();
    let compressed1 = compress_pdf_with_config(&input_data, config1)?;
    let size1 = compressed1.len();
    let ratio1 = ((original_size - size1) as f64 / original_size as f64) * 100.0;
    
    fs::write("examples/test_90_default.pdf", &compressed1)?;
    println!("  Compressed size: {} bytes ({:.2} MB)", size1, size1 as f64 / 1_048_576.0);
    println!("  Compression ratio: {:.2}%", ratio1);
    println!("  Saved to: examples/test_90_default.pdf");
    println!();

    // Test 2: Ultra aggressive compression (95% target)
    println!("Test 2: Ultra Aggressive Compression (Quality 20, Max 400px)");
    let config2 = CompressionConfig {
        jpeg_quality: 20,
        max_dimension: 400,
        remove_metadata: true,
    };
    let compressed2 = compress_pdf_with_config(&input_data, config2)?;
    let size2 = compressed2.len();
    let ratio2 = ((original_size - size2) as f64 / original_size as f64) * 100.0;
    
    fs::write("examples/test_95_ultra.pdf", &compressed2)?;
    println!("  Compressed size: {} bytes ({:.2} MB)", size2, size2 as f64 / 1_048_576.0);
    println!("  Compression ratio: {:.2}%", ratio2);
    println!("  Saved to: examples/test_95_ultra.pdf");
    println!();

    // Test 3: Moderate compression (70% target)
    println!("Test 3: Moderate Compression (Quality 50, Max 1000px)");
    let config3 = CompressionConfig {
        jpeg_quality: 50,
        max_dimension: 1000,
        remove_metadata: true,
    };
    let compressed3 = compress_pdf_with_config(&input_data, config3)?;
    let size3 = compressed3.len();
    let ratio3 = ((original_size - size3) as f64 / original_size as f64) * 100.0;
    
    fs::write("examples/test_70_moderate.pdf", &compressed3)?;
    println!("  Compressed size: {} bytes ({:.2} MB)", size3, size3 as f64 / 1_048_576.0);
    println!("  Compression ratio: {:.2}%", ratio3);
    println!("  Saved to: examples/test_70_moderate.pdf");
    println!();

    println!("=== Summary ===");
    println!("Original: {:.2} MB", original_size as f64 / 1_048_576.0);
    println!("Default (90% target): {:.2} MB ({:.2}% reduction)", size1 as f64 / 1_048_576.0, ratio1);
    println!("Ultra (95% target): {:.2} MB ({:.2}% reduction)", size2 as f64 / 1_048_576.0, ratio2);
    println!("Moderate (70% target): {:.2} MB ({:.2}% reduction)", size3 as f64 / 1_048_576.0, ratio3);

    Ok(())
}
