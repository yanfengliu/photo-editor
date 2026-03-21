use rayon::prelude::*;

use super::pipeline::PARALLEL_PIXEL_THRESHOLD;

/// Apply unsharp mask sharpening (spatial operation, needs full buffer)
pub(crate) fn apply_sharpening_cpu(data: &mut Vec<u8>, width: u32, height: u32, amount: f32, radius: f32) {
    let w = width as usize;
    let h = height as usize;
    let r = radius.ceil() as i32;
    let strength = amount / 100.0;
    let sigma_sq = radius * radius;
    let source = data.clone();

    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![0u8; w * 4];
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let cr = source[idx] as f32 / 255.0;
                let cg = source[idx + 1] as f32 / 255.0;
                let cb = source[idx + 2] as f32 / 255.0;

                let mut blur_r = 0.0f32;
                let mut blur_g = 0.0f32;
                let mut blur_b = 0.0f32;
                let mut wt_sum = 0.0f32;

                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = ((dx * dx + dy * dy) as f32).sqrt();
                        if d > radius { continue; }
                        let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        let wt = (-d * d / (2.0 * sigma_sq)).exp();
                        blur_r += source[si] as f32 / 255.0 * wt;
                        blur_g += source[si + 1] as f32 / 255.0 * wt;
                        blur_b += source[si + 2] as f32 / 255.0 * wt;
                        wt_sum += wt;
                    }
                }

                blur_r /= wt_sum;
                blur_g /= wt_sum;
                blur_b /= wt_sum;

                let oi = x * 4;
                row[oi] = ((cr + (cr - blur_r) * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 1] = ((cg + (cg - blur_g) * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 2] = ((cb + (cb - blur_b) * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 3] = source[idx + 3];
            }
            row
        })
        .collect();

    for (y, row) in rows.into_iter().enumerate() {
        data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
    }
}

/// Apply clarity (large-radius local contrast enhancement with Gaussian blur and halo control)
pub(crate) fn apply_clarity_cpu(data: &mut Vec<u8>, width: u32, height: u32, clarity: f32) {
    let w = width as usize;
    let h = height as usize;
    let base_strength = clarity / 100.0;
    let sigma = 10.0_f32;
    let sigma_sq = sigma * sigma;
    let radius: i32 = 20;
    let step: i32 = 3; // sub-sample for performance
    let halo_limit = 0.15_f32;
    let source = data.clone();

    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![0u8; w * 4];
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let cr = source[idx] as f32 / 255.0;
                let cg = source[idx + 1] as f32 / 255.0;
                let cb = source[idx + 2] as f32 / 255.0;

                // Gaussian blur
                let mut blur_r = 0.0f32;
                let mut blur_g = 0.0f32;
                let mut blur_b = 0.0f32;
                let mut wt_sum = 0.0f32;

                let mut dy = -radius;
                while dy <= radius {
                    let mut dx = -radius;
                    while dx <= radius {
                        let d_sq = (dx * dx + dy * dy) as f32;
                        let wt = (-d_sq / (2.0 * sigma_sq)).exp();
                        let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        blur_r += source[si] as f32 / 255.0 * wt;
                        blur_g += source[si + 1] as f32 / 255.0 * wt;
                        blur_b += source[si + 2] as f32 / 255.0 * wt;
                        wt_sum += wt;
                        dx += step;
                    }
                    dy += step;
                }

                blur_r /= wt_sum;
                blur_g /= wt_sum;
                blur_b /= wt_sum;

                // Detail with halo suppression
                let dr = (cr - blur_r).clamp(-halo_limit, halo_limit);
                let dg = (cg - blur_g).clamp(-halo_limit, halo_limit);
                let db = (cb - blur_b).clamp(-halo_limit, halo_limit);

                // Midtone-weighted: apply more to midtones, less to extremes
                let lum = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                let midtone_w = 1.0 - 2.0 * (lum - 0.5).abs();
                let strength = base_strength * (0.5 + midtone_w * 0.5);

                let oi = x * 4;
                row[oi] = ((cr + dr * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 1] = ((cg + dg * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 2] = ((cb + db * strength).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 3] = source[idx + 3];
            }
            row
        })
        .collect();

    for (y, row) in rows.into_iter().enumerate() {
        data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
    }
}

