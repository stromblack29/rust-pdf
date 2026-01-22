use lopdf::{Document, Object, Stream};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use image::{DynamicImage, ImageFormat};
use rayon::prelude::*;

pub fn compress_pdf(input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut doc = Document::load_mem(input)?;

    // Collect all stream objects that are images
    // We need to collect object IDs first to avoid borrowing issues while modifying
    let image_ids: Vec<_> = doc.objects.iter()
        .filter(|(_, obj)| {
            if let Ok(stream) = obj.as_stream() {
                 match stream.dict.get(b"Subtype") {
                     Ok(Object::Name(name)) if name == b"Image" => true,
                     _ => false,
                 }
            } else {
                false
            }
        })
        .map(|(id, _)| *id)
        .collect();

    println!("Found {} images to process", image_ids.len());

    // Process images in parallel
    // We can't modify the document in parallel easily, so we process the data then update.
    // However, extracting content requires access to the doc.
    // For simplicity in this iteration, we might process sequentially or careful extraction.
    
    // Let's iterate and process.
    // Needed: ID, Original Data (decoded), Filter info to re-encode.
    
    // Strategy: 
    // 1. Identify images.
    // 2. Decode them using lopdf to raw bytes.
    // 3. Process with `image` crate (resize, compress to JPEG).
    // 4. Replace the Stream content in the Document.
    
    for object_id in image_ids {
        // We have to handle errors gracefully to avoid failing the whole PDF if one image fails
        if let Ok(processed_stream) = process_image_object(&doc, object_id) {
            if let Some(obj) = doc.objects.get_mut(&object_id) {
                *obj = processed_stream;
            }
        }
    }

    // Remove unused objects (simple garbage collection)
    doc.prune_objects();

    // Compress streams (general PDF compression)
    doc.compress();

    // Save to memory
    let mut out_buffer = Vec::new();
    doc.save_to(&mut out_buffer)?;
    
    Ok(out_buffer)
}

fn process_image_object(doc: &Document, object_id: (u32, u16)) -> Result<Object, Box<dyn std::error::Error + Send + Sync>> {
    let object = doc.get_object(object_id)?;
    let stream = object.as_stream()?;
    
    // Try to decode the stream
    // lopdf's decode_content usually handles filters like FlateDecode
    // But for Images, we might need to be careful with color spaces.
    
    // Note: lopdf decode might just give raw pixels or formatted data depending on filters.
    // If it's a JPEG (DCTDecode), we can try to re-compress it if it's too large.
    // If it's a raw bitmap or PNG (FlateDecode), we can convert to JPEG.
    
    let content_data = stream.content.clone(); 
    // We interpret the stream. If it has filters, we should try to decode it.
    // However, lopdf `get_content` or `decode` doesn't fully handle all image conversion logic (like CMYK -> RGB).
    
    // Let's attempt to use the raw bytes if it is already an image format, or decode if it is a stream of pixels.
    // A robust PDF image extractor works by checking filters.
    
    let filters = stream.dict.get(b"Filter");
    let is_jpeg = match filters {
        Ok(Object::Name(name)) => name == b"DCTDecode",
        Ok(Object::Array(arr)) => arr.contains(&Object::Name(b"DCTDecode".to_vec())),
        _ => false,
    };

    // If it's already JPEG, we might still want to downscale.
    // But decoding PDF image streams is complex (ColorSpace, BitsPerComponent).
    // For this MVP, we will try to decode using `lopdf::Stream::decode` which handles Flate/LZW etc.
    
    let decoded_bytes = match stream.decompressed_content() {
        Ok(bytes) => bytes,
        Err(_) => return Err("Failed to decompress".into()),
    };

    // Now we have raw bytes. But we need to know the dimensions and color space to form an image.
    let width = stream.dict.get(b"Width").and_then(|v| v.as_i64()).unwrap_or(0) as u32;
    let height = stream.dict.get(b"Height").and_then(|v| v.as_i64()).unwrap_or(0) as u32;
    // let color_space = stream.dict.get(b"ColorSpace"); // Needed for interpretation
    
    if width == 0 || height == 0 {
         return Err("Invalid dimensions".into());
    }

    // This is the tricky part: standardizing the image data for `image` crate.
    // If it was DCTDecode, `decoded_bytes` might still be the raw JPEG stream if we accessed `content` directly, 
    // but `decompressed_content` tries to undo filters.
    // If `is_jpeg` is true, simple decoding might fail if we want the actual JPEG bytes vs raw pixels.
    
    // Better approach for MVP:
    // If it is DCTDecode (JPEG), try to load it as an image directly from raw content (before decompressing).
    // If it is FlateDecode, use decoded bytes and assume RGB/Grayscale? (Dangerous assumptions but common start).
    
    let img: DynamicImage = if is_jpeg {
         // Load from raw content (stream.content is likely the JPEG file)
         // Note: PDF streams might have extra params.
         ImageReader::new(Cursor::new(&stream.content))
            .with_guessed_format()?
            .decode()?
    } else {
        // Raw pixel data. We need to construct based on Width/Height/ColorSpace/BitsPerComponent.
        // This is complex. `image` crate can load from raw if we know the format.
        // For now, let's skip re-compressing non-JPEG complex images to avoid corruption, 
        // unless we are sure.
        
        // Simplified fallback: if we can't easily identify, skip.
        // Or Try to form an ImageBuffer.
        return Err("Non-JPEG image re-compression not fully implemented in MVP".into());
    };

    // Resize (Downscale)
    // Target: Max 800px on longest side, or preserve if smaller.
    let (w, h) = (img.width(), img.height());
    let max_dim = 1200; // Reasonable quality
    let new_img = if w > max_dim || h > max_dim {
        img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Re-encode to JPEG with lower quality
    let mut comp_bytes: Vec<u8> = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut comp_bytes, 50);
    encoder.encode(new_img.as_bytes(), new_img.width(), new_img.height(), new_img.color().into())?;
    
    // Create new stream dictionary
    let mut new_dict = stream.dict.clone();
    new_dict.set(b"Filter", Object::Name(b"DCTDecode".to_vec()));
    new_dict.set(b"Width", Object::Integer(new_img.width() as i64));
    new_dict.set(b"Height", Object::Integer(new_img.height() as i64));
    new_dict.set(b"Length", Object::Integer(comp_bytes.len() as i64));
    // Remove other filters/params that might conflict
    new_dict.remove(b"DecodeParms");
    new_dict.set(b"ColorSpace", Object::Name(b"DeviceRGB".to_vec())); // JPEG is usually DeviceRGB or DeviceGray

    Ok(Object::Stream(Stream::new(new_dict, comp_bytes)))
}
