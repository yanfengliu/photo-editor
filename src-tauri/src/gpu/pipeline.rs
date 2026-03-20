use crate::catalog::models::EditParams;

pub fn apply_edits_cpu(rgba_data: &[u8], params: &EditParams) -> Vec<u8> {
    let mut result = rgba_data.to_vec();
    let pixel_count = result.len() / 4;
    for i in 0..pixel_count {
        let idx = i * 4;
        let mut r = result[idx] as f32 / 255.0;
        let mut g = result[idx + 1] as f32 / 255.0;
        let mut b = result[idx + 2] as f32 / 255.0;
        let a = result[idx + 3];

        // White balance
        let temp_shift = (params.temperature - 6500.0) / 6500.0;
        r *= 1.0 + temp_shift * 0.1;
        b *= 1.0 - temp_shift * 0.1;
        g *= 1.0 + params.tint / 150.0 * 0.05;

        // Exposure
        let exp = (2.0_f32).powf(params.exposure);
        r *= exp; g *= exp; b *= exp;

        // Contrast
        let cf = 1.0 + params.contrast / 100.0;
        r = (r - 0.5) * cf + 0.5;
        g = (g - 0.5) * cf + 0.5;
        b = (b - 0.5) * cf + 0.5;

        // Highlights/Shadows
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if lum > 0.5 {
            let f = 1.0 + params.highlights / 200.0;
            let blend = (lum - 0.5) * 2.0;
            r = r * (1.0 - blend) + r * f * blend;
            g = g * (1.0 - blend) + g * f * blend;
            b = b * (1.0 - blend) + b * f * blend;
        } else {
            let f = 1.0 + params.shadows / 200.0;
            let blend = (0.5 - lum) * 2.0;
            r = r * (1.0 - blend) + r * f * blend;
            g = g * (1.0 - blend) + g * f * blend;
            b = b * (1.0 - blend) + b * f * blend;
        }

        // Saturation
        let sf = 1.0 + params.saturation / 100.0;
        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        r = gray + (r - gray) * sf;
        g = gray + (g - gray) * sf;
        b = gray + (b - gray) * sf;

        // Vibrance
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let cur_sat = if max_c > 0.0 { (max_c - min_c) / max_c } else { 0.0 };
        let vf = 1.0 + params.vibrance / 100.0 * (1.0 - cur_sat);
        r = gray + (r - gray) * vf;
        g = gray + (g - gray) * vf;
        b = gray + (b - gray) * vf;

        // Dehaze
        if params.dehaze.abs() > 0.01 {
            let ds = params.dehaze / 100.0;
            let atm = min_c;
            r += (r - atm) * ds;
            g += (g - atm) * ds;
            b += (b - atm) * ds;
        }

        result[idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
        result[idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
        result[idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        result[idx + 3] = a;
    }
    result
}
