use rayon::prelude::*;
use super::lens_profiles::{interpolate_focal, DistortionCoeffs, CaCoeffs, VignetteCoeffs, LensProfile};

/// Apply lens corrections to an RGBA image buffer.
/// This should run BEFORE color/tone adjustments in the pipeline.
pub fn apply_lens_correction(
    data: &[u8],
    width: u32,
    height: u32,
    profile: &LensProfile,
    focal_length: f64,
    correct_distortion: bool,
    correct_ca: bool,
    correct_vignette: bool,
    amount: f64, // 0..200, where 100 = full correction
) -> Vec<u8> {
    let fp = interpolate_focal(profile, focal_length);
    let scale = amount / 100.0;

    let w = width as usize;
    let h = height as usize;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    // Normalize radius so corner = 1.0
    let r_norm = (cx * cx + cy * cy).sqrt();

    let mut output = vec![0u8; w * h * 4];

    // Scale PTLens coefficients by amount
    let scaled_dist = if correct_distortion {
        Some(DistortionCoeffs {
            a: fp.distortion.a * scale,
            b: fp.distortion.b * scale,
            c: fp.distortion.c * scale,
        })
    } else {
        None
    };

    let scaled_ca = if correct_ca {
        Some(CaCoeffs {
            red_scale: 1.0 + (fp.ca.red_scale - 1.0) * scale,
            blue_scale: 1.0 + (fp.ca.blue_scale - 1.0) * scale,
        })
    } else {
        None
    };

    let scaled_vig = if correct_vignette {
        Some(VignetteCoeffs {
            v1: fp.vignette.v1 * scale,
            v2: fp.vignette.v2 * scale,
            v3: fp.vignette.v3 * scale,
        })
    } else {
        None
    };

    let process_row = |(y, row): (usize, &mut [u8])| {
        for x in 0..w {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let r = (dx * dx + dy * dy).sqrt() / r_norm;
            let r2 = r * r;
            let r4 = r2 * r2;
            let r6 = r4 * r2;

            // --- PTLens distortion model ---
            // For each output (undistorted) pixel, find source in distorted input.
            // PTLens forward: Rd = a*Ru^4 + b*Ru^3 + c*Ru^2 + d*Ru where d = 1-a-b-c
            // distort_factor = Rd/Ru = a*Ru^3 + b*Ru^2 + c*Ru + d
            let (src_r, src_g, src_b) = if let Some(ref dist) = scaled_dist {
                let d = 1.0 - dist.a - dist.b - dist.c;
                let distort_factor = d + dist.c * r + dist.b * r2 + dist.a * r * r2;

                if let Some(ref ca) = scaled_ca {
                    // Per-channel radial scaling for CA correction
                    let r_factor = distort_factor * ca.red_scale;
                    let g_factor = distort_factor;
                    let b_factor = distort_factor * ca.blue_scale;

                    let sr = sample_channel(data, w, h, cx, cy, dx, dy, r_factor, 0);
                    let sg = sample_channel(data, w, h, cx, cy, dx, dy, g_factor, 1);
                    let sb = sample_channel(data, w, h, cx, cy, dx, dy, b_factor, 2);
                    (sr, sg, sb)
                } else {
                    let sx = cx + dx * distort_factor;
                    let sy = cy + dy * distort_factor;
                    let rgb = sample_bilinear(data, w, h, sx, sy);
                    (rgb[0], rgb[1], rgb[2])
                }
            } else if let Some(ref ca) = scaled_ca {
                // CA only, no distortion
                let sr = sample_channel(data, w, h, cx, cy, dx, dy, ca.red_scale, 0);
                let sg = sample_channel(data, w, h, cx, cy, dx, dy, 1.0, 1);
                let sb = sample_channel(data, w, h, cx, cy, dx, dy, ca.blue_scale, 2);
                (sr, sg, sb)
            } else {
                // No geometric correction, just copy
                let idx = (y * w + x) * 4;
                (data[idx], data[idx + 1], data[idx + 2])
            };

            // --- Vignette compensation ---
            let (final_r, final_g, final_b) = if let Some(ref vig) = scaled_vig {
                // Vignette polynomial: models light falloff.
                // To compensate, divide by the falloff to restore brightness.
                let falloff = 1.0 + vig.v1 * r2 + vig.v2 * r4 + vig.v3 * r6;
                let gain = if falloff.abs() > 0.01 { 1.0 / falloff } else { 1.0 };
                (
                    (src_r as f64 * gain).clamp(0.0, 255.0) as u8,
                    (src_g as f64 * gain).clamp(0.0, 255.0) as u8,
                    (src_b as f64 * gain).clamp(0.0, 255.0) as u8,
                )
            } else {
                (src_r, src_g, src_b)
            };

            let idx = x * 4;
            row[idx] = final_r;
            row[idx + 1] = final_g;
            row[idx + 2] = final_b;
            row[idx + 3] = data[(y * w + x) * 4 + 3]; // preserve alpha
        }
    };

    let pixel_count = w * h;
    if pixel_count >= 512 * 512 {
        output
            .par_chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(process_row);
    } else {
        output
            .chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(process_row);
    }

    output
}

