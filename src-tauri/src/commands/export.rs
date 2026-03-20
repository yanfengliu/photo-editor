use tauri::State;
use crate::state::AppState;
use crate::catalog::models::EditParams;

#[derive(serde::Deserialize)]
pub struct ExportSettings {
    pub format: String,
    pub quality: u8,
    pub output_path: String,
    pub max_dimension: Option<u32>,
}

#[tauri::command]
pub async fn export_image(
    state: State<'_, AppState>,
    image_id: String,
    settings: ExportSettings,
) -> Result<String, String> {
    let file_path = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
            .map_err(|e| e.to_string())?;
        record.file_path.clone()
    };

    let edit_params: EditParams = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
            .map_err(|e| e.to_string())?;
        match record.edit_params {
            Some(ref json) => serde_json::from_str(json).unwrap_or_default(),
            None => EditParams::default(),
        }
    };

    let pixels = crate::imaging::loader::load_full(&file_path)
        .map_err(|e| e.to_string())?;
    let processed = crate::gpu::pipeline::apply_edits_cpu(&pixels, &edit_params);

    crate::imaging::export::export_pixels(
        &processed,
        &settings.output_path,
        &settings.format,
        settings.quality,
        settings.max_dimension,
    ).map_err(|e| e.to_string())?;

    Ok(settings.output_path)
}

#[tauri::command]
pub async fn batch_export(
    state: State<'_, AppState>,
    image_ids: Vec<String>,
    settings: ExportSettings,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for image_id in &image_ids {
        let file_path = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let record = crate::catalog::queries::get_image_by_id(&db, image_id)
                .map_err(|e| e.to_string())?;
            record.file_path.clone()
        };

        let file_name = std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        let ext = match settings.format.as_str() {
            "png" => "png",
            "tiff" => "tiff",
            _ => "jpg",
        };
        let output_path = format!("{}/{}.{}", settings.output_path, file_name, ext);

        let edit_params: EditParams = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let record = crate::catalog::queries::get_image_by_id(&db, image_id)
                .map_err(|e| e.to_string())?;
            match record.edit_params {
                Some(ref json) => serde_json::from_str(json).unwrap_or_default(),
                None => EditParams::default(),
            }
        };

        let pixels = crate::imaging::loader::load_full(&file_path)
            .map_err(|e| e.to_string())?;
        let processed = crate::gpu::pipeline::apply_edits_cpu(&pixels, &edit_params);

        crate::imaging::export::export_pixels(
            &processed,
            &output_path,
            &settings.format,
            settings.quality,
            settings.max_dimension,
        ).map_err(|e| e.to_string())?;

        results.push(output_path);
    }
    Ok(results)
}
