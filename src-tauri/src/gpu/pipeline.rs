use std::sync::mpsc;

use rayon::prelude::*;

use crate::catalog::models::EditParams;
use crate::gpu::context::GpuContext;
use crate::gpu::passes::basic_adjustments::BasicAdjustmentsParams;

const PARALLEL_PIXEL_THRESHOLD: usize = 512 * 512;

#[derive(Clone, Copy)]
struct CpuEditProfile {
    apply_white_balance: bool,
    temp_red_scale: f32,
    temp_blue_scale: f32,
    tint_green_scale: f32,
    apply_exposure: bool,
    exposure_scale: f32,
    apply_contrast: bool,
    contrast_scale: f32,
    apply_tone_regions: bool,
    highlights_factor: f32,
    shadows_factor: f32,
    whites_factor: f32,
    blacks_factor: f32,
    apply_saturation: bool,
    saturation_scale: f32,
    apply_vibrance: bool,
    vibrance_scale: f32,
    apply_dehaze: bool,
    dehaze_scale: f32,
}

impl CpuEditProfile {
    fn from_params(params: &EditParams) -> Self {
        let temp_shift = (params.temperature - 6500.0) / 6500.0;
        let temp_red_scale = 1.0 + temp_shift * 0.1;
        let temp_blue_scale = 1.0 - temp_shift * 0.1;
        let tint_green_scale = 1.0 + params.tint / 150.0 * 0.05;
        let exposure_scale = (2.0_f32).powf(params.exposure);
        let contrast_scale = 1.0 + params.contrast / 100.0;
        let highlights_factor = params.highlights / 200.0;
        let shadows_factor = params.shadows / 200.0;
        let whites_factor = params.whites / 200.0;
        let blacks_factor = params.blacks / 200.0;
        let saturation_scale = 1.0 + params.saturation / 100.0;
        let vibrance_scale = params.vibrance / 100.0;
        let dehaze_scale = params.dehaze / 100.0;

        let apply_tone_regions = params.highlights.abs() > 0.001
            || params.shadows.abs() > 0.001
            || params.whites.abs() > 0.001
            || params.blacks.abs() > 0.001;

        Self {
            apply_white_balance: temp_shift.abs() > 0.001 || params.tint.abs() > 0.001,
            temp_red_scale,
            temp_blue_scale,
            tint_green_scale,
            apply_exposure: params.exposure.abs() > 0.001,
            exposure_scale,
            apply_contrast: params.contrast.abs() > 0.001,
            contrast_scale,
            apply_tone_regions,
            highlights_factor,
            shadows_factor,
            whites_factor,
            blacks_factor,
            apply_saturation: params.saturation.abs() > 0.001,
            saturation_scale,
            apply_vibrance: params.vibrance.abs() > 0.001,
            vibrance_scale,
            apply_dehaze: params.dehaze.abs() > 0.01,
            dehaze_scale,
        }
    }

    fn is_neutral(self) -> bool {
        !self.apply_white_balance
            && !self.apply_exposure
            && !self.apply_contrast
            && !self.apply_tone_regions
            && !self.apply_saturation
            && !self.apply_vibrance
            && !self.apply_dehaze
    }
}

