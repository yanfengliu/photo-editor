use tauri::State;
use crate::state::AppState;
use crate::catalog::models::{ImageRecord, CollectionRecord, ImportProgress};
use crate::catalog::import;

#[tauri::command]
pub async fn import_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<ImageRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    import::import_folder(&db, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_images(
    state: State<'_, AppState>,
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<Vec<ImageRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(100);
    let sort_by = sort_by.unwrap_or_else(|| "date_taken".to_string());
    let sort_order = sort_order.unwrap_or_else(|| "DESC".to_string());
    crate::catalog::queries::get_images(&db, offset, limit, &sort_by, &sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_images(
    state: State<'_, AppState>,
    query: String,
    rating_min: Option<u8>,
    color_label: Option<String>,
    flag: Option<String>,
) -> Result<Vec<ImageRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::search::search_images(&db, &query, rating_min, color_label.as_deref(), flag.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_rating(
    state: State<'_, AppState>,
    image_id: String,
    rating: u8,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::set_rating(&db, &image_id, rating).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_color_label(
    state: State<'_, AppState>,
    image_id: String,
    color_label: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::set_color_label(&db, &image_id, &color_label).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_flag(
    state: State<'_, AppState>,
    image_id: String,
    flag: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::set_flag(&db, &image_id, &flag).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tags(
    state: State<'_, AppState>,
    image_id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::add_tags(&db, &image_id, &tags).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag(
    state: State<'_, AppState>,
    image_id: String,
    tag: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::remove_tag(&db, &image_id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<CollectionRecord, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::create_collection(&db, &name, parent_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_collection(
    state: State<'_, AppState>,
    collection_id: String,
    image_ids: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::add_to_collection(&db, &collection_id, &image_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collections(
    state: State<'_, AppState>,
) -> Result<Vec<CollectionRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::get_collections(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_images(
    state: State<'_, AppState>,
    image_ids: Vec<String>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::delete_images(&db, &image_ids).map_err(|e| e.to_string())
}
