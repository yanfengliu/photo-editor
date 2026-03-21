use std::sync::mpsc;

use rayon::prelude::*;

use crate::catalog::models::EditParams;
use crate::gpu::context::GpuContext;
use crate::gpu::cpu_edits::{apply_edits_to_pixel, apply_hsl_to_pixel, CpuEditProfile};
use crate::gpu::curves::CurveLuts;
use crate::gpu::passes::basic_adjustments::BasicAdjustmentsParams;
use crate::gpu::spatial::{
    apply_clarity_cpu, apply_dehaze_cpu, apply_denoise_cpu, apply_grain_cpu,
    apply_sharpening_cpu, apply_tone_regions_cpu, apply_vignette_cpu,
};
use crate::imaging::lens_profiles;

/// Lens metadata from EXIF, used for auto-detecting lens profiles
#[derive(Debug, Clone, Default)]
pub struct LensMetadata {
    pub lens_name: Option<String>,
    pub focal_length: Option<f64>,
}

pub const PARALLEL_PIXEL_THRESHOLD: usize = 512 * 512;

/// Checks if crop/rotation params differ from defaults
fn needs_crop_rotation(params: &EditParams) -> bool {
    params.rotation != 0
        || params.rotation_fine.abs() > 0.001
        || params.crop_x != 0.0
        || params.crop_y != 0.0
        || params.crop_width < 0.999
        || params.crop_height < 0.999
}

