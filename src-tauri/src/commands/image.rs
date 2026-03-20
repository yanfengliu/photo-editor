use tauri::State;
use crate::state::AppState;
use crate::imaging::loader;

#[tauri::command]
pub async fn load_thumbnail(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::get_thumbnail(&db, &image_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_preview(
    image_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;
    drop(db);
    loader::load_preview(&record.file_path, 2048).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_full_resolution(
    image_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;
    drop(db);
    loader::load_full(&record.file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_exif_data(
    image_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;
    drop(db);
    crate::imaging::exif::read_exif(&record.file_path).map_err(|e| e.to_string())
}
