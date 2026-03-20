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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::models::EditParams;

    fn make_pixel(r: u8, g: u8, b: u8) -> Vec<u8> {
        vec![r, g, b, 255]
    }

    fn make_gray_image(value: u8, pixel_count: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&[value, value, value, 255]);
        }
        data
    }

    #[test]
    fn test_neutral_params_no_change() {
        let input = make_gray_image(128, 4);
        let params = EditParams::default();
        let output = apply_edits_cpu(&input, &params);
        // With neutral params, output should be very close to input
        for i in 0..4 {
            let idx = i * 4;
            let diff = (output[idx] as i32 - input[idx] as i32).abs();
            assert!(diff <= 1, "Pixel {} R: expected ~{}, got {} (diff {})", i, input[idx], output[idx], diff);
        }
    }

    #[test]
    fn test_exposure_brightens() {
        let input = make_gray_image(100, 1);
        let mut params = EditParams::default();
        params.exposure = 1.0; // +1 EV should roughly double brightness
        let output = apply_edits_cpu(&input, &params);
        assert!(output[0] > input[0], "Exposure +1 should brighten: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_exposure_darkens() {
        let input = make_gray_image(200, 1);
        let mut params = EditParams::default();
        params.exposure = -1.0;
        let output = apply_edits_cpu(&input, &params);
        assert!(output[0] < input[0], "Exposure -1 should darken: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_contrast_increase() {
        // Mid-gray should stay roughly the same
        let input = vec![128, 128, 128, 255];
        let mut params = EditParams::default();
        params.contrast = 50.0;
        let output = apply_edits_cpu(&input, &params);
        let diff = (output[0] as i32 - 128).abs();
        assert!(diff < 10, "Mid-gray with contrast should stay near 128, got {}", output[0]);
    }

    #[test]
    fn test_saturation_desaturate() {
        // Red pixel should become gray-ish with saturation = -100
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.saturation = -100.0;
        let output = apply_edits_cpu(&input, &params);
        // R, G, B should be roughly equal (gray)
        let max_diff = (output[0] as i32 - output[1] as i32).abs()
            .max((output[1] as i32 - output[2] as i32).abs());
        assert!(max_diff < 5, "Desaturated red should be near gray, got ({}, {}, {})", output[0], output[1], output[2]);
    }

    #[test]
    fn test_warm_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 10000.0; // Very warm
        let output = apply_edits_cpu(&input, &params);
        assert!(output[0] >= output[2], "Warm WB should shift red > blue: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_cool_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 3000.0; // Very cool
        let output = apply_edits_cpu(&input, &params);
        assert!(output[2] >= output[0], "Cool WB should shift blue > red: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_alpha_preserved() {
        let input = vec![100, 100, 100, 200]; // Semi-transparent
        let mut params = EditParams::default();
        params.exposure = 1.0;
        let output = apply_edits_cpu(&input, &params);
        assert_eq!(output[3], 200, "Alpha should be preserved");
    }

    #[test]
    fn test_output_clamped() {
        let input = vec![250, 250, 250, 255];
        let mut params = EditParams::default();
        params.exposure = 3.0; // Very bright - would exceed 255
        let output = apply_edits_cpu(&input, &params);
        assert!(output[0] <= 255, "Output should be clamped to 255");
        assert!(output[1] <= 255);
        assert!(output[2] <= 255);
    }

    #[test]
    fn test_dehaze_positive() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.dehaze = 50.0;
        let output = apply_edits_cpu(&input, &params);
        // Dehaze should increase contrast/visibility
        // Just verify it doesn't crash and produces valid output
        assert!(output[0] <= 255);
        assert!(output[3] == 255);
    }

    #[test]
    fn test_multiple_params() {
        let input = make_gray_image(100, 4);
        let mut params = EditParams::default();
        params.exposure = 0.5;
        params.contrast = 20.0;
        params.saturation = 10.0;
        params.vibrance = 10.0;
        let output = apply_edits_cpu(&input, &params);
        assert_eq!(output.len(), input.len());
        // Just verify all pixels are valid
        for i in 0..4 {
            assert!(output[i * 4 + 3] == 255);
        }
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<u8> = vec![];
        let params = EditParams::default();
        let output = apply_edits_cpu(&input, &params);
        assert!(output.is_empty());
    }
}