/// Apply 90-degree rotation to RGBA buffer. Returns (new_data, new_width, new_height).
fn rotate_90_steps(data: &[u8], width: u32, height: u32, rotation: i32) -> (Vec<u8>, u32, u32) {
    let steps = rotation.rem_euclid(360) / 90;
    if steps == 0 {
        return (data.to_vec(), width, height);
    }

    let (w, h) = (width as usize, height as usize);
    let mut src = data.to_vec();
    let mut sw = w;
    let mut sh = h;

    for _ in 0..steps {
        let dw = sh;
        let dh = sw;
        let mut dst = vec![0u8; dw * dh * 4];
        for y in 0..sh {
            for x in 0..sw {
                // 90° CW: (x, y) -> (h - 1 - y, x)
                let src_idx = (y * sw + x) * 4;
                let dst_x = sh - 1 - y;
                let dst_y = x;
                let dst_idx = (dst_y * dw + dst_x) * 4;
                dst[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
            }
        }
        src = dst;
        sw = dw;
        sh = dh;
    }

    (src, sw as u32, sh as u32)
}

/// Crop an RGBA buffer to a sub-region defined by normalized [0,1] coordinates.
/// Returns (new_data, new_width, new_height).
fn crop_region(
    data: &[u8],
    width: u32,
    height: u32,
    crop_x: f32,
    crop_y: f32,
    crop_w: f32,
    crop_h: f32,
) -> (Vec<u8>, u32, u32) {
    let x0 = ((crop_x * width as f32).round() as u32).min(width);
    let y0 = ((crop_y * height as f32).round() as u32).min(height);
    let cw = ((crop_w * width as f32).round() as u32).min(width - x0).max(1);
    let ch = ((crop_h * height as f32).round() as u32).min(height - y0).max(1);

    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    let stride = (width * 4) as usize;
    for row in y0..y0 + ch {
        let start = (row as usize) * stride + (x0 as usize) * 4;
        let end = start + (cw as usize) * 4;
        out.extend_from_slice(&data[start..end]);
    }

    (out, cw, ch)
}

/// Rotate an image by an arbitrary angle (degrees) using bilinear interpolation,
/// then crop to the largest inscribed axis-aligned rectangle (same aspect ratio).
fn rotate_fine(data: &[u8], width: u32, height: u32, angle_deg: f32) -> (Vec<u8>, u32, u32) {
    let w = width as f64;
    let h = height as f64;
    let angle_rad = (angle_deg as f64) * std::f64::consts::PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let abs_sin = sin_a.abs();
    let abs_cos = cos_a.abs();

    // Inscribed rectangle scale (same formula as ImageCanvas.tsx)
    let inscribed_scale = (w / (w * abs_cos + h * abs_sin))
        .min(h / (w * abs_sin + h * abs_cos));

    let out_w = ((w * inscribed_scale).round() as u32).max(1);
    let out_h = ((h * inscribed_scale).round() as u32).max(1);

    // Center of source and output
    let cx_src = (w - 1.0) / 2.0;
    let cy_src = (h - 1.0) / 2.0;
    let cx_out = (out_w as f64 - 1.0) / 2.0;
    let cy_out = (out_h as f64 - 1.0) / 2.0;

    let stride = width as usize * 4;
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];

    for oy in 0..out_h {
        for ox in 0..out_w {
            // Map output pixel to source via inverse rotation
            let dx = ox as f64 - cx_out;
            let dy = oy as f64 - cy_out;
            let sx = cos_a * dx + sin_a * dy + cx_src;
            let sy = -sin_a * dx + cos_a * dy + cy_src;

            // Bilinear interpolation
            let x0 = sx.floor() as i64;
            let y0 = sy.floor() as i64;
            let fx = (sx - x0 as f64) as f32;
            let fy = (sy - y0 as f64) as f32;

            let sample = |px: i64, py: i64| -> [f32; 4] {
                let cx = px.clamp(0, width as i64 - 1) as usize;
                let cy = py.clamp(0, height as i64 - 1) as usize;
                let i = cy * stride + cx * 4;
                [
                    data[i] as f32,
                    data[i + 1] as f32,
                    data[i + 2] as f32,
                    data[i + 3] as f32,
                ]
            };

            let tl = sample(x0, y0);
            let tr = sample(x0 + 1, y0);
            let bl = sample(x0, y0 + 1);
            let br = sample(x0 + 1, y0 + 1);

            let oi = (oy * out_w + ox) as usize * 4;
            for c in 0..4 {
                let top = tl[c] * (1.0 - fx) + tr[c] * fx;
                let bot = bl[c] * (1.0 - fx) + br[c] * fx;
                out[oi + c] = (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    (out, out_w, out_h)
}

/// Apply crop and 90° rotation to processed image. Returns (data, width, height).
pub fn apply_crop_rotation(
    data: Vec<u8>,
    width: u32,
    height: u32,
    params: &EditParams,
) -> (Vec<u8>, u32, u32) {
    if !needs_crop_rotation(params) {
        return (data, width, height);
    }

    // Crop first (operates on the original orientation)
    let (cropped, cw, ch) = if params.crop_x != 0.0
        || params.crop_y != 0.0
        || params.crop_width < 0.999
        || params.crop_height < 0.999
    {
        crop_region(&data, width, height, params.crop_x, params.crop_y, params.crop_width, params.crop_height)
    } else {
        (data, width, height)
    };

    // 90° rotation
    let (rotated, rw, rh) = if params.rotation != 0 {
        rotate_90_steps(&cropped, cw, ch, params.rotation)
    } else {
        (cropped, cw, ch)
    };

    // Fine rotation with inscribed crop
    if params.rotation_fine.abs() > 0.001 {
        rotate_fine(&rotated, rw, rh, params.rotation_fine)
    } else {
        (rotated, rw, rh)
    }
}

/// Apply lens correction if enabled, returning corrected data or original
fn apply_lens_correction_step(
    data: &[u8],
    width: u32,
    height: u32,
    params: &EditParams,
    lens_meta: Option<&LensMetadata>,
) -> Option<Vec<u8>> {
    if !params.enable_lens_correction {
        return None;
    }
    if !params.lens_ca_correction && !params.lens_vignette_correction && params.lens_distortion.abs() < 0.001 {
        return None;
    }

    // Find profile: explicit ID first, then auto-detect from EXIF
    let profile = params
        .lens_profile_id
        .as_deref()
        .and_then(|id| lens_profiles::find_profile_by_id(id))
        .or_else(|| {
            lens_meta
                .and_then(|m| m.lens_name.as_deref())
                .and_then(|name| lens_profiles::find_profile_by_name(name))
        });

    let profile = profile?;

    let focal = lens_meta
        .and_then(|m| m.focal_length)
        .unwrap_or(profile.focal_range.0);

    let correct_distortion = params.lens_distortion.abs() > 0.001;
    let amount = params.lens_distortion_amount as f64;

    Some(crate::imaging::lens_correction::apply_lens_correction(
        data,
        width,
        height,
        profile,
        focal,
        correct_distortion,
        params.lens_ca_correction,
        params.lens_vignette_correction,
        amount,
    ))
}

pub fn apply_edits_cpu(rgba_data: &[u8], width: u32, height: u32, params: &EditParams) -> Vec<u8> {
    apply_edits_cpu_with_lens(rgba_data, width, height, params, None)
}

pub fn apply_edits_cpu_with_lens(rgba_data: &[u8], width: u32, height: u32, params: &EditParams, lens_meta: Option<&LensMetadata>) -> Vec<u8> {
    if rgba_data.is_empty() {
        return Vec::new();
    }

    // Lens correction runs first (geometric transform before color adjustments)
    let corrected;
    let input = if let Some(lc) = apply_lens_correction_step(rgba_data, width, height, params, lens_meta) {
        corrected = lc;
        &corrected
    } else {
        rgba_data
    };

    let profile = CpuEditProfile::from_params(params);
    let curves = CurveLuts::from_params(params);

    let mut result = input.to_vec();
    let pixel_count = result.len() / 4;

    // --- Per-pixel passes: basic adjustments, HSL, curves ---
    let apply_basic = !profile.is_neutral();
    let apply_curves = !curves.is_identity();
    let apply_hsl = profile.apply_hsl;

    if apply_basic || apply_curves || apply_hsl {
        let process_pixel = |pixel: &mut [u8]| {
            if apply_basic { apply_edits_to_pixel(pixel, profile); }
            if apply_hsl { apply_hsl_to_pixel(pixel, profile); }
            if apply_curves { curves.apply(pixel); }
        };

        if pixel_count >= PARALLEL_PIXEL_THRESHOLD {
            result.par_chunks_exact_mut(4).for_each(process_pixel);
        } else {
            result.chunks_exact_mut(4).for_each(process_pixel);
        }
    }

    // --- Spatial passes (need full buffer + dimensions) ---
    // Edge-aware tone region adjustment (highlights/shadows/whites/blacks)
    apply_tone_regions_cpu(&mut result, width, height,
        params.highlights, params.shadows, params.whites, params.blacks);
    if params.dehaze.abs() > 0.01 {
        apply_dehaze_cpu(&mut result, width, height, params.dehaze);
    }
    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut result, width, height, params.sharpening_amount, params.sharpening_radius, params.sharpening_detail);
    }
    if params.clarity.abs() > 0.01 {
        apply_clarity_cpu(&mut result, width, height, params.clarity);
    }
    if params.denoise_luminance > 0.01 || params.denoise_color > 0.01 {
        apply_denoise_cpu(&mut result, width, height, params.denoise_luminance, params.denoise_color);
    }

    // --- Coordinate-aware passes ---
    if params.vignette_amount.abs() > 0.01 {
        apply_vignette_cpu(&mut result, width, height, params.vignette_amount);
    }

    if params.grain_amount > 0.01 {
        apply_grain_cpu(&mut result, width, height, params.grain_amount, params.grain_size);
    }

    result
}

/// Check if any non-basic CPU post-processing is needed
fn needs_cpu_post_processing(params: &EditParams) -> bool {
    let defaults = EditParams::default();

    params.highlights.abs() > 0.01
        || params.shadows.abs() > 0.01
        || params.whites.abs() > 0.01
        || params.blacks.abs() > 0.01
        || params.dehaze.abs() > 0.01
        || params.clarity.abs() > 0.001
        || params.sharpening_amount > 0.001
        || params.denoise_luminance > 0.001
        || params.denoise_color > 0.001
        || params.vignette_amount.abs() > 0.001
        || params.grain_amount > 0.001
        || params.curve_rgb != defaults.curve_rgb
        || params.curve_r != defaults.curve_r
        || params.curve_g != defaults.curve_g
        || params.curve_b != defaults.curve_b
        || params.hsl_hue != defaults.hsl_hue
        || params.hsl_saturation != defaults.hsl_saturation
        || params.hsl_luminance != defaults.hsl_luminance
}

/// Apply only the non-basic CPU passes (HSL, curves, spatial, coordinate-aware)
fn apply_cpu_post_processing(mut data: Vec<u8>, width: u32, height: u32, params: &EditParams) -> Vec<u8> {
    let profile = CpuEditProfile::from_params(params);
    let curves = CurveLuts::from_params(params);
    let pixel_count = data.len() / 4;

    // Per-pixel: HSL + curves (basic adjustments already done by GPU)
    let apply_hsl = profile.apply_hsl;
    let apply_curves = !curves.is_identity();

    if apply_hsl || apply_curves {
        let process_pixel = |pixel: &mut [u8]| {
            if apply_hsl { apply_hsl_to_pixel(pixel, profile); }
            if apply_curves { curves.apply(pixel); }
        };
        if pixel_count >= PARALLEL_PIXEL_THRESHOLD {
            data.par_chunks_exact_mut(4).for_each(process_pixel);
        } else {
            data.chunks_exact_mut(4).for_each(process_pixel);
        }
    }

    // Spatial passes
    // Edge-aware tone region adjustment (highlights/shadows/whites/blacks)
    apply_tone_regions_cpu(&mut data, width, height,
        params.highlights, params.shadows, params.whites, params.blacks);
    if params.dehaze.abs() > 0.01 {
        apply_dehaze_cpu(&mut data, width, height, params.dehaze);
    }
    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut data, width, height, params.sharpening_amount, params.sharpening_radius, params.sharpening_detail);
    }
    if params.clarity.abs() > 0.01 {
        apply_clarity_cpu(&mut data, width, height, params.clarity);
    }
    if params.denoise_luminance > 0.01 || params.denoise_color > 0.01 {
        apply_denoise_cpu(&mut data, width, height, params.denoise_luminance, params.denoise_color);
    }

    // Coordinate-aware passes
    if params.vignette_amount.abs() > 0.01 {
        apply_vignette_cpu(&mut data, width, height, params.vignette_amount);
    }
    if params.grain_amount > 0.01 {
        apply_grain_cpu(&mut data, width, height, params.grain_amount, params.grain_size);
    }

    data
}

