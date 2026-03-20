use std::path::Path;
use std::io::Cursor;

pub fn generate_thumbnail(path: &Path, size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = if crate::imaging::raw::is_raw_path(path) {
        crate::imaging::raw::load_raw_preview(path)?
    } else {
        image::open(path)?
    };
    let thumb = img.resize(size, size, image::imageops::FilterType::Triangle);
    let rgb = thumb.to_rgb8();
    let mut buf = Cursor::new(Vec::new());
    rgb.write_to(&mut buf, image::ImageFormat::Jpeg)?;
    Ok(buf.into_inner())
}
