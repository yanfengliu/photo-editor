use std::sync::mpsc;

use rayon::prelude::*;

use crate::catalog::models::EditParams;
use crate::gpu::context::GpuContext;
use crate::gpu::cpu_edits::{apply_edits_to_pixel, apply_hsl_to_pixel, CpuEditProfile};
use crate::gpu::curves::CurveLuts;
use crate::gpu::passes::basic_adjustments::BasicAdjustmentsParams;
use crate::gpu::spatial::{
    apply_clarity_cpu, apply_denoise_cpu, apply_grain_cpu, apply_sharpening_cpu,
    apply_vignette_cpu,
};
use crate::imaging::lens_profiles;

/// Lens metadata from EXIF, used for auto-detecting lens profiles
#[derive(Debug, Clone, Default)]
pub struct LensMetadata {
    pub lens_name: Option<String>,
    pub focal_length: Option<f64>,
}

pub const PARALLEL_PIXEL_THRESHOLD: usize = 512 * 512;

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

    let profile = match profile {
        Some(p) => p,
        None => return None,
    };

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
    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut result, width, height, params.sharpening_amount, params.sharpening_radius);
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

    params.clarity.abs() > 0.001
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
    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut data, width, height, params.sharpening_amount, params.sharpening_radius);
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

    if params.sharpening_amount > 0.01 {
        apply_sharpening_cpu(&mut result, width, height, params.sharpening_amount, params.sharpening_radius);
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
}
