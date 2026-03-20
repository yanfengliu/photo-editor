use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub date_taken: Option<String>,
    pub rating: u8,
    pub color_label: String,
    pub flag: String,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<f64>,
    pub aperture: Option<f64>,
    pub shutter_speed: Option<String>,
    pub edit_params: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub image_count: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub image_id: String,
    pub action: String,
    pub params_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub id: String,
    pub image_id: String,
    pub name: String,
    pub params_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub total: u32,
    pub processed: u32,
    pub current_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditParams {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub temperature: f32,
    pub tint: f32,
    pub saturation: f32,
    pub vibrance: f32,
    pub clarity: f32,
    pub dehaze: f32,
    pub sharpening_amount: f32,
    pub sharpening_radius: f32,
    pub denoise_luminance: f32,
    pub denoise_color: f32,
    pub denoise_ai: bool,
    pub vignette_amount: f32,
    pub grain_amount: f32,
    pub grain_size: f32,
    pub curve_rgb: Vec<CurvePoint>,
    pub curve_r: Vec<CurvePoint>,
    pub curve_g: Vec<CurvePoint>,
    pub curve_b: Vec<CurvePoint>,
    pub hsl_hue: [f32; 8],
    pub hsl_saturation: [f32; 8],
    pub hsl_luminance: [f32; 8],
}

impl Default for EditParams {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            temperature: 6500.0,
            tint: 0.0,
            saturation: 0.0,
            vibrance: 0.0,
            clarity: 0.0,
            dehaze: 0.0,
            sharpening_amount: 0.0,
            sharpening_radius: 1.0,
            denoise_luminance: 0.0,
            denoise_color: 0.0,
            denoise_ai: false,
            vignette_amount: 0.0,
            grain_amount: 0.0,
            grain_size: 25.0,
            curve_rgb: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
            curve_r: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
            curve_g: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
            curve_b: vec![
                CurvePoint { x: 0.0, y: 0.0 },
                CurvePoint { x: 1.0, y: 1.0 },
            ],
            hsl_hue: [0.0; 8],
            hsl_saturation: [0.0; 8],
            hsl_luminance: [0.0; 8],
        }
    }
}
