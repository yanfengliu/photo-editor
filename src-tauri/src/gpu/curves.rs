use crate::catalog::models::{CurvePoint, EditParams};

/// Build a 256-entry LUT from curve control points using monotone cubic interpolation.
/// Points must be sorted by x. Returns identity LUT if fewer than 2 points.
pub(crate) fn build_curve_lut(points: &[CurvePoint]) -> [f32; 256] {
    let mut lut = [0.0f32; 256];
    let n = points.len();

    if n < 2 {
        // Identity
        for i in 0..256 {
            lut[i] = i as f32 / 255.0;
        }
        return lut;
    }

    // For exactly 2 points, use linear interpolation
    if n == 2 {
        for i in 0..256 {
            let t = i as f32 / 255.0;
            let dx = points[1].x - points[0].x;
            if dx.abs() < 1e-6 {
                lut[i] = points[0].y;
            } else {
                let frac = ((t - points[0].x) / dx).clamp(0.0, 1.0);
                lut[i] = (points[0].y + frac * (points[1].y - points[0].y)).clamp(0.0, 1.0);
            }
        }
        return lut;
    }

    // Monotone cubic (Fritsch-Carlson) spline
    let xs: Vec<f32> = points.iter().map(|p| p.x).collect();
    let ys: Vec<f32> = points.iter().map(|p| p.y).collect();

    // Compute slopes between segments
    let mut deltas = vec![0.0f32; n - 1];
    let mut h = vec![0.0f32; n - 1];
    for i in 0..n - 1 {
        h[i] = xs[i + 1] - xs[i];
        deltas[i] = if h[i].abs() < 1e-10 { 0.0 } else { (ys[i + 1] - ys[i]) / h[i] };
    }

    // Compute tangents with Fritsch-Carlson monotonicity
    let mut m = vec![0.0f32; n];
    m[0] = deltas[0];
    m[n - 1] = deltas[n - 2];
    for i in 1..n - 1 {
        if deltas[i - 1] * deltas[i] <= 0.0 {
            m[i] = 0.0;
        } else {
            m[i] = (deltas[i - 1] + deltas[i]) / 2.0;
        }
    }

    // Enforce monotonicity
    for i in 0..n - 1 {
        if deltas[i].abs() < 1e-10 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let alpha = m[i] / deltas[i];
            let beta = m[i + 1] / deltas[i];
            let s = alpha * alpha + beta * beta;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                m[i] = tau * alpha * deltas[i];
                m[i + 1] = tau * beta * deltas[i];
            }
        }
    }

    // Evaluate spline at each LUT entry
    for i in 0..256 {
        let t = i as f32 / 255.0;

        // Clamp to curve range
        if t <= xs[0] {
            lut[i] = ys[0].clamp(0.0, 1.0);
            continue;
        }
        if t >= xs[n - 1] {
            lut[i] = ys[n - 1].clamp(0.0, 1.0);
            continue;
        }

        // Find segment
        let mut seg = 0;
        for j in 0..n - 1 {
            if t >= xs[j] && t < xs[j + 1] {
                seg = j;
                break;
            }
        }

        let dx = h[seg];
        if dx.abs() < 1e-10 {
            lut[i] = ys[seg].clamp(0.0, 1.0);
            continue;
        }

        let frac = (t - xs[seg]) / dx;
        let frac2 = frac * frac;
        let frac3 = frac2 * frac;

        // Hermite basis
        let h00 = 2.0 * frac3 - 3.0 * frac2 + 1.0;
        let h10 = frac3 - 2.0 * frac2 + frac;
        let h01 = -2.0 * frac3 + 3.0 * frac2;
        let h11 = frac3 - frac2;

        let val = h00 * ys[seg] + h10 * dx * m[seg] + h01 * ys[seg + 1] + h11 * dx * m[seg + 1];
        lut[i] = val.clamp(0.0, 1.0);
    }

    lut
}

/// Check if a curve is the identity (straight line from (0,0) to (1,1))
pub(crate) fn is_identity_curve(points: &[CurvePoint]) -> bool {
    if points.len() != 2 {
        return false;
    }
    (points[0].x - 0.0).abs() < 0.001
        && (points[0].y - 0.0).abs() < 0.001
        && (points[1].x - 1.0).abs() < 0.001
        && (points[1].y - 1.0).abs() < 0.001
}

fn sample_lut(lut: &[f32; 256], val: f32) -> f32 {
    let idx_f = val.clamp(0.0, 1.0) * 255.0;
    let lo = idx_f.floor() as usize;
    let hi = (lo + 1).min(255);
    let frac = idx_f - idx_f.floor();
    lut[lo] * (1.0 - frac) + lut[hi] * frac
}

/// Pre-built LUTs for all 4 curve channels
pub(crate) struct CurveLuts {
    rgb: [f32; 256],
    r: [f32; 256],
    g: [f32; 256],
    b: [f32; 256],
    has_rgb: bool,
    has_r: bool,
    has_g: bool,
    has_b: bool,
}

impl CurveLuts {
    pub(crate) fn from_params(params: &EditParams) -> Self {
        let has_rgb = !is_identity_curve(&params.curve_rgb);
        let has_r = !is_identity_curve(&params.curve_r);
        let has_g = !is_identity_curve(&params.curve_g);
        let has_b = !is_identity_curve(&params.curve_b);
        Self {
            rgb: if has_rgb { build_curve_lut(&params.curve_rgb) } else { [0.0; 256] },
            r: if has_r { build_curve_lut(&params.curve_r) } else { [0.0; 256] },
            g: if has_g { build_curve_lut(&params.curve_g) } else { [0.0; 256] },
            b: if has_b { build_curve_lut(&params.curve_b) } else { [0.0; 256] },
            has_rgb,
            has_r,
            has_g,
            has_b,
        }
    }

    pub(crate) fn is_identity(&self) -> bool {
        !self.has_rgb && !self.has_r && !self.has_g && !self.has_b
    }

    pub(crate) fn apply(&self, pixel: &mut [u8]) {
        let mut r = pixel[0] as f32 / 255.0;
        let mut g = pixel[1] as f32 / 255.0;
        let mut b = pixel[2] as f32 / 255.0;

        // Per-channel curves first, then master RGB
        if self.has_r { r = sample_lut(&self.r, r); }
        if self.has_g { g = sample_lut(&self.g, g); }
        if self.has_b { b = sample_lut(&self.b, b); }
        if self.has_rgb {
            r = sample_lut(&self.rgb, r);
            g = sample_lut(&self.rgb, g);
            b = sample_lut(&self.rgb, b);
        }

        pixel[0] = (r.clamp(0.0, 1.0) * 255.0) as u8;
        pixel[1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
        pixel[2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
    }
}
