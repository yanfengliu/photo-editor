use rayon::prelude::*;

use super::pipeline::PARALLEL_PIXEL_THRESHOLD;

/// Box filter (mean filter) over a 2D grayscale image.
/// radius defines the half-window size, so the full window is (2*radius+1)^2.
fn box_filter(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut temp = vec![0.0f32; w * h];
    let mut out = vec![0.0f32; w * h];
    // Horizontal pass
    for y in 0..h {
        let mut sum = 0.0f32;
        let mut count = 0u32;
        // Initialize window
        for x in 0..=radius.min(w - 1) {
            sum += src[y * w + x];
            count += 1;
        }
        for x in 0..w {
            // Add right edge
            let right = x + radius + 1;
            if right < w {
                sum += src[y * w + right];
                count += 1;
            }
            // Remove left edge
            if x > radius {
                sum -= src[y * w + x - radius - 1];
                count -= 1;
            }
            temp[y * w + x] = sum / count as f32;
        }
    }
    // Vertical pass
    for x in 0..w {
        let mut sum = 0.0f32;
        let mut count = 0u32;
        for y in 0..=radius.min(h - 1) {
            sum += temp[y * w + x];
            count += 1;
        }
        for y in 0..h {
            let bottom = y + radius + 1;
            if bottom < h {
                sum += temp[bottom * w + x];
                count += 1;
            }
            if y > radius {
                sum -= temp[(y - radius - 1) * w + x];
                count -= 1;
            }
            out[y * w + x] = sum / count as f32;
        }
    }
    out
}

/// Guided image filter (He, Sun, Tang — TPAMI 2013).
///
/// Filters `input` using `guide` as the guidance image. Preserves edges
/// present in the guide while smoothing the input.
///
/// - `guide`: guidance image (grayscale, same size as input)
/// - `input`: image to filter (grayscale, same size)
/// - `w`, `h`: dimensions
/// - `radius`: window radius for the box filter
/// - `eps`: regularization (larger = smoother)
///
/// Reference: He K., Sun J., Tang X., "Guided Image Filtering",
/// IEEE TPAMI 35(6), 1397–1409, 2013.
fn guided_filter(guide: &[f32], input: &[f32], w: usize, h: usize, radius: usize, eps: f32) -> Vec<f32> {
    let n = w * h;
    // Step 1: compute means
    let mean_i = box_filter(guide, w, h, radius);
    let mean_p = box_filter(input, w, h, radius);

    // Step 2: compute correlation and variance
    let mut ip = vec![0.0f32; n];
    let mut ii = vec![0.0f32; n];
    for k in 0..n {
        ip[k] = guide[k] * input[k];
        ii[k] = guide[k] * guide[k];
    }
    let mean_ip = box_filter(&ip, w, h, radius);
    let mean_ii = box_filter(&ii, w, h, radius);

    // Step 3: compute a and b
    let mut a = vec![0.0f32; n];
    let mut b = vec![0.0f32; n];
    for k in 0..n {
        let cov_ip = mean_ip[k] - mean_i[k] * mean_p[k];
        let var_i = mean_ii[k] - mean_i[k] * mean_i[k];
        a[k] = cov_ip / (var_i + eps);
        b[k] = mean_p[k] - a[k] * mean_i[k];
    }

    // Step 4: compute output
    let mean_a = box_filter(&a, w, h, radius);
    let mean_b = box_filter(&b, w, h, radius);
    let mut q = vec![0.0f32; n];
    for k in 0..n {
        q[k] = mean_a[k] * guide[k] + mean_b[k];
    }
    q
}

