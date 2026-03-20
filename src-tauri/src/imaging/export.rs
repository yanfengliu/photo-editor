use std::path::Path;
use std::io::Cursor;
use image::{DynamicImage, RgbaImage, ImageFormat};

pub fn export_pixels(
    rgba_data: &[u8], output_path: &str, format: &str, quality: u8, max_dimension: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pixel_count = rgba_data.len() / 4;
    let side = (pixel_count as f64).sqrt() as u32;
    let height = if side > 0 { pixel_count as u32 / side } else { 0 };
    let img = RgbaImage::from_raw(side, height, rgba_data.to_vec()).ok_or("Failed to create image")?;
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
