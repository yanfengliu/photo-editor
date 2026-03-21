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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    // Lens Correction
    #[serde(default)]
    pub enable_lens_correction: bool,
    #[serde(default)]
    pub lens_profile_id: Option<String>,
    #[serde(default)]
    pub lens_distortion: f32,
    #[serde(default = "default_true")]
    pub lens_ca_correction: bool,
    #[serde(default = "default_true")]
    pub lens_vignette_correction: bool,
    #[serde(default = "default_lens_amount")]
    pub lens_distortion_amount: f32,
}

fn default_true() -> bool { true }
fn default_lens_amount() -> f32 { 100.0 }

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
            enable_lens_correction: false,
            lens_profile_id: None,
            lens_distortion: 0.0,
            lens_ca_correction: true,
            lens_vignette_correction: true,
            lens_distortion_amount: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_params_default() {
        let params = EditParams::default();
        assert_eq!(params.exposure, 0.0);
        assert_eq!(params.contrast, 0.0);
        assert_eq!(params.temperature, 6500.0);
        assert_eq!(params.tint, 0.0);
        assert_eq!(params.saturation, 0.0);
        assert_eq!(params.vibrance, 0.0);
        assert_eq!(params.sharpening_amount, 0.0);
        assert_eq!(params.sharpening_radius, 1.0);
        assert_eq!(params.denoise_luminance, 0.0);
        assert_eq!(params.denoise_color, 0.0);
        assert!(!params.denoise_ai);
        assert_eq!(params.vignette_amount, 0.0);
        assert_eq!(params.grain_amount, 0.0);
        assert_eq!(params.grain_size, 25.0);
        assert_eq!(params.dehaze, 0.0);
        assert_eq!(params.clarity, 0.0);
    }

    #[test]
    fn test_edit_params_default_curves() {
        let params = EditParams::default();
        assert_eq!(params.curve_rgb.len(), 2);
        assert_eq!(params.curve_rgb[0].x, 0.0);
        assert_eq!(params.curve_rgb[0].y, 0.0);
        assert_eq!(params.curve_rgb[1].x, 1.0);
        assert_eq!(params.curve_rgb[1].y, 1.0);
        assert_eq!(params.curve_r, params.curve_rgb);
        assert_eq!(params.curve_g, params.curve_rgb);
        assert_eq!(params.curve_b, params.curve_rgb);
    }

    #[test]
    fn test_edit_params_default_hsl() {
        let params = EditParams::default();
        assert_eq!(params.hsl_hue, [0.0; 8]);
        assert_eq!(params.hsl_saturation, [0.0; 8]);
        assert_eq!(params.hsl_luminance, [0.0; 8]);
    }

    #[test]
    fn test_edit_params_serialization() {
        let params = EditParams::default();
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: EditParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exposure, params.exposure);
        assert_eq!(deserialized.temperature, params.temperature);
        assert_eq!(deserialized.curve_rgb.len(), params.curve_rgb.len());
    }

    #[test]
    fn test_edit_params_custom_serialization() {
        let mut params = EditParams::default();
        params.exposure = 2.5;
        params.contrast = -30.0;
        params.temperature = 4000.0;
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: EditParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exposure, 2.5);
        assert_eq!(deserialized.contrast, -30.0);
        assert_eq!(deserialized.temperature, 4000.0);
    }

    #[test]
    fn test_image_record_serialization() {
        let record = ImageRecord {
            id: "test-id".to_string(),
            file_path: "/photos/test.jpg".to_string(),
            file_name: "test.jpg".to_string(),
            format: "jpeg".to_string(),
            width: 1920,
            height: 1080,
            date_taken: Some("2024-01-15".to_string()),
            rating: 5,
            color_label: "red".to_string(),
            flag: "picked".to_string(),
            camera: Some("Canon EOS R5".to_string()),
            lens: Some("RF 24-70mm".to_string()),
            iso: Some(400),
            focal_length: Some(50.0),
            aperture: Some(2.8),
            shutter_speed: Some("1/250".to_string()),
            edit_params: None,
            tags: vec!["landscape".to_string()],
            created_at: "2024-01-15T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Canon EOS R5"));
    }

    #[test]
    fn test_collection_record() {
        let col = CollectionRecord {
            id: "col-1".to_string(),
            name: "Favorites".to_string(),
            parent_id: None,
            image_count: 42,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(col.name, "Favorites");
        assert_eq!(col.image_count, 42);
        assert!(col.parent_id.is_none());
    }

    #[test]
    fn test_history_entry() {
        let entry = HistoryEntry {
            id: "h-1".to_string(),
            image_id: "img-1".to_string(),
            action: "edit".to_string(),
            params_json: "{}".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(entry.action, "edit");
    }

    #[test]
    fn test_curve_point() {
        let pt = CurvePoint { x: 0.5, y: 0.7 };
        assert_eq!(pt.x, 0.5);
        assert_eq!(pt.y, 0.7);
        let json = serde_json::to_string(&pt).unwrap();
        let deserialized: CurvePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.x, 0.5);
        assert_eq!(deserialized.y, 0.7);
    }
}