/// Sample a single color channel with radial scaling (for CA correction)
fn sample_channel(
    data: &[u8],
    w: usize,
    h: usize,
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
    factor: f64,
    channel: usize,
) -> u8 {
    let sx = cx + dx * factor;
    let sy = cy + dy * factor;

    let x0 = sx.floor() as i64;
    let y0 = sy.floor() as i64;
    let fx = sx - x0 as f64;
    let fy = sy - y0 as f64;

    let get = |xi: i64, yi: i64| -> f64 {
        let xi = xi.clamp(0, w as i64 - 1) as usize;
        let yi = yi.clamp(0, h as i64 - 1) as usize;
        data[(yi * w + xi) * 4 + channel] as f64
    };

    let v = get(x0, y0) * (1.0 - fx) * (1.0 - fy)
        + get(x0 + 1, y0) * fx * (1.0 - fy)
        + get(x0, y0 + 1) * (1.0 - fx) * fy
        + get(x0 + 1, y0 + 1) * fx * fy;

    v.clamp(0.0, 255.0) as u8
}

/// Bilinear interpolation sampling all RGB channels at once
fn sample_bilinear(data: &[u8], w: usize, h: usize, sx: f64, sy: f64) -> [u8; 3] {
    let x0 = sx.floor() as i64;
    let y0 = sy.floor() as i64;
    let fx = sx - x0 as f64;
    let fy = sy - y0 as f64;

    let get = |xi: i64, yi: i64| -> [f64; 3] {
        let xi = xi.clamp(0, w as i64 - 1) as usize;
        let yi = yi.clamp(0, h as i64 - 1) as usize;
        let idx = (yi * w + xi) * 4;
        [data[idx] as f64, data[idx + 1] as f64, data[idx + 2] as f64]
    };

    let tl = get(x0, y0);
    let tr = get(x0 + 1, y0);
    let bl = get(x0, y0 + 1);
    let br = get(x0 + 1, y0 + 1);

    let mut result = [0u8; 3];
    for c in 0..3 {
        let v = tl[c] * (1.0 - fx) * (1.0 - fy)
            + tr[c] * fx * (1.0 - fy)
            + bl[c] * (1.0 - fx) * fy
            + br[c] * fx * fy;
        result[c] = v.clamp(0.0, 255.0) as u8;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::lens_profiles::get_all_profiles;

    fn make_gradient(w: usize, h: usize) -> Vec<u8> {
        let mut data = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                data[idx] = (x * 255 / w) as u8;
                data[idx + 1] = (y * 255 / h) as u8;
                data[idx + 2] = 128;
                data[idx + 3] = 255;
            }
        }
        data
    }

    fn find_any_profile() -> &'static LensProfile {
        get_all_profiles()
            .iter()
            .find(|p| !p.profiles.is_empty())
            .expect("Need at least one lens profile with calibration data")
    }

    #[test]
    fn test_zero_amount_is_identity() {
        let w = 64;
        let h = 64;
        let data = vec![128u8; w * h * 4];
        let profile = find_any_profile();
        let focal = profile.profiles[0].focal_length;
        let result = apply_lens_correction(&data, w as u32, h as u32, profile, focal, true, true, true, 0.0);
        // With amount=0, d=1 (identity), so center pixels should be unchanged
        let center = (h / 2 * w + w / 2) * 4;
        assert_eq!(result[center], data[center]);
    }

    #[test]
    fn test_correction_changes_pixels() {
        let w = 128;
        let h = 128;
        let data = make_gradient(w, h);
        let profile = find_any_profile();
        let focal = profile.profiles[0].focal_length;
        let result = apply_lens_correction(&data, w as u32, h as u32, profile, focal, true, false, false, 100.0);
        // Edge pixels should shift with distortion correction
        let mut differs = false;
        for i in (0..w * 4).step_by(4) {
            if result[i] != data[i] || result[i + 1] != data[i + 1] {
                differs = true;
                break;
            }
        }
        assert!(differs, "Distortion correction should change edge pixels");
    }

    #[test]
    fn test_no_corrections_enabled() {
        let w = 32;
        let h = 32;
        let data = vec![100u8; w * h * 4];
        let profile = find_any_profile();
        let focal = profile.profiles[0].focal_length;
        let result = apply_lens_correction(&data, w as u32, h as u32, profile, focal, false, false, false, 100.0);
        assert_eq!(result, data, "No corrections should produce identical output");
    }

    #[test]
    fn test_alpha_preserved() {
        let w = 32;
        let h = 32;
        let mut data = vec![128u8; w * h * 4];
        for i in (0..data.len()).step_by(4) {
            data[i + 3] = 200;
        }
        let profile = find_any_profile();
        let focal = profile.profiles[0].focal_length;
        let result = apply_lens_correction(&data, w as u32, h as u32, profile, focal, true, true, true, 100.0);
        for i in (0..result.len()).step_by(4) {
            assert_eq!(result[i + 3], 200, "Alpha should be preserved");
        }
    }
}