/// Apply unsharp mask sharpening (spatial operation, needs full buffer)
pub(crate) fn apply_sharpening_cpu(data: &mut [u8], width: u32, height: u32, amount: f32, radius: f32, detail: f32) {
    let w = width as usize;
    let h = height as usize;
    let r = radius.ceil() as i32;
    let strength = amount / 100.0;
    let threshold = (1.0 - detail / 100.0) * 0.1;
    let sigma_sq = radius * radius;
    let source = data.to_owned();

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
                // Standard unsharp mask with threshold (detail parameter)
                // Only sharpen where the difference exceeds the threshold
                let dr = cr - blur_r;
                let dg = cg - blur_g;
                let db = cb - blur_b;
                let apply_r = if dr.abs() > threshold { dr * strength } else { 0.0 };
                let apply_g = if dg.abs() > threshold { dg * strength } else { 0.0 };
                let apply_b = if db.abs() > threshold { db * strength } else { 0.0 };
                row[oi] = ((cr + apply_r).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 1] = ((cg + apply_g).clamp(0.0, 1.0) * 255.0) as u8;
                row[oi + 2] = ((cb + apply_b).clamp(0.0, 1.0) * 255.0) as u8;
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
pub(crate) fn apply_clarity_cpu(data: &mut [u8], width: u32, height: u32, clarity: f32) {
    let w = width as usize;
    let h = height as usize;
    let base_strength = clarity / 100.0;
    let sigma = 10.0_f32;
    let sigma_sq = sigma * sigma;
    let radius: i32 = 20;
    let step: i32 = 3; // sub-sample for performance
    let halo_limit = 0.15_f32;
    let source = data.to_owned();

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

/// Apply bilateral denoise with separate luma/chroma processing.
///
/// Professional noise reduction processes luminance and chrominance
/// independently (Tomasi & Manduchi 1998). The eye is less sensitive
/// to color noise, so chroma channels can be filtered more aggressively.
///
/// Reference: Tomasi C., Manduchi R., "Bilateral Filtering for Gray
/// and Color Images", ICCV 1998.
pub(crate) fn apply_denoise_cpu(data: &mut [u8], width: u32, height: u32, lum_strength: f32, color_strength: f32) {
    let w = width as usize;
    let h = height as usize;
    if lum_strength < 0.5 && color_strength < 0.5 {
        return;
    }
    let source = data.to_owned();

    // --- Luminance denoising ---
    if lum_strength >= 0.5 {
        let spatial_sigma = 3.0 + lum_strength / 20.0;
        let range_sigma = 0.05 + lum_strength / 500.0;
        let radius = (spatial_sigma * 2.0).ceil() as i32;
        let spatial_sigma_sq = spatial_sigma * spatial_sigma;
        let range_sigma_sq = range_sigma * range_sigma;

        let rows: Vec<Vec<u8>> = (0..h)
            .into_par_iter()
            .map(|y| {
                let mut row = vec![0u8; w * 4];
                for x in 0..w {
                    let idx = (y * w + x) * 4;
                    let cr = source[idx] as f32 / 255.0;
                    let cg = source[idx + 1] as f32 / 255.0;
                    let cb = source[idx + 2] as f32 / 255.0;
                    let center_y = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;

                    let mut sum_y = 0.0f32;
                    let mut wt_sum = 0.0f32;

                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                            let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                            let si = (sy * w + sx) * 4;
                            let nr = source[si] as f32 / 255.0;
                            let ng = source[si + 1] as f32 / 255.0;
                            let nb = source[si + 2] as f32 / 255.0;
                            let ny = 0.2126 * nr + 0.7152 * ng + 0.0722 * nb;

                            let spatial_dist = (dx * dx + dy * dy) as f32;
                            let spatial_w = (-spatial_dist / (2.0 * spatial_sigma_sq)).exp();
                            let luma_diff = ny - center_y;
                            let range_w = (-luma_diff * luma_diff / (2.0 * range_sigma_sq)).exp();

                            let wt = spatial_w * range_w;
                            sum_y += ny * wt;
                            wt_sum += wt;
                        }
                    }

                    let filtered_y = sum_y / wt_sum;
                    // Adjust RGB to match the filtered luminance while preserving color ratios
                    let ratio = if center_y > 0.001 { filtered_y / center_y } else { 1.0 };
                    let oi = x * 4;
                    row[oi] = ((cr * ratio).clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 1] = ((cg * ratio).clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 2] = ((cb * ratio).clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 3] = source[idx + 3];
                }
                row
            })
            .collect();

        for (y, row) in rows.into_iter().enumerate() {
            data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
        }
    }

    // --- Chroma denoising (use the luma-denoised data as input) ---
    if color_strength >= 0.5 {
        let chroma_source = data.to_owned();
        let spatial_sigma = 4.0 + color_strength / 15.0;
        // Larger range sigma for chroma — eye is less sensitive to color noise
        let range_sigma = 0.10 + color_strength / 200.0;
        let radius = (spatial_sigma * 2.0).ceil() as i32;
        let spatial_sigma_sq = spatial_sigma * spatial_sigma;
        let range_sigma_sq = range_sigma * range_sigma;

        let rows: Vec<Vec<u8>> = (0..h)
            .into_par_iter()
            .map(|y| {
                let mut row = vec![0u8; w * 4];
                for x in 0..w {
                    let idx = (y * w + x) * 4;
                    let cr = chroma_source[idx] as f32 / 255.0;
                    let cg = chroma_source[idx + 1] as f32 / 255.0;
                    let cb = chroma_source[idx + 2] as f32 / 255.0;
                    let center_y = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
                    let center_cb = cb - center_y;
                    let center_cr = cr - center_y;

                    let mut sum_cb = 0.0f32;
                    let mut sum_cr = 0.0f32;
                    let mut wt_sum = 0.0f32;

                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                            let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                            let si = (sy * w + sx) * 4;
                            let nr = chroma_source[si] as f32 / 255.0;
                            let ng = chroma_source[si + 1] as f32 / 255.0;
                            let nb = chroma_source[si + 2] as f32 / 255.0;
                            let ny = 0.2126 * nr + 0.7152 * ng + 0.0722 * nb;
                            let ncb = nb - ny;
                            let ncr = nr - ny;

                            let spatial_dist = (dx * dx + dy * dy) as f32;
                            let spatial_w = (-spatial_dist / (2.0 * spatial_sigma_sq)).exp();
                            let dcb = ncb - center_cb;
                            let dcr = ncr - center_cr;
                            let chroma_dist = dcb * dcb + dcr * dcr;
                            let range_w = (-chroma_dist / (2.0 * range_sigma_sq)).exp();

                            let wt = spatial_w * range_w;
                            sum_cb += ncb * wt;
                            sum_cr += ncr * wt;
                            wt_sum += wt;
                        }
                    }

                    let filtered_cb = sum_cb / wt_sum;
                    let filtered_cr = sum_cr / wt_sum;
                    // Reconstruct RGB: R = Y + Cr, G = (Y - 0.2126*R - 0.0722*B) / 0.7152, B = Y + Cb
                    let new_r = center_y + filtered_cr;
                    let new_b = center_y + filtered_cb;
                    let new_g = (center_y - 0.2126 * new_r - 0.0722 * new_b) / 0.7152;

                    let oi = x * 4;
                    row[oi] = (new_r.clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 1] = (new_g.clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 2] = (new_b.clamp(0.0, 1.0) * 255.0) as u8;
                    row[oi + 3] = chroma_source[idx + 3];
                }
                row
            })
            .collect();

        for (y, row) in rows.into_iter().enumerate() {
            data[y * w * 4..(y + 1) * w * 4].copy_from_slice(&row);
        }
    }
}

