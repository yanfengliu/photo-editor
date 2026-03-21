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
    contrast_exponent: f32,
    pub(crate) apply_tone_regions: bool,
    highlights_factor: f32,
    shadows_factor: f32,
    whites_factor: f32,
    blacks_factor: f32,
    pub(crate) apply_saturation: bool,
    saturation_scale: f32,
    pub(crate) apply_vibrance: bool,
    vibrance_scale: f32,
    pub(crate) apply_hsl: bool,
    pub(crate) hsl_hue: [f32; 8],
    pub(crate) hsl_saturation: [f32; 8],
    pub(crate) hsl_luminance: [f32; 8],
}

/// Compute white-balance channel multipliers from color temperature.
///
/// Uses Hernandez-Andres et al. (1999) rational polynomial to map a correlated
/// color temperature (CCT) to CIE 1931 xy chromaticity on the Planckian locus,
/// then converts both source and D65 illuminants to linear sRGB and returns
/// per-channel correction ratios (normalized so green = 1).
///
/// Returns (red_scale, blue_scale) relative to D65 neutral.
pub(crate) fn planckian_wb_scales(temperature_k: f32) -> (f32, f32) {
    // Compute CIE xy for both source and reference (6500K) using the same
    // polynomial, so they cancel exactly at the neutral point.
    let src_xy = planckian_cie_xy(temperature_k.clamp(2000.0, 25000.0));
    let ref_xy = planckian_cie_xy(6500.0);

    let src_rgb = cie_xy_to_linear_srgb(src_xy.0, src_xy.1);
    let ref_rgb = cie_xy_to_linear_srgb(ref_xy.0, ref_xy.1);

    // Per-channel correction ratio, normalized to green=1
    // Temperature adjusts blue-amber axis without changing overall brightness
    let r_ratio = if src_rgb.0.abs() > 1e-6 { ref_rgb.0 / src_rgb.0 } else { 1.0 };
    let g_ratio = if src_rgb.1.abs() > 1e-6 { ref_rgb.1 / src_rgb.1 } else { 1.0 };
    let b_ratio = if src_rgb.2.abs() > 1e-6 { ref_rgb.2 / src_rgb.2 } else { 1.0 };

    (r_ratio / g_ratio, b_ratio / g_ratio)
}

/// CIE 1931 xy chromaticity on the Planckian locus via Hernandez-Andres et al. (1999).
fn planckian_cie_xy(t: f32) -> (f32, f32) {
    let x = if t <= 4000.0 {
        -0.2661239e9 / (t * t * t) - 0.2343589e6 / (t * t) + 0.8776956e3 / t + 0.179910
    } else {
        -3.0258469e9 / (t * t * t) + 2.1070379e6 / (t * t) + 0.2226347e3 / t + 0.240390
    };
    let y = if t <= 2222.0 {
        -1.1063814 * x * x * x - 1.34811020 * x * x + 2.18555832 * x - 0.20219683
    } else if t <= 4000.0 {
        -0.9549476 * x * x * x - 1.37418593 * x * x + 2.09137015 * x - 0.16748867
    } else {
        3.0817580 * x * x * x - 5.87338670 * x * x + 3.75112997 * x - 0.37001483
    };
    (x, y)
}

/// Convert CIE xy chromaticity (Y=1) to linear sRGB via IEC 61966-2-1 matrix.
fn cie_xy_to_linear_srgb(x: f32, y: f32) -> (f32, f32, f32) {
    let big_x = x / y;
    let big_z = (1.0 - x - y) / y;
    (
         3.2406 * big_x - 1.5372 - 0.4986 * big_z,
        -0.9689 * big_x + 1.8758 + 0.0415 * big_z,
         0.0557 * big_x - 0.2040 + 1.0570 * big_z,
    )
}

impl CpuEditProfile {
    pub(crate) fn from_params(params: &EditParams) -> Self {
        let (temp_red_scale, temp_blue_scale) = planckian_wb_scales(params.temperature);
        let tint_green_scale = 1.0 + params.tint / 150.0 * 0.05;
        let exposure_scale = (2.0_f32).powf(params.exposure);
        let contrast_exponent = (1.0 + params.contrast / 100.0).max(0.01);
        let highlights_factor = params.highlights / 200.0;
        let shadows_factor = params.shadows / 200.0;
        let whites_factor = params.whites / 200.0;
        let blacks_factor = params.blacks / 200.0;
        let saturation_scale = 1.0 + params.saturation / 100.0;
        let vibrance_scale = params.vibrance / 100.0;
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
            apply_white_balance: (params.temperature - 6500.0).abs() > 1.0 || params.tint.abs() > 0.001,
            temp_red_scale,
            temp_blue_scale,
            tint_green_scale,
            apply_exposure: params.exposure.abs() > 0.001,
            exposure_scale,
            apply_contrast: params.contrast.abs() > 0.001,
            contrast_exponent,
            apply_tone_regions,
            highlights_factor,
            shadows_factor,
            whites_factor,
            blacks_factor,
            apply_saturation: params.saturation.abs() > 0.001,
            saturation_scale,
            apply_vibrance: params.vibrance.abs() > 0.001,
            vibrance_scale,
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
            && !self.apply_hsl
    }
}

