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

/// Apply clarity (large-radius local contrast enhancement)
pub(crate) fn apply_clarity_cpu(data: &mut Vec<u8>, width: u32, height: u32, clarity: f32) {
    let w = width as usize;
    let h = height as usize;
    let strength = clarity / 100.0;
    let radius: i32 = 8;
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

                // Box blur with step=2 (matching shader)
                let mut blur_r = 0.0f32;
                let mut blur_g = 0.0f32;
                let mut blur_b = 0.0f32;
                let mut count = 0.0f32;

                let mut dy = -radius;
                while dy <= radius {
                    let mut dx = -radius;
                    while dx <= radius {
                        let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        blur_r += source[si] as f32 / 255.0;
                        blur_g += source[si + 1] as f32 / 255.0;
                        blur_b += source[si + 2] as f32 / 255.0;
                        count += 1.0;
                        dx += 2;
                    }
                    dy += 2;
                }

                blur_r /= count;
                blur_g /= count;
                blur_b /= count;

                // Local contrast = original + (original - blur) * strength
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
