use std::path::Path;
use image::GenericImageView;

pub fn load_preview(file_path: &str, max_size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let img = if is_raw(&ext) {
        crate::imaging::raw::load_raw_preview(path)?
    } else {
        image::open(path)?
    };
    let img = if img.width() > max_size || img.height() > max_size {
        img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3)
    } else { img };
    Ok(img.to_rgba8().into_raw())
}

pub fn load_full(file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let img = if is_raw(&ext) {
        crate::imaging::raw::load_raw_full(path)?
    } else {
        image::open(path)?
    };
    Ok(img.to_rgba8().into_raw())
}

fn is_raw(ext: &str) -> bool {
    matches!(ext, "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "raf" | "pef")
}