/// Symmetric S-curve: maps [0,1]→[0,1] through (0,0), (0.5,0.5), (1,1).
/// Exponent > 1 steepens midtones (more contrast), < 1 flattens (less contrast).
#[inline]
fn s_curve_contrast(c: f32, exponent: f32) -> f32 {
    let d = (c - 0.5).abs();
    let t = (1.0 - 2.0 * d).max(0.0);
    let curved = 1.0 - t.powf(exponent);
    0.5 + (c - 0.5).signum() * 0.5 * curved
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

    // Contrast — symmetric S-curve (power curve pivot at 0.5)
    if profile.apply_contrast {
        r = s_curve_contrast(r, profile.contrast_exponent);
        g = s_curve_contrast(g, profile.contrast_exponent);
        b = s_curve_contrast(b, profile.contrast_exponent);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// At D65 (6500K), the adaptation should be nearly identity (scales ≈ 1.0).
    #[test]
    fn wb_d65_neutral() {
        let (r, b) = planckian_wb_scales(6500.0);
        assert!((r - 1.0).abs() < 0.02, "D65 red scale {r} should be ~1.0");
        assert!((b - 1.0).abs() < 0.02, "D65 blue scale {b} should be ~1.0");
    }

    /// Illuminant A (2856K, tungsten). Correcting warm light means boosting blue
    /// and reducing red relative to D65.
    #[test]
    fn wb_illuminant_a() {
        let (r, b) = planckian_wb_scales(2856.0);
        assert!(r < 1.0, "Tungsten red scale {r} should be < 1.0 (reduce warm)");
        assert!(b > 1.0, "Tungsten blue scale {b} should be > 1.0 (boost cool)");
    }

    /// Cool daylight (10000K). Correcting cool light means boosting red,
    /// reducing blue relative to D65.
    #[test]
    fn wb_cool_daylight() {
        let (r, b) = planckian_wb_scales(10000.0);
        assert!(r > 1.0, "Cool red scale {r} should be > 1.0");
        assert!(b < 1.0, "Cool blue scale {b} should be < 1.0");
    }

    /// Verify Hernandez-Andres CIE xy at 6500K is close to published D65.
    /// The polynomial approximates the Planckian locus; D65 is slightly off-locus,
    /// so we allow a tolerance of ~0.01 for y (known limitation).
    #[test]
    fn wb_cie_xy_at_d65() {
        let (x, y) = planckian_cie_xy(6500.0);
        assert!((x - 0.3127).abs() < 0.005, "CIE x at 6500K: {x} should be ~0.3127");
        // y is intentionally relaxed — D65 is off the Planckian locus
        assert!((y - 0.3290).abs() < 0.01, "CIE y at 6500K: {y} should be ~0.329");
    }

    /// Monotonicity: as temperature increases from warm→cool, red scale should
    /// increase and blue scale should decrease.
    #[test]
    fn wb_monotonic_trend() {
        let temps = [2500.0, 3500.0, 5000.0, 6500.0, 8000.0, 12000.0];
        let scales: Vec<(f32, f32)> = temps.iter().map(|&t| planckian_wb_scales(t)).collect();
        for i in 1..scales.len() {
            assert!(
                scales[i].0 >= scales[i - 1].0 - 0.01,
                "Red scale should increase with temperature: {}K={} vs {}K={}",
                temps[i - 1], scales[i - 1].0, temps[i], scales[i].0
            );
            assert!(
                scales[i].1 <= scales[i - 1].1 + 0.01,
                "Blue scale should decrease with temperature: {}K={} vs {}K={}",
                temps[i - 1], scales[i - 1].1, temps[i], scales[i].1
            );
        }
    }

    /// Clamping: extreme values should not produce NaN or infinity.
    #[test]
    fn wb_extreme_values() {
        for &t in &[1000.0, 2000.0, 25000.0, 50000.0] {
            let (r, b) = planckian_wb_scales(t);
            assert!(r.is_finite(), "Red scale at {t}K should be finite, got {r}");
            assert!(b.is_finite(), "Blue scale at {t}K should be finite, got {b}");
            assert!(r > 0.0, "Red scale at {t}K should be positive, got {r}");
            assert!(b > 0.0, "Blue scale at {t}K should be positive, got {b}");
        }
    }

    #[test]
    fn apply_edits_clamps_output() {
        let mut pixel = [255, 255, 255, 255];
        let mut profile = CpuEditProfile::from_params(&crate::catalog::models::EditParams::default());
        profile.apply_exposure = true;
        profile.exposure_scale = 4.0; // over-expose
        apply_edits_to_pixel(&mut pixel, profile);
        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 255);
        assert_eq!(pixel[2], 255);
    }

    #[test]
    fn apply_edits_neutral_noop() {
        let mut pixel = [128, 64, 200, 255];
        let original = pixel;
        let profile = CpuEditProfile::from_params(&crate::catalog::models::EditParams::default());
        assert!(profile.is_neutral());
        apply_edits_to_pixel(&mut pixel, profile);
        assert_eq!(pixel, original);
    }

    #[test]
    fn s_curve_preserves_endpoints_and_midpoint() {
        // Identity at exponent=1
        for v in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let result = s_curve_contrast(v, 1.0);
            assert!((result - v).abs() < 0.001, "Identity: s_curve({v}, 1) = {result}");
        }
        // Fixed points at 0, 0.5, 1 for any exponent
        for exp in [0.5, 1.5, 2.0] {
            assert!((s_curve_contrast(0.0, exp) - 0.0).abs() < 0.001);
            assert!((s_curve_contrast(0.5, exp) - 0.5).abs() < 0.001);
            assert!((s_curve_contrast(1.0, exp) - 1.0).abs() < 0.001);
        }
        // High contrast darkens shadows, brightens highlights
        let dark = s_curve_contrast(0.2, 2.0);
        let bright = s_curve_contrast(0.8, 2.0);
        assert!(dark < 0.2, "S-curve should darken shadows: {dark}");
        assert!(bright > 0.8, "S-curve should brighten highlights: {bright}");
    }
}
