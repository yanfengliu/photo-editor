use std::path::Path;
use image::{DynamicImage, ImageBuffer, RgbImage};
use quickraw::{data, DemosaicingMethod, Export, Input, Output, OutputType};

pub fn is_raw_extension(ext: &str) -> bool {
    matches!(ext, "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "raf" | "pef")
}

pub fn is_raw_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| is_raw_extension(&ext.to_lowercase()))
        .unwrap_or(false)
}

pub fn load_raw_preview(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    if let Some(preview) = try_load_embedded_thumbnail(path)? {
        return Ok(preview);
    }

    render_raw_image(path)
}

pub fn load_raw_full(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    render_raw_image(path)
}

pub fn raw_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy();
    let info = Export::export_exif_info(Input::ByFile(path_str.as_ref()))?;
    let width = info.usize("width").unwrap_or(0) as u32;
    let height = info.usize("height").unwrap_or(0) as u32;

    if width > 0 && height > 0 {
        Ok((width, height))
    } else {
        let image = render_raw_image(path)?;
        Ok((image.width(), image.height()))
    }
}

fn try_load_embedded_thumbnail(
    path: &Path,
) -> Result<Option<DynamicImage>, Box<dyn std::error::Error>> {
    let raw_data = std::fs::read(path)?;
    match Export::export_thumbnail_data(&raw_data) {
        Ok((thumbnail_data, _orientation)) => {
            let preview = image::load_from_memory(&thumbnail_data)?;
            Ok(Some(preview))
        }
        Err(_) => Ok(None),
    }
}

fn render_raw_image(path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy();
    let export_job = Export::new(
        Input::ByFile(path_str.as_ref()),
        Output::new(
            DemosaicingMethod::Linear,
            data::XYZ2SRGB,
            data::GAMMA_SRGB,
            OutputType::Raw8,
            true,
            true,
        ),
    )?;

    let (pixels, width, height) = export_job.export_8bit_image();
    let image = rgb_from_raw(pixels, width as u32, height as u32)?;
    Ok(DynamicImage::ImageRgb8(image))
}

fn rgb_from_raw(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<RgbImage, Box<dyn std::error::Error>> {
    ImageBuffer::from_raw(width, height, pixels)
        .ok_or_else(|| "Failed to create RAW RGB image buffer".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_supported_raw_extensions() {
        assert!(is_raw_extension("cr3"));
        assert!(is_raw_extension("nef"));
        assert!(!is_raw_extension("jpg"));
    }

    #[test]
    fn invalid_raw_file_returns_error() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("photo-editor-invalid-raw-{stamp}.cr3"));
        std::fs::write(&path, b"not a raw file").unwrap();

        let result = load_raw_preview(&path);

        assert!(result.is_err());
        std::fs::remove_file(path).unwrap();
    }
}
