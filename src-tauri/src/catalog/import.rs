use std::path::Path;
use rayon::prelude::*;
use walkdir::WalkDir;
use crate::catalog::db::Database;
use crate::catalog::models::ImageRecord;
use crate::imaging::{thumbnail, exif};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "bmp", "webp",
    "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef",
];

pub fn import_folder(
    db: &Database,
    folder_path: &str,
) -> Result<Vec<ImageRecord>, Box<dyn std::error::Error>> {
    let entries: Vec<_> = WalkDir::new(folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .collect();

    let results: Vec<_> = entries
        .par_iter()
        .filter_map(|entry| import_single_file(entry.path()).ok())
        .collect();

    let mut imported = Vec::new();
    for (record, thumb_data) in &results {
        match db.conn.execute(
            "INSERT OR IGNORE INTO images (id, file_path, file_name, format, width, height, date_taken,
             camera, lens, iso, focal_length, aperture, shutter_speed, thumbnail, exif_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                record.id, record.file_path, record.file_name, record.format,
                record.width, record.height, record.date_taken, record.camera,
                record.lens, record.iso, record.focal_length, record.aperture,
                record.shutter_speed, thumb_data, serde_json::Value::Null.to_string(),
            ],
        ) {
            Ok(_) => imported.push(record.clone()),
            Err(e) => log::warn!("Failed to import {}: {}", record.file_path, e),
        }
    }
    Ok(imported)
}

fn import_single_file(path: &Path) -> Result<(ImageRecord, Vec<u8>), Box<dyn std::error::Error>> {
    let file_path = path.to_string_lossy().to_string();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    let format = match ext.as_str() {
        "jpg" | "jpeg" => "jpeg",
        "png" => "png",
        "tiff" | "tif" => "tiff",
        "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" | "raf" | "pef" => "raw",
        _ => "jpeg",
    }.to_string();

    let (width, height) = image::image_dimensions(path).unwrap_or((0, 0));
    let exif_data = exif::read_exif_basic(path);
    let thumb_data = thumbnail::generate_thumbnail(path, 256)?;

    let record = ImageRecord {
        id: uuid::Uuid::new_v4().to_string(),
        file_path: file_path.replace('\\', "/"),
        file_name, format, width, height,
        date_taken: exif_data.date_taken,
        rating: 0,
        color_label: "none".to_string(),
        flag: "none".to_string(),
        camera: exif_data.camera, lens: exif_data.lens,
        iso: exif_data.iso, focal_length: exif_data.focal_length,
        aperture: exif_data.aperture, shutter_speed: exif_data.shutter_speed,
        edit_params: None, tags: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    Ok((record, thumb_data))
}
