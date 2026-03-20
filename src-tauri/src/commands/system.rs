use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn browse_folder() -> Result<Option<String>, String> {
    // The actual folder browsing is handled by tauri-plugin-dialog on the frontend
    // This command is a placeholder if needed for server-side folder browsing
    Ok(None)
}

#[tauri::command]
pub async fn get_gpu_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let gpu = state.gpu.lock().map_err(|e| e.to_string())?;
    match &*gpu {
        Some(ctx) => Ok(serde_json::json!({
            "available": true,
            "adapter_name": ctx.adapter_name,
            "backend": ctx.backend,
        })),
        None => Ok(serde_json::json!({
            "available": false,
            "adapter_name": "None",
            "backend": "CPU fallback",
        })),
    }
}

#[tauri::command]
pub async fn get_app_config() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "proxy_resolution": 2048,
        "thumbnail_size": 256,
        "cache_size_mb": 512,
    }))
}

#[tauri::command]
pub async fn set_app_config(
    _config: serde_json::Value,
) -> Result<(), String> {
    // TODO: Persist config to file
    Ok(())
}
