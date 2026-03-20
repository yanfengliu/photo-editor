use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::catalog::db::Database;
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
}