pub fn apply_edits_with_backend(
    gpu: Option<&mut GpuContext>,
    rgba_data: &[u8],
    width: u32,
    height: u32,
    params: &EditParams,
) -> Vec<u8> {
    apply_edits_with_backend_lens(gpu, rgba_data, width, height, params, None)
}

pub fn apply_edits_with_backend_lens(
    gpu: Option<&mut GpuContext>,
    rgba_data: &[u8],
    width: u32,
    height: u32,
    params: &EditParams,
    lens_meta: Option<&LensMetadata>,
) -> Vec<u8> {
    // Lens correction runs first (geometric transform)
    let corrected;
    let input = if let Some(lc) = apply_lens_correction_step(rgba_data, width, height, params, lens_meta) {
        corrected = lc;
        &corrected
    } else {
        rgba_data
    };

    // Try GPU for basic adjustments (WB, exposure, contrast, tone regions, sat, vibrance, dehaze)
    if let Some(gpu_ctx) = gpu {
        match apply_edits_gpu_basic(gpu_ctx, input, width, height, params) {
            Ok(result) => {
                // GPU handled basic adjustments; apply remaining passes on CPU if needed
                if needs_cpu_post_processing(params) {
                    return apply_cpu_post_processing(result, width, height, params);
                }
                return result;
            }
            Err(err) => log::warn!("GPU pipeline failed, falling back to CPU: {}", err),
        }
    }

    // Full CPU fallback (skip lens correction since we already did it above)
    let profile = CpuEditProfile::from_params(params);
    let curves = CurveLuts::from_params(params);
    let mut result = input.to_vec();
    let pixel_count = result.len() / 4;

    let apply_basic = !profile.is_neutral();
    let apply_curves = !curves.is_identity();
    let apply_hsl = profile.apply_hsl;

    if apply_basic || apply_curves || apply_hsl {
        let process_pixel = |pixel: &mut [u8]| {
            if apply_basic { apply_edits_to_pixel(pixel, profile); }
            if apply_hsl { apply_hsl_to_pixel(pixel, profile); }
            if apply_curves { curves.apply(pixel); }
        };
        if pixel_count >= PARALLEL_PIXEL_THRESHOLD {
            result.par_chunks_exact_mut(4).for_each(process_pixel);
        } else {
            result.chunks_exact_mut(4).for_each(process_pixel);
        }
    }

    // Edge-aware tone region adjustment (highlights/shadows/whites/blacks)
    apply_tone_regions_cpu(&mut result, width, height,
        params.highlights, params.shadows, params.whites, params.blacks);
    if params.dehaze.abs() > 0.01 {
        apply_dehaze_cpu(&mut result, width, height, params.dehaze);
    }
    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut result, width, height, params.sharpening_amount, params.sharpening_radius, params.sharpening_detail);
    }
    if params.clarity.abs() > 0.01 {
        apply_clarity_cpu(&mut result, width, height, params.clarity);
    }
    if params.denoise_luminance > 0.01 || params.denoise_color > 0.01 {
        apply_denoise_cpu(&mut result, width, height, params.denoise_luminance, params.denoise_color);
    }
    if params.vignette_amount.abs() > 0.01 {
        apply_vignette_cpu(&mut result, width, height, params.vignette_amount);
    }
    if params.grain_amount > 0.01 {
        apply_grain_cpu(&mut result, width, height, params.grain_amount, params.grain_size);
    }

    result
}