/// Apply dehaze using dark channel prior (He et al. 2009, CVPR)
/// with guided filter refinement (He et al. 2013, TPAMI).
///
/// Positive strength removes haze, negative adds atmospheric haze.
/// Steps: 1) dark channel, 2) atmospheric light, 3) transmission via
/// normalized dark channel, 4) guided filter refinement, 5) scene radiance.
///
/// Reference: He K., Sun J., Tang X., "Single Image Haze Removal Using
/// Dark Channel Prior", CVPR 2009 (Best Paper).
/// Refinement: He K., Sun J., Tang X., "Guided Image Filtering",
/// IEEE TPAMI 35(6), 1397–1409, 2013.
pub(crate) fn apply_dehaze_cpu(data: &mut [u8], width: u32, height: u32, dehaze: f32) {
    let w = width as usize;
    let h = height as usize;
    let omega = dehaze / 100.0; // [-1, 1]
    let patch_radius: i32 = 7; // 15×15 patch (He et al. recommend 15×15)
    let source = data.to_owned();

    // Step 1: Compute un-normalized dark channel for atmospheric light estimation
    //   DC(x) = min_{y∈Ω(x)} min_{c∈{r,g,b}} I^c(y)
    let dark_channel: Vec<f32> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0.0f32; w];
            for (x, dark_val) in row.iter_mut().enumerate() {
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
                *dark_val = dark;
            }
            row
        })
        .collect();

    // Step 2: Estimate atmospheric light A — top 0.1% brightest pixels in the
    // dark channel, then average their original RGB values (He et al. §3.2).
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
    let atm = [
        (atm_r / n).max(0.01),
        (atm_g / n).max(0.01),
        (atm_b / n).max(0.01),
    ];

    // Step 3: Compute coarse per-pixel transmission using the NORMALIZED dark channel.
    //   t(x) = 1 - ω * min_{y∈Ω(x)} min_{c} (I^c(y) / A^c)
    // Per He et al. eq. 12: each channel is divided by its atmospheric light
    // component BEFORE the double-min, not after.
    let coarse_transmission: Vec<f32> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            let mut row = vec![0.0f32; w];
            for (x, trans_val) in row.iter_mut().enumerate() {
                let mut dc_norm = 1.0f32;
                for dy in -patch_radius..=patch_radius {
                    let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                    for dx in -patch_radius..=patch_radius {
                        let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                        let si = (sy * w + sx) * 4;
                        let nr = source[si] as f32 / 255.0 / atm[0];
                        let ng = source[si + 1] as f32 / 255.0 / atm[1];
                        let nb = source[si + 2] as f32 / 255.0 / atm[2];
                        dc_norm = dc_norm.min(nr.min(ng).min(nb));
                    }
                }
                *trans_val = (1.0 - omega * dc_norm).max(0.1);
            }
            row
        })
        .collect();

    // Step 4: Refine transmission with guided filter (He et al. 2013).
    // Uses the grayscale input image as guidance, radius=60, eps=1e-3.
    // This eliminates block artifacts at patch boundaries.
    let guide: Vec<f32> = (0..num_pixels)
        .map(|i| {
            let si = i * 4;
            let r = source[si] as f32 / 255.0;
            let g = source[si + 1] as f32 / 255.0;
            let b = source[si + 2] as f32 / 255.0;
            0.2126 * r + 0.7152 * g + 0.0722 * b
        })
        .collect();
    let transmission = guided_filter(&guide, &coarse_transmission, w, h, 60, 1e-3);

    // Step 5: Recover scene radiance: J(x) = (I(x) - A) / max(t(x), t0) + A
    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![0u8; w * 4];
            for x in 0..w {
                let idx = (y * w + x) * 4;
                let t = transmission[y * w + x].max(0.1);
                let r = source[idx] as f32 / 255.0;
                let g = source[idx + 1] as f32 / 255.0;
                let b = source[idx + 2] as f32 / 255.0;

                let out_r = (r - atm[0]) / t + atm[0];
                let out_g = (g - atm[1]) / t + atm[1];
                let out_b = (b - atm[2]) / t + atm[2];

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

/// Apply tone region adjustments (highlights/shadows/whites/blacks) with
/// edge-aware guided-filter masks to prevent halos at high-contrast edges.
///
/// Uses the same power-curve luminance masks as the per-pixel path but
/// refines them with a guided filter (He et al. 2013) to make the masks
/// edge-aware, matching the approach used by RawTherapee's Shadows/Highlights
/// module.
pub(crate) fn apply_tone_regions_cpu(
    data: &mut [u8],
    width: u32,
    height: u32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
) {
    let w = width as usize;
    let h = height as usize;
    let n = w * h;
    let hl_factor = highlights / 200.0;
    let sh_factor = shadows / 200.0;
    let w_factor = whites / 200.0;
    let b_factor = blacks / 200.0;

    if hl_factor.abs() < 0.001 && sh_factor.abs() < 0.001
        && w_factor.abs() < 0.001 && b_factor.abs() < 0.001
    {
        return;
    }

    // Step 1: Extract luminance as guide
    let guide: Vec<f32> = (0..n)
        .map(|i| {
            let si = i * 4;
            let r = data[si] as f32 / 255.0;
            let g = data[si + 1] as f32 / 255.0;
            let b = data[si + 2] as f32 / 255.0;
            0.2126 * r + 0.7152 * g + 0.0722 * b
        })
        .collect();

    // Step 2: Compute raw masks
    let mut hl_mask = vec![0.0f32; n];
    let mut sh_mask = vec![0.0f32; n];
    let mut w_mask = vec![0.0f32; n];
    let mut b_mask = vec![0.0f32; n];
    for i in 0..n {
        let l = guide[i];
        hl_mask[i] = l * l;
        sh_mask[i] = (1.0 - l) * (1.0 - l);
        w_mask[i] = hl_mask[i] * hl_mask[i];
        b_mask[i] = sh_mask[i] * sh_mask[i];
    }

    // Step 3: Refine masks with guided filter (edge-aware smoothing)
    // radius=30, eps=0.01 — preserves edges while smoothing the masks
    let gf_radius = 30;
    let gf_eps = 0.01;
    let hl_refined = if hl_factor.abs() > 0.001 {
        guided_filter(&guide, &hl_mask, w, h, gf_radius, gf_eps)
    } else { hl_mask };
    let sh_refined = if sh_factor.abs() > 0.001 {
        guided_filter(&guide, &sh_mask, w, h, gf_radius, gf_eps)
    } else { sh_mask };
    let w_refined = if w_factor.abs() > 0.001 {
        guided_filter(&guide, &w_mask, w, h, gf_radius, gf_eps)
    } else { w_mask };
    let b_refined = if b_factor.abs() > 0.001 {
        guided_filter(&guide, &b_mask, w, h, gf_radius, gf_eps)
    } else { b_mask };

    // Step 4: Apply adjustments
    let process_row = |y: usize, row: &mut [u8]| {
        for x in 0..w {
            let idx = y * w + x;
            let i = x * 4;
            let r = row[i] as f32 / 255.0;
            let g = row[i + 1] as f32 / 255.0;
            let b = row[i + 2] as f32 / 255.0;
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

            let lum_shift = hl_refined[idx] * hl_factor
                + sh_refined[idx] * sh_factor
                + w_refined[idx] * w_factor
                + b_refined[idx] * b_factor;

            if lum_shift.abs() > 0.0001 {
                let target_lum = (lum + lum_shift).clamp(0.0, 1.5);
                let ratio = if lum < 0.001 { 1.0 + lum_shift } else { target_lum / lum };
                row[i] = ((r * ratio).clamp(0.0, 1.0) * 255.0) as u8;
                row[i + 1] = ((g * ratio).clamp(0.0, 1.0) * 255.0) as u8;
                row[i + 2] = ((b * ratio).clamp(0.0, 1.0) * 255.0) as u8;
            }
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