fn apply_edits_to_pixel(pixel: &mut [u8], profile: CpuEditProfile) {
    let mut r = pixel[0] as f32 / 255.0;
    let mut g = pixel[1] as f32 / 255.0;
    let mut b = pixel[2] as f32 / 255.0;

    if profile.apply_white_balance {
        r *= profile.temp_red_scale;
        b *= profile.temp_blue_scale;
        g *= profile.tint_green_scale;
    }

    if profile.apply_exposure {
        r *= profile.exposure_scale;
        g *= profile.exposure_scale;
        b *= profile.exposure_scale;
    }

    if profile.apply_contrast {
        r = (r - 0.5) * profile.contrast_scale + 0.5;
        g = (g - 0.5) * profile.contrast_scale + 0.5;
        b = (b - 0.5) * profile.contrast_scale + 0.5;
    }

    // Tone regions: smooth power-curve masks instead of hard thresholds
    if profile.apply_tone_regions {
        let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        // Smooth luminance masks
        let hl_mask = lum * lum;                         // highlights
        let sh_mask = (1.0 - lum) * (1.0 - lum);        // shadows
        let w_mask = hl_mask * hl_mask;                  // whites (tighter)
        let b_mask = sh_mask * sh_mask;                  // blacks (tighter)

        let lum_shift = hl_mask * profile.highlights_factor
                      + sh_mask * profile.shadows_factor
                      + w_mask * profile.whites_factor
                      + b_mask * profile.blacks_factor;

        if lum_shift.abs() > 0.0001 {
            let target_lum = (lum + lum_shift).clamp(0.0, 1.5);
            let ratio = if lum < 0.001 { 1.0 + lum_shift } else { target_lum / lum };
            r *= ratio;
            g *= ratio;
            b *= ratio;
        }
    }

    // Saturation
    if profile.apply_saturation {
        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        r = gray + (r - gray) * profile.saturation_scale;
        g = gray + (g - gray) * profile.saturation_scale;
        b = gray + (b - gray) * profile.saturation_scale;
    }

    // Vibrance — recalculate gray after saturation
    if profile.apply_vibrance {
        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let cur_sat = if max_c > 0.0 { (max_c - min_c) / max_c } else { 0.0 };
        let vibrance_factor = 1.0 + profile.vibrance_scale * (1.0 - cur_sat);
        r = gray + (r - gray) * vibrance_factor;
        g = gray + (g - gray) * vibrance_factor;
        b = gray + (b - gray) * vibrance_factor;
    }

    // Dehaze
    if profile.apply_dehaze {
        let min_c = r.min(g).min(b);
        r += (r - min_c) * profile.dehaze_scale;
        g += (g - min_c) * profile.dehaze_scale;
        b += (b - min_c) * profile.dehaze_scale;
    }

    pixel[0] = (r.clamp(0.0, 1.0) * 255.0) as u8;
    pixel[1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
    pixel[2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
}

pub fn apply_edits_cpu(rgba_data: &[u8], params: &EditParams) -> Vec<u8> {
    if rgba_data.is_empty() {
        return Vec::new();
    }

    let profile = CpuEditProfile::from_params(params);
    if profile.is_neutral() {
        return rgba_data.to_vec();
    }

    let mut result = rgba_data.to_vec();
    let pixel_count = result.len() / 4;

    if pixel_count >= PARALLEL_PIXEL_THRESHOLD {
        result
            .par_chunks_exact_mut(4)
            .for_each(|pixel| apply_edits_to_pixel(pixel, profile));
    } else {
        result
            .chunks_exact_mut(4)
            .for_each(|pixel| apply_edits_to_pixel(pixel, profile));
    }

    result
}

pub fn supports_gpu_basic_pipeline(params: &EditParams) -> bool {
    let defaults = EditParams::default();

    params.clarity.abs() < 0.001
        && params.sharpening_amount.abs() < 0.001
        && params.denoise_luminance.abs() < 0.001
        && params.denoise_color.abs() < 0.001
        && !params.denoise_ai
        && params.vignette_amount.abs() < 0.001
        && params.grain_amount.abs() < 0.001
        && params.curve_rgb == defaults.curve_rgb
        && params.curve_r == defaults.curve_r
        && params.curve_g == defaults.curve_g
        && params.curve_b == defaults.curve_b
        && params.hsl_hue == defaults.hsl_hue
        && params.hsl_saturation == defaults.hsl_saturation
        && params.hsl_luminance == defaults.hsl_luminance
}

pub fn apply_edits_with_backend(
    gpu: Option<&mut GpuContext>,
    rgba_data: &[u8],
    width: u32,
    height: u32,
    params: &EditParams,
) -> Vec<u8> {
    if let Some(gpu_ctx) = gpu {
        if supports_gpu_basic_pipeline(params) {
            match apply_edits_gpu_basic(gpu_ctx, rgba_data, width, height, params) {
                Ok(result) => return result,
                Err(err) => log::warn!("GPU pipeline failed, falling back to CPU: {}", err),
            }
        }
    }

    apply_edits_cpu(rgba_data, params)
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
    use crate::catalog::models::EditParams;

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
        let input = vec![128, 128, 128, 255];
        let mut params = EditParams::default();
        params.contrast = 50.0;
        let output = apply_edits_cpu(&input, &params);
        let diff = (output[0] as i32 - 128).abs();
        assert!(diff < 10, "Mid-gray with contrast should stay near 128, got {}", output[0]);
    }

    #[test]
    fn test_saturation_desaturate() {
        let input = vec![255, 0, 0, 255];
        let mut params = EditParams::default();
        params.saturation = -100.0;
        let output = apply_edits_cpu(&input, &params);
        let max_diff = (output[0] as i32 - output[1] as i32).abs()
            .max((output[1] as i32 - output[2] as i32).abs());
        assert!(max_diff < 5, "Desaturated red should be near gray, got ({}, {}, {})", output[0], output[1], output[2]);
    }

    #[test]
    fn test_warm_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 10000.0;
        let output = apply_edits_cpu(&input, &params);
        assert!(output[0] >= output[2], "Warm WB should shift red > blue: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_cool_temperature() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.temperature = 3000.0;
        let output = apply_edits_cpu(&input, &params);
        assert!(output[2] >= output[0], "Cool WB should shift blue > red: R={}, B={}", output[0], output[2]);
    }

    #[test]
    fn test_alpha_preserved() {
        let input = vec![100, 100, 100, 200];
        let mut params = EditParams::default();
        params.exposure = 1.0;
        let output = apply_edits_cpu(&input, &params);
        assert_eq!(output[3], 200, "Alpha should be preserved");
    }

    #[test]
    fn test_output_clamped() {
        let input = vec![250, 250, 250, 255];
        let mut params = EditParams::default();
        params.exposure = 3.0;
        let output = apply_edits_cpu(&input, &params);
        assert_eq!(output[0], 255, "Output should be clamped to 255");
        assert_eq!(output[1], 255);
        assert_eq!(output[2], 255);
    }

    #[test]
    fn test_dehaze_positive() {
        let input = make_gray_image(128, 1);
        let mut params = EditParams::default();
        params.dehaze = 50.0;
        let output = apply_edits_cpu(&input, &params);
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
        let output = apply_edits_cpu(&input, &params);
        assert_eq!(output.len(), input.len());
        for i in 0..4 {
            assert_eq!(output[i * 4 + 3], 255);
        }
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<u8> = vec![];
        let params = EditParams::default();
        let output = apply_edits_cpu(&input, &params);
        assert!(output.is_empty());
    }

    #[test]
    fn test_gpu_supports_basic_pipeline_for_supported_params() {
        let params = EditParams::default();
        assert!(supports_gpu_basic_pipeline(&params));
    }

    #[test]
    fn test_gpu_support_rejects_unsupported_params() {
        let mut params = EditParams::default();
        params.clarity = 10.0;
        assert!(!supports_gpu_basic_pipeline(&params));
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