fn apply_edits_gpu_basic(
    gpu: &mut GpuContext,
    rgba_data: &[u8],
    width: u32,
    height: u32,
    params: &EditParams,
) -> Result<Vec<u8>, String> {
    if rgba_data.is_empty() {
        return Ok(Vec::new());
    }

    let expected_len = width as usize * height as usize * 4;
    if rgba_data.len() != expected_len {
        return Err(format!(
            "Input buffer length {} does not match {}x{} RGBA image",
            rgba_data.len(),
            width,
            height
        ));
    }

    // Reuse cached textures + readback buffer when dimensions match
    gpu.get_or_create_resources(width, height);
    let res = gpu.cached.as_ref().unwrap();

    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &res.textures.input,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        extent,
    );

    let padded_bytes_per_row = res.padded_bytes_per_row;

    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Develop GPU Command Encoder"),
    });

    gpu.basic_adjustments_pass.encode(
        &gpu.device,
        &mut encoder,
        &res.textures.input_view,
        &res.textures.output_view,
        &BasicAdjustmentsParams::from(params),
        width,
        height,
    );

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &res.textures.output,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &res.readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        extent,
    );

    gpu.queue.submit(Some(encoder.finish()));

    let slice = res.readback.slice(..);
    let (sender, receiver) = mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    gpu.device.poll(wgpu::Maintain::Wait);

    let map_result = receiver.recv().map_err(|err| err.to_string())?;
    map_result.map_err(|err| err.to_string())?;

    let mapped = slice.get_mapped_range();
    let bytes_per_row = width as usize * 4;
    let padded_row_len = padded_bytes_per_row as usize;
    let mut result = vec![0u8; expected_len];

    for row in 0..height as usize {
        let src_offset = row * padded_row_len;
        let dst_offset = row * bytes_per_row;
        result[dst_offset..dst_offset + bytes_per_row]
            .copy_from_slice(&mapped[src_offset..src_offset + bytes_per_row]);
    }

    drop(mapped);
    // Must unmap before next map_async call
    res.readback.unmap();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::models::{CurvePoint, EditParams};
    use crate::gpu::curves::{build_curve_lut, is_identity_curve};

    fn make_gray_image(value: u8, pixel_count: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&[value, value, value, 255]);
        }
        data
    }

    /// Helper: apply CPU edits to a 1-row image (for per-pixel tests)
    fn cpu_edit(input: &[u8], params: &EditParams) -> Vec<u8> {
        let pixel_count = input.len() / 4;
        apply_edits_cpu(input, pixel_count as u32, 1, params)
    }

    #[test]
    fn test_neutral_params_no_change() {
        let input = make_gray_image(128, 4);
        let params = EditParams::default();
        let output = cpu_edit(&input, &params);
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
        params.exposure = 1.0;
        let output = cpu_edit(&input, &params);
        assert!(output[0] > input[0], "Exposure +1 should brighten: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_exposure_darkens() {
        let input = make_gray_image(200, 1);
        let mut params = EditParams::default();
        params.exposure = -1.0;
        let output = cpu_edit(&input, &params);
        assert!(output[0] < input[0], "Exposure -1 should darken: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_contrast_increase() {
        let input = vec![128, 128, 128, 255];
        let mut params = EditParams::default();
        params.contrast = 50.0;
        let output = cpu_edit(&input, &params);
        let diff = (output[0] as i32 - 128).abs();
        assert!(diff < 10, "Mid-gray with contrast should stay near 128, got {}", output[0]);
    }

    #[test]
    fn test_saturation_desaturate() {
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.saturation = -100.0;
        let output = cpu_edit(&input, &params);
        let max_diff = (output[0] as i32 - output[1] as i32).abs()
            .max((output[1] as i32 - output[2] as i32).abs());
        assert!(max_diff < 5, "Desaturated red should be near gray, got ({}, {}, {})", output[0], output[1], output[2]);
    }

    #[test]
    fn test_warm_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 10000.0;
        let output = cpu_edit(&input, &params);
        assert!(output[0] >= output[2], "Warm WB should shift red > blue: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_cool_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 3000.0;
        let output = cpu_edit(&input, &params);
        assert!(output[2] >= output[0], "Cool WB should shift blue > red: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_alpha_preserved() {
        let input = vec![100, 100, 100, 200];
        let mut params = EditParams::default();
        params.exposure = 1.0;
        let output = cpu_edit(&input, &params);
        assert_eq!(output[3], 200, "Alpha should be preserved");
    }

    #[test]
    fn test_output_clamped() {
        let input = vec![250, 250, 250, 255];
        let mut params = EditParams::default();
        params.exposure = 3.0;
        let output = cpu_edit(&input, &params);
        assert_eq!(output[0], 255, "Output should be clamped to 255");
        assert_eq!(output[1], 255);
        assert_eq!(output[2], 255);
    }

    #[test]
    fn test_dehaze_positive() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.dehaze = 50.0;
        let output = cpu_edit(&input, &params);
        assert_eq!(output[3], 255);
    }

    #[test]
    fn test_multiple_params() {
        let input = make_gray_image(100, 4);
        let mut params = EditParams::default();
        params.exposure = 0.5;
        params.contrast = 20.0;
        params.saturation = 10.0;
        params.vibrance = 10.0;
        let output = cpu_edit(&input, &params);
        assert_eq!(output.len(), input.len());
        for i in 0..4 {
            assert_eq!(output[i * 4 + 3], 255);
        }
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<u8> = vec![];
        let params = EditParams::default();
        let output = cpu_edit(&input, &params);
        assert!(output.is_empty());
    }

    #[test]
    fn test_needs_cpu_post_processing_default() {
        let params = EditParams::default();
        assert!(!needs_cpu_post_processing(&params));
    }

    #[test]
    fn test_needs_cpu_post_processing_with_clarity() {
        let mut params = EditParams::default();
        params.clarity = 10.0;
        assert!(needs_cpu_post_processing(&params));
    }

    // --- Tone curve tests ---

    #[test]
    fn test_identity_curve_no_change() {
        let input = make_gray_image(128, 1);
        let params = EditParams::default(); // default curves are identity
        let output = cpu_edit(&input, &params);
        assert_eq!(output[0], 128);
    }

    #[test]
    fn test_curve_lut_identity() {
        let identity = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let lut = build_curve_lut(&identity);
        for i in 0..256 {
            let expected = i as f32 / 255.0;
            assert!((lut[i] - expected).abs() < 0.01, "LUT[{}]: expected {}, got {}", i, expected, lut[i]);
        }
    }

    #[test]
    fn test_curve_lut_brighten() {
        let points = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.75 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let lut = build_curve_lut(&points);
        assert!(lut[128] > 0.6, "Midpoint should be lifted: got {}", lut[128]);
        assert!(lut[0] < 0.01, "Black point should be near 0: got {}", lut[0]);
        assert!(lut[255] > 0.99, "White point should be near 1: got {}", lut[255]);
    }

    #[test]
    fn test_curve_lut_darken() {
        let points = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.25 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let lut = build_curve_lut(&points);
        assert!(lut[128] < 0.4, "Midpoint should be darkened: got {}", lut[128]);
    }

    #[test]
    fn test_rgb_curve_brightens_image() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.curve_rgb = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.75 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let output = cpu_edit(&input, &params);
        assert!(output[0] > 128, "Brightening curve should lift mid-gray: got {}", output[0]);
    }

    #[test]
    fn test_per_channel_curve_shifts_color() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.curve_r = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.8 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let output = cpu_edit(&input, &params);
        assert!(output[0] > output[1], "Red channel boost: R={} should be > G={}", output[0], output[1]);
        assert!(output[0] > output[2], "Red channel boost: R={} should be > B={}", output[0], output[2]);
    }

    #[test]
    fn test_curve_with_exposure_combined() {
        let input = make_gray_image(100, 1);
        let mut params = EditParams::default();
        params.exposure = 1.0;
        params.curve_rgb = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.75 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let output = cpu_edit(&input, &params);
        assert!(output[0] > input[0], "Combined exposure+curve should brighten: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_curve_alpha_preserved() {
        let input = vec![128, 128, 128, 200];
        let mut params = EditParams::default();
        params.curve_rgb = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.8 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let output = cpu_edit(&input, &params);
        assert_eq!(output[3], 200, "Alpha should be preserved through curve: got {}", output[3]);
    }

    #[test]
    fn test_is_identity_curve() {
        let identity = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        assert!(is_identity_curve(&identity));

        let non_identity = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.7 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        assert!(!is_identity_curve(&non_identity));
    }

    // --- HSL tests ---

    #[test]
    fn test_hsl_saturation_desaturates_red() {
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.hsl_saturation = [-100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let output = cpu_edit(&input, &params);
        let max_diff = (output[0] as i32 - output[1] as i32).abs()
            .max((output[1] as i32 - output[2] as i32).abs());
        assert!(max_diff < 30, "Desaturated red should be near gray, got ({}, {}, {})", output[0], output[1], output[2]);
    }

    #[test]
    fn test_hsl_hue_shifts_red() {
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.hsl_hue = [120.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let output = cpu_edit(&input, &params);
        assert!(output[1] > output[0] && output[1] > output[2],
            "Hue-shifted red should be greenish, got ({}, {}, {})", output[0], output[1], output[2]);
    }

    #[test]
    fn test_hsl_luminance_darkens() {
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.hsl_luminance = [-50.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let output = cpu_edit(&input, &params);
        assert!(output[0] < input[0], "Luminance reduction should darken red: {} vs {}", output[0], input[0]);
    }

    #[test]
    fn test_hsl_does_not_affect_unrelated_channel() {
        let input = vec![0, 0, 255, 255];
        let mut params = EditParams::default();
        params.hsl_saturation = [-100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let output = cpu_edit(&input, &params);
        assert_eq!(output[0], 0, "Blue pixel R should stay 0, got {}", output[0]);
        assert_eq!(output[1], 0, "Blue pixel G should stay 0, got {}", output[1]);
        assert_eq!(output[2], 255, "Blue pixel B should stay 255, got {}", output[2]);
    }

    // --- Spatial tests ---

    #[test]
    fn test_sharpening_changes_pixels() {
        let mut input = make_gray_image(100, 9);
        input[4 * 4] = 200; input[4 * 4 + 1] = 200; input[4 * 4 + 2] = 200;
        let mut params = EditParams::default();
        params.sharpening_amount = 100.0;
        params.sharpening_radius = 1.0;
        let output = apply_edits_cpu(&input, 3, 3, &params);
        assert!(output[4 * 4] > input[4 * 4], "Sharpening should boost bright center: {} vs {}", output[4 * 4], input[4 * 4]);
    }

    #[test]
    fn test_clarity_changes_pixels() {
        let mut input = Vec::with_capacity(20 * 4);
        for i in 0..20 {
            let v = if i < 10 { 80u8 } else { 180 };
            input.extend_from_slice(&[v, v, v, 255]);
        }
        let mut params = EditParams::default();
        params.clarity = 50.0;
        let output = apply_edits_cpu(&input, 20, 1, &params);
        assert!(output[0] < input[0] || output[19 * 4] > input[19 * 4],
            "Clarity should enhance local contrast");
    }

    #[test]
    fn test_vignette_darkens_corners() {
        let input = make_gray_image(200, 100);
        let mut params = EditParams::default();
        params.vignette_amount = 100.0;
        let output = apply_edits_cpu(&input, 10, 10, &params);
        let center_idx = (5 * 10 + 5) * 4;
        let corner_idx = 0;
        assert!(output[corner_idx] < output[center_idx],
            "Vignette: corner ({}) should be darker than center ({})", output[corner_idx], output[center_idx]);
    }

    #[test]
    fn test_grain_adds_variation() {
        let input = make_gray_image(128, 16);
        let mut params = EditParams::default();
        params.grain_amount = 50.0;
        params.grain_size = 25.0;
        let output = apply_edits_cpu(&input, 4, 4, &params);
        let mut has_variation = false;
        for i in 0..16 {
            if output[i * 4] != 128 {
                has_variation = true;
                break;
            }
        }
        assert!(has_variation, "Grain should add variation to uniform image");
    }

    #[test]
    fn test_denoise_smooths_noise() {
        let mut input = Vec::with_capacity(25 * 4);
        for i in 0..25 {
            let v = if i % 2 == 0 { 140u8 } else { 120 };
            input.extend_from_slice(&[v, v, v, 255]);
        }
        let mut params = EditParams::default();
        params.denoise_luminance = 50.0;
        let output = apply_edits_cpu(&input, 5, 5, &params);
        let center = output[12 * 4];
        assert!(center > 120 && center < 140,
            "Denoise should smooth noise toward average, got {}", center);
    }

    #[test]
    fn test_gpu_pipeline_smoke_if_available() {
        let mut gpu = match pollster::block_on(GpuContext::new()) {
            Ok(gpu) => gpu,
            Err(_) => return,
        };

        let input = vec![128, 128, 128, 255];
        let mut params = EditParams::default();
        params.exposure = 0.5;

        let output = apply_edits_gpu_basic(&mut gpu, &input, 1, 1, &params)
            .expect("GPU basic pipeline should process a 1x1 image");

        assert_eq!(output.len(), input.len());
        assert_eq!(output[3], 255);
    }

    // --- Crop & Rotation tests ---

    /// Make a 2x2 RGBA image with distinct pixel colors
    fn make_2x2_image() -> Vec<u8> {
        vec![
            255, 0, 0, 255,    // (0,0) red
            0, 255, 0, 255,    // (1,0) green
            0, 0, 255, 255,    // (0,1) blue
            255, 255, 0, 255,  // (1,1) yellow
        ]
    }

    #[test]
    fn test_rotate_0_no_change() {
        let input = make_2x2_image();
        let (out, w, h) = rotate_90_steps(&input, 2, 2, 0);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(out, input);
    }

    #[test]
    fn test_rotate_90_cw() {
        let input = make_2x2_image();
        let (out, w, h) = rotate_90_steps(&input, 2, 2, 90);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // After 90° CW: (0,0)→(1,0), (1,0)→(1,1), (0,1)→(0,0), (1,1)→(0,1)
        // New layout: blue, red, yellow, green
        assert_eq!(&out[0..4], &[0, 0, 255, 255]);   // blue at (0,0)
        assert_eq!(&out[4..8], &[255, 0, 0, 255]);   // red at (1,0)
        assert_eq!(&out[8..12], &[255, 255, 0, 255]); // yellow at (0,1)
        assert_eq!(&out[12..16], &[0, 255, 0, 255]);  // green at (1,1)
    }

    #[test]
    fn test_rotate_180() {
        let input = make_2x2_image();
        let (out, w, h) = rotate_90_steps(&input, 2, 2, 180);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // 180°: reversed order
        assert_eq!(&out[0..4], &[255, 255, 0, 255]);  // yellow
        assert_eq!(&out[4..8], &[0, 0, 255, 255]);    // blue
        assert_eq!(&out[8..12], &[0, 255, 0, 255]);   // green
        assert_eq!(&out[12..16], &[255, 0, 0, 255]);  // red
    }

    #[test]
    fn test_rotate_270() {
        let input = make_2x2_image();
        let (out, w, h) = rotate_90_steps(&input, 2, 2, 270);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // 270° CW = 90° CCW
        assert_eq!(&out[0..4], &[0, 255, 0, 255]);    // green at (0,0)
        assert_eq!(&out[4..8], &[255, 255, 0, 255]);  // yellow at (1,0)
        assert_eq!(&out[8..12], &[255, 0, 0, 255]);   // red at (0,1)
        assert_eq!(&out[12..16], &[0, 0, 255, 255]);  // blue at (1,1)
    }

    #[test]
    fn test_rotate_360_is_identity() {
        let input = make_2x2_image();
        let (out, w, h) = rotate_90_steps(&input, 2, 2, 360);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(out, input);
    }

    #[test]
    fn test_rotate_non_square() {
        // 3x1 image
        let input = vec![
            255, 0, 0, 255,   // pixel 0
            0, 255, 0, 255,   // pixel 1
            0, 0, 255, 255,   // pixel 2
        ];
        let (out, w, h) = rotate_90_steps(&input, 3, 1, 90);
        assert_eq!(w, 1);
        assert_eq!(h, 3);
        // 90° CW of 3x1 → 1x3: bottom-to-top becomes left-to-right
        assert_eq!(&out[0..4], &[255, 0, 0, 255]);   // (0,0) was (0,0)
        assert_eq!(&out[4..8], &[0, 255, 0, 255]);   // (0,1) was (1,0)
        assert_eq!(&out[8..12], &[0, 0, 255, 255]);  // (0,2) was (2,0)
    }

    #[test]
    fn test_crop_full_image() {
        let input = make_2x2_image();
        let (out, w, h) = crop_region(&input, 2, 2, 0.0, 0.0, 1.0, 1.0);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(out, input);
    }

    #[test]
    fn test_crop_top_left_quarter() {
        let input = make_2x2_image();
        let (out, w, h) = crop_region(&input, 2, 2, 0.0, 0.0, 0.5, 0.5);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(&out[0..4], &[255, 0, 0, 255]); // red pixel
    }

    #[test]
    fn test_crop_bottom_right_quarter() {
        let input = make_2x2_image();
        let (out, w, h) = crop_region(&input, 2, 2, 0.5, 0.5, 0.5, 0.5);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(&out[0..4], &[255, 255, 0, 255]); // yellow pixel
    }

    #[test]
    fn test_crop_right_half() {
        let input = make_2x2_image();
        let (out, w, h) = crop_region(&input, 2, 2, 0.5, 0.0, 0.5, 1.0);
        assert_eq!(w, 1);
        assert_eq!(h, 2);
        assert_eq!(&out[0..4], &[0, 255, 0, 255]);   // green
        assert_eq!(&out[4..8], &[255, 255, 0, 255]);  // yellow
    }

    #[test]
    fn test_needs_crop_rotation_default() {
        let params = EditParams::default();
        assert!(!needs_crop_rotation(&params));
    }

    #[test]
    fn test_needs_crop_rotation_with_rotation() {
        let mut params = EditParams::default();
        params.rotation = 90;
        assert!(needs_crop_rotation(&params));
    }

    #[test]
    fn test_needs_crop_rotation_with_crop() {
        let mut params = EditParams::default();
        params.crop_width = 0.5;
        assert!(needs_crop_rotation(&params));
    }

    #[test]
    fn test_apply_crop_rotation_no_op() {
        let input = make_2x2_image();
        let params = EditParams::default();
        let (out, w, h) = apply_crop_rotation(input.clone(), 2, 2, &params);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        assert_eq!(out, input);
    }

    #[test]
    fn test_apply_crop_then_rotate() {
        let input = make_2x2_image();
        let mut params = EditParams::default();
        params.crop_x = 0.0;
        params.crop_y = 0.0;
        params.crop_width = 0.5;
        params.crop_height = 1.0;
        params.rotation = 90;
        let (_out, w, h) = apply_crop_rotation(input, 2, 2, &params);
        // Crop to left column (1x2), then rotate 90° → 2x1
        assert_eq!(w, 2);
        assert_eq!(h, 1);
    }

    #[test]
    fn test_rotate_fine_zero_is_noop() {
        let input = vec![255u8; 4 * 4 * 4]; // 4x4 white
        let (out, w, h) = rotate_fine(&input, 4, 4, 0.0);
        assert_eq!(w, 4);
        assert_eq!(h, 4);
        assert_eq!(out, input);
    }

    #[test]
    fn test_rotate_fine_produces_smaller_image() {
        // 100x100 image rotated by 10° should produce inscribed rect smaller than 100x100
        let input = vec![128u8; 100 * 100 * 4];
        let (_, w, h) = rotate_fine(&input, 100, 100, 10.0);
        assert!(w < 100 && w > 50, "width {w} should be between 50 and 100");
        assert!(h < 100 && h > 50, "height {h} should be between 50 and 100");
        assert_eq!(w, h); // square input → square output
    }

    #[test]
    fn test_rotate_fine_45_deg_square() {
        // At 45° a square inscribed rectangle has side = W / (cos45 + sin45) = W / sqrt(2)
        let input = vec![0u8; 100 * 100 * 4];
        let (_, w, h) = rotate_fine(&input, 100, 100, 45.0);
        let expected = (100.0 / 2.0_f64.sqrt()).round() as u32;
        assert!((w as i32 - expected as i32).unsigned_abs() <= 1, "w={w} expected ~{expected}");
        assert!((h as i32 - expected as i32).unsigned_abs() <= 1, "h={h} expected ~{expected}");
    }

    #[test]
    fn test_needs_crop_rotation_with_fine() {
        let mut params = EditParams::default();
        assert!(!needs_crop_rotation(&params));
        params.rotation_fine = 5.0;
        assert!(needs_crop_rotation(&params));
    }
}
