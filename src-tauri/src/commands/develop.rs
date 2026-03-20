use tauri::State;
use crate::state::AppState;
use crate::catalog::models::EditParams;

#[derive(serde::Serialize)]
pub struct PreviewImagePayload {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

fn get_cached_preview(
    state: &AppState,
    image_id: &str,
    file_path: &str,
    max_size: u32,
) -> Result<crate::state::CachedPreview, String> {
    let cache_key = format!("{image_id}:{max_size}");

    if let Some(preview) = state
        .preview_cache
        .lock()
        .map_err(|e| e.to_string())?
        .get(&cache_key)
        .cloned()
    {
        return Ok(preview);
    }

    let preview = crate::imaging::loader::load_preview(file_path, max_size)
        .map_err(|e| e.to_string())?;
    let cached = crate::state::CachedPreview {
        data: preview.data.into(),
        width: preview.width,
        height: preview.height,
    };

    let mut cache = state.preview_cache.lock().map_err(|e| e.to_string())?;
    if cache.len() >= 8 {
        cache.clear();
    }
    cache.insert(cache_key, cached.clone());

    Ok(cached)
}

#[tauri::command]
pub async fn apply_edits(
    state: State<'_, AppState>,
    image_id: String,
    params: EditParams,
    preview_size: Option<u32>,
) -> Result<PreviewImagePayload, String> {
    // Load the image
    let file_path = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
            .map_err(|e| e.to_string())?;
        record.file_path.clone()
    };

    let max_size = preview_size.unwrap_or(2048);
    let preview = get_cached_preview(&state, &image_id, &file_path, max_size)?;

    let gpu = state.gpu.lock().map_err(|e| e.to_string())?;
    let result = crate::gpu::pipeline::apply_edits_with_backend(
        gpu.as_ref(),
        preview.data.as_ref(),
        preview.width,
        preview.height,
        &params,
    );

    Ok(PreviewImagePayload {
        data: result,
        width: preview.width,
        height: preview.height,
    })
}

#[tauri::command]
pub async fn save_edit_params(
    state: State<'_, AppState>,
    image_id: String,
    params: EditParams,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let params_json = serde_json::to_string(&params).map_err(|e| e.to_string())?;
    crate::catalog::queries::save_edit_params(&db, &image_id, &params_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_edit_params(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<EditParams, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;
    match record.edit_params {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None => Ok(EditParams::default()),
    }
}

#[tauri::command]
pub async fn reset_edits(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<EditParams, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let params = EditParams::default();
    let params_json = serde_json::to_string(&params).map_err(|e| e.to_string())?;
    crate::catalog::queries::save_edit_params(&db, &image_id, &params_json)
        .map_err(|e| e.to_string())?;
    Ok(params)
}

#[tauri::command]
pub async fn save_snapshot(
    state: State<'_, AppState>,
    image_id: String,
    name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::save_snapshot(&db, &image_id, &name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_snapshot(
    state: State<'_, AppState>,
    _image_id: String,
    snapshot_id: String,
) -> Result<EditParams, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let params_json = crate::catalog::queries::load_snapshot(&db, &snapshot_id)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&params_json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<Vec<crate::catalog::models::HistoryEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::get_history(&db, &image_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_edits(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;
    drop(db);
    let mut clipboard = state.clipboard_edits.lock().map_err(|e| e.to_string())?;
    *clipboard = record.edit_params;
    Ok(())
}

#[tauri::command]
pub async fn paste_edits(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<EditParams, String> {
    let clipboard = state.clipboard_edits.lock().map_err(|e| e.to_string())?;
    let params_json = clipboard.as_ref()
        .ok_or_else(|| "No edits in clipboard".to_string())?;
    let params: EditParams = serde_json::from_str(params_json)
        .map_err(|e| e.to_string())?;
    drop(clipboard);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    crate::catalog::queries::save_edit_params(&db, &image_id, &serde_json::to_string(&params).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(params)
}
