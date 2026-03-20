pub struct AiDenoiser;

impl AiDenoiser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> { Ok(Self) }

    pub fn denoise(&self, _rgba: &[u8], _w: u32, _h: u32, _strength: f32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Err("AI denoise not yet implemented. ONNX model required.".into())
    }
}
