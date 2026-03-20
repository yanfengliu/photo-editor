use std::sync::Mutex;
use crate::catalog::db::Database;
use crate::gpu::context::GpuContext;

pub struct AppState {
    pub db: Mutex<Database>,
    pub gpu: Mutex<Option<GpuContext>>,
    pub clipboard_edits: Mutex<Option<String>>,
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
        })
    }
}
