use std::path::{Path, PathBuf};
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
    import_paths(db, &[folder_path.to_string()])
}

pub fn import_paths(
    db: &Database,
    paths: &[String],
) -> Result<Vec<ImageRecord>, Box<dyn std::error::Error>> {
    let entries = collect_supported_entries(paths)?;

    let results: Vec<_> = entries
        .par_iter()
        .map(|path| {
            (
                path.clone(),
                import_single_file(path).map_err(|err| err.to_string()),
            )
        })
        .collect();

    let mut imported = Vec::new();
    let mut failures = Vec::new();

    for (path, result) in results {
        match result {
            Ok((record, thumb_data)) => match db.conn.execute(
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
                Ok(affected_rows) if affected_rows > 0 => imported.push(record),
                Ok(_) => {}
                Err(err) => failures.push(format!("{}: {}", path.display(), err)),
            },
            Err(err) => failures.push(format!("{}: {}", path.display(), err)),
        }
    }

    if imported.is_empty() && !failures.is_empty() {
        return Err(format!("Failed to import the selected files. {}", failures[0]).into());
    }

    if !failures.is_empty() {
        log::warn!("Skipped {} files during import", failures.len());
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

fn collect_supported_entries(
    paths: &[String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();

    for input in paths {
        let path = Path::new(input);

        if path.is_dir() {
            entries.extend(
                WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_type().is_file() && is_supported_image_path(entry.path()))
                    .map(|entry| entry.into_path()),
            );
        } else if path.is_file() {
            if !is_supported_image_path(path) {
                return Err(format!("Unsupported file type: {}", path.display()).into());
            }
            entries.push(path.to_path_buf());
        } else {
            return Err(format!("Selected path does not exist: {}", input).into());
        }
    }

    entries.sort();
    entries.dedup();

    if entries.is_empty() {
        return Err("No supported image files were found in the selected sources.".into());
    }

    Ok(entries)
}

fn is_supported_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("photo-editor-import-{name}-{stamp}"))
    }

    #[test]
    fn collect_supported_entries_supports_files_and_directories() {
        let root = temp_path("sources");
        let nested = root.join("nested");
        let selected_file = root.join("selected.jpg");
        let nested_file = nested.join("nested.png");
        let raw_file = root.join("raw.cr3");

        fs::create_dir_all(&nested).unwrap();
        fs::write(&selected_file, []).unwrap();
        fs::write(&nested_file, []).unwrap();
        fs::write(&raw_file, []).unwrap();

        let entries = collect_supported_entries(&[
            selected_file.to_string_lossy().into_owned(),
            root.to_string_lossy().into_owned(),
        ])
        .unwrap();

        assert!(entries.contains(&selected_file));
        assert!(entries.contains(&nested_file));
        assert!(entries.contains(&raw_file));
        assert_eq!(entries.iter().filter(|path| **path == selected_file).count(), 1);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn collect_supported_entries_rejects_unsupported_selected_files() {
        let root = temp_path("unsupported");
        let text_file = root.join("notes.txt");

        fs::create_dir_all(&root).unwrap();
        fs::write(&text_file, b"notes").unwrap();

        let err = collect_supported_entries(&[text_file.to_string_lossy().into_owned()])
            .unwrap_err()
            .to_string();

        assert!(err.contains("Unsupported file type"));

        fs::remove_dir_all(&root).unwrap();
    }
}
