use std::path::Path;
use image::DynamicImage;

pub fn load_raw_preview(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    Err(format!("RAW not yet supported: {}. LibRaw FFI required.", path.display()).into())
}

pub fn load_raw_full(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    Err(format!("RAW not yet supported: {}. LibRaw FFI required.", path.display()).into())
}
