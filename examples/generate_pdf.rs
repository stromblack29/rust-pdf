use lopdf::{Document, Object, Stream, Dictionary};
use lopdf::content::{Content, Operation};
use image::{ImageBuffer, Rgb};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Font".into()),
        ("Subtype", "Type1".into()),
        ("BaseFont", "Courier".into()),
    ]));
    let resources_id = doc.add_object(Dictionary::from_iter(vec![
        ("Font", Dictionary::from_iter(vec![("F1", font_id.into())]).into()),
    ]));
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 48.into()]),
            Operation::new("Td", vec![100.into(), 600.into()]),
            Operation::new("Tj", vec![Object::String(b"Test PDF with Image".to_vec(), lopdf::StringFormat::Literal)]),
            Operation::new("ET", vec![]),
        ],
    };
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.encode().unwrap()));
    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Page".into()),
        ("Parent", pages_id.into()),
        ("Contents", content_id.into()),
    ]));
    let pages = Dictionary::from_iter(vec![
        ("Type", "Pages".into()),
        ("Kids", vec![page_id.into()].into()),
        ("Count", 1.into()),
        ("Resources", resources_id.into()),
        ("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()].into()),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Catalog".into()),
        ("Pages", pages_id.into()),
    ]));
    doc.trailer.set("Root", catalog_id);
    
    // Add a large random image to simulate content
    // Create 2000x2000 image
    let width = 2000;
    let height = 2000;
    let mut img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
        // Gradient pattern which compresses better than random noise
        *pixel = Rgb([((x as f32 / width as f32) * 255.0) as u8, ((y as f32 / height as f32) * 255.0) as u8, 128]);
    }
    
    let mut jpeg_data = Vec::new();
    // High quality to make it large
    img_buf.write_to(&mut Cursor::new(&mut jpeg_data), image::ImageFormat::Jpeg).unwrap();
    
    let image_stream = Stream::new(
        Dictionary::from_iter(vec![
            ("Type", "XObject".into()),
            ("Subtype", "Image".into()),
            ("Width", (width as i64).into()),
            ("Height", (height as i64).into()),
            ("BitsPerComponent", 8.into()),
            ("ColorSpace", "DeviceRGB".into()),
            ("Filter", "DCTDecode".into()),
        ]),
        jpeg_data,
    );
    let image_id = doc.add_object(Object::Stream(image_stream));
    
    // Link image to resources (basic link, won't show on page unless referenced in content, but exists in file)
    if let Ok(Object::Dictionary(res)) = doc.get_object_mut(resources_id) {
         let mut xobject = Dictionary::new();
         xobject.set("Im1", Object::Reference(image_id));
         res.set("XObject", Object::Dictionary(xobject));
    }
    
    doc.save("test_large.pdf")?;
    println!("Created test_large.pdf");
    Ok(())
}
