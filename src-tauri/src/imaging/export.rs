use std::path::Path;
use std::io::Cursor;
use image::{DynamicImage, RgbaImage, ImageFormat};

pub fn export_pixels(
    rgba_data: &[u8], width: u32, height: u32, output_path: &str, format: &str, quality: u8, max_dimension: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = RgbaImage::from_raw(width, height, rgba_data.to_vec()).ok_or("Failed to create image from RGBA data")?;
    let mut img = DynamicImage::ImageRgba8(img);
    if let Some(max) = max_dimension {
        if img.width() > max || img.height() > max {
            img = img.resize(max, max, image::imageops::FilterType::Lanczos3);
        }
    }
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    match format {
        "png" => img.save_with_format(path, ImageFormat::Png)?,
        "tiff" => img.save_with_format(path, ImageFormat::Tiff)?,
        _ => {
            let rgb = img.to_rgb8();
            let mut buf = Cursor::new(Vec::new());
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
            rgb.write_with_encoder(encoder)?;
            std::fs::write(path, buf.into_inner())?;
        }
    }
    Ok(())
}
