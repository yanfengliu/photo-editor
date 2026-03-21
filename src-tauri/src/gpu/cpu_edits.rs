use crate::catalog::models::EditParams;

#[derive(Clone, Copy)]
pub(crate) struct CpuEditProfile {
    pub(crate) apply_white_balance: bool,
    temp_red_scale: f32,
    temp_blue_scale: f32,
    tint_green_scale: f32,
    pub(crate) apply_exposure: bool,
    exposure_scale: f32,
    pub(crate) apply_contrast: bool,
    contrast_scale: f32,
    pub(crate) apply_tone_regions: bool,
    highlights_factor: f32,
    shadows_factor: f32,
    whites_factor: f32,
    blacks_factor: f32,
    pub(crate) apply_saturation: bool,
    saturation_scale: f32,
    pub(crate) apply_vibrance: bool,
    vibrance_scale: f32,
    pub(crate) apply_dehaze: bool,
    dehaze_scale: f32,
    pub(crate) apply_hsl: bool,
    pub(crate) hsl_hue: [f32; 8],
    pub(crate) hsl_saturation: [f32; 8],
    pub(crate) hsl_luminance: [f32; 8],
}

impl CpuEditProfile {
    pub(crate) fn from_params(params: &EditParams) -> Self {
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

        let apply_hsl = params.hsl_hue.iter().any(|v| v.abs() > 0.001)
            || params.hsl_saturation.iter().any(|v| v.abs() > 0.001)
            || params.hsl_luminance.iter().any(|v| v.abs() > 0.001);

        let mut hsl_hue = [0.0f32; 8];
        let mut hsl_saturation = [0.0f32; 8];
        let mut hsl_luminance = [0.0f32; 8];
        for i in 0..8 {
            hsl_hue[i] = params.hsl_hue[i];
            hsl_saturation[i] = params.hsl_saturation[i];
            hsl_luminance[i] = params.hsl_luminance[i];
        }

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
            apply_hsl,
            hsl_hue,
            hsl_saturation,
            hsl_luminance,
        }
    }

    pub(crate) fn is_neutral(self) -> bool {
        !self.apply_white_balance
            && !self.apply_exposure
            && !self.apply_contrast
            && !self.apply_tone_regions
            && !self.apply_saturation
            && !self.apply_vibrance
            && !self.apply_dehaze
            && !self.apply_hsl
    }
}

/// Apply per-pixel basic adjustments (WB, exposure, contrast, tone regions, sat, vibrance, dehaze)
pub(crate) fn apply_edits_to_pixel(pixel: &mut [u8], profile: CpuEditProfile) {
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

/// Apply per-pixel HSL color adjustments
pub(crate) fn apply_hsl_to_pixel(pixel: &mut [u8], profile: CpuEditProfile) {
    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;

    let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);

    for ch in 0..8u32 {
        let w = get_channel_weight(h, ch);
        if w > 0.0 {
            h += profile.hsl_hue[ch as usize] / 360.0 * w;
            s *= 1.0 + profile.hsl_saturation[ch as usize] / 100.0 * w;
            l *= 1.0 + profile.hsl_luminance[ch as usize] / 100.0 * w;
        }
    }

    // Wrap hue, clamp s/l
    h = h.rem_euclid(1.0);
    s = s.clamp(0.0, 1.0);
    l = l.clamp(0.0, 1.0);

    let (nr, ng, nb) = hsl_to_rgb(h, s, l);
    pixel[0] = (nr.clamp(0.0, 1.0) * 255.0) as u8;
    pixel[1] = (ng.clamp(0.0, 1.0) * 255.0) as u8;
    pixel[2] = (nb.clamp(0.0, 1.0) * 255.0) as u8;
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let l = (max_c + min_c) * 0.5;
    if (max_c - min_c).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }
    let d = max_c - min_c;
    let s = if l < 0.5 { d / (max_c + min_c) } else { d / (2.0 - max_c - min_c) };
    let mut h = if (max_c - r).abs() < 1e-6 {
        let mut v = (g - b) / d;
        if g < b { v += 6.0; }
        v
    } else if (max_c - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h /= 6.0;
    (h, s, l)
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    let mut t = t_in;
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn get_channel_weight(hue: f32, channel: u32) -> f32 {
    let center = channel as f32 / 8.0;
    let mut dist = (hue - center).abs();
    dist = dist.min(1.0 - dist); // wrap around
    let width = 1.0 / 8.0;
    (1.0 - dist / width).max(0.0)
}