/// Apply bilateral denoise (edge-preserving noise reduction)
pub(crate) fn apply_denoise_cpu(data: &mut Vec<u8>, width: u32, height: u32, lum_strength: f32, color_strength: f32) {
    let w = width as usize;
    let h = height as usize;
    let strength = lum_strength.max(color_strength);
    let spatial_sigma = 3.0 + strength / 20.0;
    let range_sigma = 0.05 + strength / 500.0;
    let radius = (spatial_sigma * 2.0).ceil() as i32;
    let spatial_sigma_sq = spatial_sigma * spatial_sigma;
    let range_sigma_sq = range_sigma * range_sigma;
    let source = data.clone();

    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![0u8; w * 4];
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let cr = source[idx] as f32 / 255.0;
                let cg = source[idx + 1] as f32 / 255.0;
                let cb = source[idx + 2] as f32 / 255.0;

                let mut sum_r = 0.0f32;
                let mut sum_g = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut wt_sum = 0.0f32;

                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        let nr = source[si] as f32 / 255.0;
                        let ng = source[si + 1] as f32 / 255.0;
                        let nb = source[si + 2] as f32 / 255.0;

                        let spatial_dist = (dx * dx + dy * dy) as f32;
                        let spatial_w = (-spatial_dist / (2.0 * spatial_sigma_sq)).exp();

                        let dr = nr - cr;
                        let dg = ng - cg;
                        let db = nb - cb;
                        let color_dist = dr * dr + dg * dg + db * db;
                        let range_w = (-color_dist / (2.0 * range_sigma_sq)).exp();

                        let wt = spatial_w * range_w;
                        sum_r += nr * wt;
                        sum_g += ng * wt;
                        sum_b += nb * wt;
                        wt_sum += wt;
                    }
                }

                let oi = x * 4;
                row[oi] = ((sum_r / wt_sum).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 1] = ((sum_g / wt_sum).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 2] = ((sum_b / wt_sum).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 3] = source[idx + 3];
            }
            row
        })
        .collect();

    for (y, row) in rows.into_iter().enumerate() {
        data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
    }
}

/// Apply dehaze using dark channel prior (He et al. 2009).
///
/// Positive strength removes haze, negative adds atmospheric haze.
/// Steps: 1) compute local dark channel, 2) estimate atmospheric light,
/// 3) compute transmission, 4) recover scene radiance.
pub(crate) fn apply_dehaze_cpu(data: &mut Vec<u8>, width: u32, height: u32, dehaze: f32) {
    let w = width as usize;
    let h = height as usize;
    let omega = dehaze / 100.0; // [-1, 1]
    let patch_radius: i32 = 7; // 15x15 patch (standard in paper)
    let source = data.clone();

    // Step 1: Compute dark channel — min(R,G,B) over local patch
    let dark_channel: Vec<f32> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0.0f32; w];
            for x in 0..w {
                let mut dark = 1.0f32;
                for dy in -patch_radius..=patch_radius {
                    let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                    for dx in -patch_radius..=patch_radius {
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        let r = source[si] as f32 / 255.0;
                        let g = source[si + 1] as f32 / 255.0;
                        let b = source[si + 2] as f32 / 255.0;
                        dark = dark.min(r.min(g).min(b));
                    }
                }
                row[x] = dark;
            }
            row
        })
        .collect();

    // Step 2: Estimate atmospheric light — average of the brightest 0.1% pixels
    // in the dark channel (He et al. use top 0.1%, then pick brightest in original)
    let num_pixels = w * h;
    let top_count = (num_pixels as f32 * 0.001).max(1.0) as usize;
    let mut dc_indexed: Vec<(usize, f32)> = dark_channel.iter().copied().enumerate().collect();
    dc_indexed.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let (mut atm_r, mut atm_g, mut atm_b) = (0.0f32, 0.0f32, 0.0f32);
    for &(idx, _) in dc_indexed.iter().take(top_count) {
        let si = idx * 4;
        atm_r += source[si] as f32 / 255.0;
        atm_g += source[si + 1] as f32 / 255.0;
        atm_b += source[si + 2] as f32 / 255.0;
    }
    let n = top_count as f32;
    let atm = [atm_r / n, atm_g / n, atm_b / n];

    // Step 3 + 4: Compute transmission and recover radiance (per-pixel)
    let t_min = 0.1_f32; // prevent division by near-zero

    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![0u8; w * 4];
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let dc_idx = y * w + x;
                let r = source[idx] as f32 / 255.0;
                let g = source[idx + 1] as f32 / 255.0;
                let b = source[idx + 2] as f32 / 255.0;

                // Transmission: t = 1 - omega * dark_channel / A
                // Use per-channel atmospheric normalization for dark channel
                let dc_norm = dark_channel[dc_idx] / atm[0].max(atm[1]).max(atm[2]).max(0.01);
                let transmission = (1.0 - omega * dc_norm).max(t_min);

                // Recover radiance: J = (I - A) / t + A
                let out_r = (r - atm[0]) / transmission + atm[0];
                let out_g = (g - atm[1]) / transmission + atm[1];
                let out_b = (b - atm[2]) / transmission + atm[2];

                let oi = x * 4;
                row[oi] = (out_r.clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 1] = (out_g.clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 2] = (out_b.clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 3] = source[idx + 3];
            }
            row
        })
        .collect();

    for (y, row) in rows.into_iter().enumerate() {
        data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
    }
}

