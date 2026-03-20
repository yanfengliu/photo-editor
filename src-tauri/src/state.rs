use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::catalog::db::Database;
use crate::catalog::models::EditParams;
use crate::gpu::context::GpuContext;

#[derive(Clone)]
pub struct CachedPreview {
    pub data: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
}

pub struct AppState {
    pub db: Mutex<Database>,
    pub gpu: Mutex<Option<GpuContext>>,
    pub clipboard_edits: Mutex<Option<String>>,
    pub preview_cache: Mutex<HashMap<String, CachedPreview>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::new()?;

        let gpu = match pollster::block_on(GpuContext::new()) {
            Ok(ctx) => Some(ctx),
            Err(e) => {
                log::warn!("GPU initialization failed: {}. Falling back to CPU.", e);
                None
            }
        };

        Ok(Self {
            db: Mutex::new(db),
            gpu: Mutex::new(gpu),
            clipboard_edits: Mutex::new(None),
            preview_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Look up the file path for an image by its ID.
    pub fn get_image_file_path(&self, image_id: &str) -> Result<String, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let record = crate::catalog::queries::get_image_by_id(&db, image_id)
            .map_err(|e| e.to_string())?;
        Ok(record.file_path.clone())
    }

    /// Look up the stored edit params for an image, falling back to defaults.
    pub fn get_image_edit_params(&self, image_id: &str) -> Result<EditParams, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let record = crate::catalog::queries::get_image_by_id(&db, image_id)
            .map_err(|e| e.to_string())?;
        match record.edit_params {
            Some(ref json) => serde_json::from_str(json).map_err(|e| e.to_string()),
            None => Ok(EditParams::default()),
        }
    }
}