/// Apply vignette effect (coordinate-based)
pub(crate) fn apply_vignette_cpu(data: &mut [u8], width: u32, height: u32, amount: f32) {
    let w = width as usize;
    let h = height as usize;
    let strength = amount / 100.0;

    let process_row = |y: usize, row: &mut [u8]| {
        let vy = (y as f32 / h as f32) * 2.0 - 1.0;
        for x in 0..w {
            let vx = (x as f32 / w as f32) * 2.0 - 1.0;
            let dist = (vx * vx + vy * vy).sqrt();

            // smoothstep(0.3, 1.2, dist)
            let t = ((dist - 0.3) / (1.2 - 0.3)).clamp(0.0, 1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            let vignette = (1.0 - strength * smooth).max(0.0);

            let i = x * 4;
            row[i] = ((row[i] as f32 / 255.0 * vignette).clamp(0.0, 1.0) * 255.0) as u8;
            row[i + 1] = ((row[i + 1] as f32 / 255.0 * vignette).clamp(0.0, 1.0) * 255.0) as u8;
            row[i + 2] = ((row[i + 2] as f32 / 255.0 * vignette).clamp(0.0, 1.0) * 255.0) as u8;
        }
    };

    if w * h >= PARALLEL_PIXEL_THRESHOLD {
        data.par_chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(|(y, row)| process_row(y, row));
    } else {
        data.chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(|(y, row)| process_row(y, row));
    }
}

/// Hash function for grain noise (matches shader)
fn grain_hash(x: f32, y: f32) -> f32 {
    let mut p3x = (x * 0.1031).fract();
    let mut p3y = (y * 0.1031).fract();
    let mut p3z = (x * 0.1031).fract(); // p.xyx pattern
    let dot = p3x * (p3y + 33.33) + p3y * (p3z + 33.33) + p3z * (p3x + 33.33);
    p3x += dot;
    p3y += dot;
    p3z += dot;
    ((p3x + p3y) * p3z).fract()
}

/// Apply film grain effect (coordinate-based)
pub(crate) fn apply_grain_cpu(data: &mut [u8], width: u32, height: u32, amount: f32, size: f32) {
    let w = width as usize;
    let h = height as usize;
    let strength = amount / 200.0;
    let grain_scale = (size / 25.0).max(0.5);

    let process_row = |y: usize, row: &mut [u8]| {
        for x in 0..w {
            let gx = x as f32 / grain_scale;
            let gy = y as f32 / grain_scale;
            let noise = grain_hash(gx, gy) * 2.0 - 1.0;

            let i = x * 4;
            let r = row[i] as f32 / 255.0;
            let g = row[i + 1] as f32 / 255.0;
            let b = row[i + 2] as f32 / 255.0;
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let midtone_weight = 1.0 - (lum - 0.5).abs() * 2.0;
            let grain = noise * strength * (0.5 + midtone_weight * 0.5);

            row[i] = ((r + grain).clamp(0.0, 1.0) * 255.0) as u8;
            row[i + 1] = ((g + grain).clamp(0.0, 1.0) * 255.0) as u8;
            row[i + 2] = ((b + grain).clamp(0.0, 1.0) * 255.0) as u8;
        }
    };

    if w * h >= PARALLEL_PIXEL_THRESHOLD {
        data.par_chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(|(y, row)| process_row(y, row));
    } else {
        data.chunks_exact_mut(w * 4)
            .enumerate()
            .for_each(|(y, row)| process_row(y, row));
    }
}
