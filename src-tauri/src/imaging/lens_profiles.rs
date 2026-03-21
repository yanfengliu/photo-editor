use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensProfile {
    pub lens_id: String,
    pub lens_name: String,
    pub aliases: Vec<String>,
    pub mount: String,
    pub focal_range: (f64, f64),
    pub profiles: Vec<FocalProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocalProfile {
    pub focal_length: f64,
    pub distortion: DistortionCoeffs,
    pub ca: CaCoeffs,
    pub vignette: VignetteCoeffs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DistortionCoeffs {
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CaCoeffs {
    pub red_scale: f64,
    pub blue_scale: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VignetteCoeffs {
    pub v1: f64,
    pub v2: f64,
    pub v3: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensProfileSummary {
    pub lens_id: String,
    pub lens_name: String,
    pub mount: String,
    pub focal_range: (f64, f64),
}

impl From<&LensProfile> for LensProfileSummary {
    fn from(p: &LensProfile) -> Self {
        Self {
            lens_id: p.lens_id.clone(),
            lens_name: p.lens_name.clone(),
            mount: p.mount.clone(),
            focal_range: p.focal_range,
        }
    }
}

macro_rules! lens {
    ($id:expr, $name:expr, $aliases:expr, $mount:expr, $range:expr, $profiles:expr) => {
        LensProfile {
            lens_id: $id.into(),
            lens_name: $name.into(),
            aliases: $aliases.iter().map(|s: &&str| s.to_string()).collect(),
            mount: $mount.into(),
            focal_range: $range,
            profiles: $profiles,
        }
    };
}

macro_rules! fp {
    ($fl:expr, $k1:expr, $k2:expr, $k3:expr, $rs:expr, $bs:expr, $v1:expr, $v2:expr, $v3:expr) => {
        FocalProfile {
            focal_length: $fl,
            distortion: DistortionCoeffs { k1: $k1, k2: $k2, k3: $k3 },
            ca: CaCoeffs { red_scale: $rs, blue_scale: $bs },
            vignette: VignetteCoeffs { v1: $v1, v2: $v2, v3: $v3 },
        }
    };
}

static PROFILES: LazyLock<Vec<LensProfile>> = LazyLock::new(|| {
    vec![
        // Canon EF
        lens!("canon-ef-24-70-2.8-ii", "Canon EF 24-70mm f/2.8L II USM",
            &["EF24-70mm f/2.8L II USM", "Canon EF 24-70mm f/2.8L II"],
            "EF", (24.0, 70.0), vec![
                fp!(24.0, -0.035, 0.012, -0.003, 1.0005, 0.9995, 0.8, -0.3, 0.05),
                fp!(35.0, -0.010, 0.003, 0.0, 1.0003, 0.9997, 0.5, -0.15, 0.02),
                fp!(50.0, 0.005, -0.002, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(70.0, 0.012, -0.005, 0.001, 1.0002, 0.9998, 0.4, -0.1, 0.02),
            ]),
        lens!("canon-ef-70-200-2.8-iii", "Canon EF 70-200mm f/2.8L IS III USM",
            &["EF70-200mm f/2.8L IS III USM", "Canon EF 70-200mm f/2.8L IS III"],
            "EF", (70.0, 200.0), vec![
                fp!(70.0, 0.005, -0.002, 0.0, 1.0003, 0.9997, 0.4, -0.1, 0.02),
                fp!(100.0, 0.008, -0.003, 0.001, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(135.0, 0.012, -0.004, 0.001, 1.0002, 0.9998, 0.35, -0.1, 0.02),
                fp!(200.0, 0.018, -0.006, 0.002, 1.0003, 0.9997, 0.5, -0.15, 0.03),
            ]),
        lens!("canon-ef-50-1.4", "Canon EF 50mm f/1.4 USM",
            &["EF50mm f/1.4 USM", "Canon EF 50mm f/1.4"],
            "EF", (50.0, 50.0), vec![
                fp!(50.0, 0.003, -0.001, 0.0, 1.0003, 0.9997, 0.6, -0.2, 0.03),
            ]),
        lens!("canon-efs-18-55-stm", "Canon EF-S 18-55mm f/3.5-5.6 IS STM",
            &["EF-S18-55mm f/3.5-5.6 IS STM", "Canon EF-S 18-55mm"],
            "EF-S", (18.0, 55.0), vec![
                fp!(18.0, -0.060, 0.025, -0.008, 1.0008, 0.9992, 1.2, -0.5, 0.1),
                fp!(35.0, -0.008, 0.002, 0.0, 1.0004, 0.9996, 0.6, -0.2, 0.03),
                fp!(55.0, 0.015, -0.005, 0.001, 1.0003, 0.9997, 0.5, -0.15, 0.03),
            ]),
        // Canon RF
        lens!("canon-rf-24-105-4", "Canon RF 24-105mm f/4L IS USM",
            &["RF24-105mm F4 L IS USM", "Canon RF 24-105mm f/4L"],
            "RF", (24.0, 105.0), vec![
                fp!(24.0, -0.030, 0.010, -0.002, 1.0004, 0.9996, 0.7, -0.25, 0.04),
                fp!(50.0, 0.003, -0.001, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(105.0, 0.010, -0.004, 0.001, 1.0002, 0.9998, 0.4, -0.12, 0.02),
            ]),
        lens!("canon-rf-50-1.2", "Canon RF 50mm f/1.2L USM",
            &["RF50mm F1.2 L USM", "Canon RF 50mm f/1.2L"],
            "RF", (50.0, 50.0), vec![
                fp!(50.0, 0.002, -0.001, 0.0, 1.0002, 0.9998, 0.5, -0.15, 0.02),
            ]),
        // Nikon F
        lens!("nikon-24-70-2.8e", "Nikon AF-S NIKKOR 24-70mm f/2.8E ED VR",
            &["AF-S NIKKOR 24-70mm f/2.8E ED VR", "Nikon AF-S 24-70mm f/2.8E"],
            "F", (24.0, 70.0), vec![
                fp!(24.0, -0.032, 0.011, -0.003, 1.0004, 0.9996, 0.75, -0.28, 0.05),
                fp!(35.0, -0.008, 0.002, 0.0, 1.0003, 0.9997, 0.45, -0.12, 0.02),
                fp!(50.0, 0.004, -0.001, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(70.0, 0.010, -0.004, 0.001, 1.0002, 0.9998, 0.35, -0.1, 0.02),
            ]),
        lens!("nikon-50-1.8g", "Nikon AF-S NIKKOR 50mm f/1.8G",
            &["AF-S NIKKOR 50mm f/1.8G", "Nikon 50mm f/1.8G"],
            "F", (50.0, 50.0), vec![
                fp!(50.0, 0.004, -0.002, 0.0, 1.0003, 0.9997, 0.55, -0.18, 0.03),
            ]),
        lens!("nikon-afp-18-55", "Nikon AF-P DX NIKKOR 18-55mm f/3.5-5.6G VR",
            &["AF-P DX NIKKOR 18-55mm f/3.5-5.6G VR", "Nikon AF-P 18-55mm"],
            "F-DX", (18.0, 55.0), vec![
                fp!(18.0, -0.055, 0.022, -0.007, 1.0007, 0.9993, 1.1, -0.45, 0.09),
                fp!(35.0, -0.006, 0.002, 0.0, 1.0004, 0.9996, 0.55, -0.18, 0.03),
                fp!(55.0, 0.014, -0.005, 0.001, 1.0003, 0.9997, 0.45, -0.12, 0.02),
            ]),
        // Nikon Z
        lens!("nikon-z-24-70-4", "Nikon NIKKOR Z 24-70mm f/4 S",
            &["NIKKOR Z 24-70mm f/4 S", "Nikon Z 24-70mm f/4 S"],
            "Z", (24.0, 70.0), vec![
                fp!(24.0, -0.028, 0.009, -0.002, 1.0003, 0.9997, 0.65, -0.22, 0.04),
                fp!(50.0, 0.003, -0.001, 0.0, 1.0002, 0.9998, 0.28, -0.07, 0.01),
                fp!(70.0, 0.008, -0.003, 0.001, 1.0002, 0.9998, 0.32, -0.09, 0.01),
            ]),
        lens!("nikon-z-50-1.8", "Nikon NIKKOR Z 50mm f/1.8 S",
            &["NIKKOR Z 50mm f/1.8 S", "Nikon Z 50mm f/1.8 S"],
            "Z", (50.0, 50.0), vec![
                fp!(50.0, 0.002, -0.001, 0.0, 1.0002, 0.9998, 0.4, -0.12, 0.02),
            ]),
        // Sony E
        lens!("sony-fe-24-70-2.8-gm", "Sony FE 24-70mm f/2.8 GM",
            &["FE 24-70mm F2.8 GM", "Sony FE 24-70mm f/2.8 GM", "SEL2470GM"],
            "E", (24.0, 70.0), vec![
                fp!(24.0, -0.030, 0.010, -0.002, 1.0004, 0.9996, 0.7, -0.25, 0.04),
                fp!(35.0, -0.008, 0.002, 0.0, 1.0003, 0.9997, 0.4, -0.12, 0.02),
                fp!(50.0, 0.004, -0.002, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(70.0, 0.010, -0.004, 0.001, 1.0002, 0.9998, 0.35, -0.1, 0.02),
            ]),
        lens!("sony-fe-70-200-2.8-gm", "Sony FE 70-200mm f/2.8 GM OSS",
            &["FE 70-200mm F2.8 GM OSS", "SEL70200GM"],
            "E", (70.0, 200.0), vec![
                fp!(70.0, 0.005, -0.002, 0.0, 1.0003, 0.9997, 0.4, -0.1, 0.02),
                fp!(100.0, 0.008, -0.003, 0.001, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(200.0, 0.016, -0.006, 0.002, 1.0003, 0.9997, 0.45, -0.13, 0.03),
            ]),
        lens!("sony-fe-50-1.4-gm", "Sony FE 50mm f/1.4 GM",
            &["FE 50mm F1.4 GM", "SEL50F14GM"],
            "E", (50.0, 50.0), vec![
                fp!(50.0, 0.002, -0.001, 0.0, 1.0002, 0.9998, 0.45, -0.13, 0.02),
            ]),
        lens!("sony-fe-24-105-4-g", "Sony FE 24-105mm f/4 G OSS",
            &["FE 24-105mm F4 G OSS", "SEL24105G"],
            "E", (24.0, 105.0), vec![
                fp!(24.0, -0.028, 0.009, -0.002, 1.0004, 0.9996, 0.7, -0.24, 0.04),
                fp!(50.0, 0.003, -0.001, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(105.0, 0.012, -0.004, 0.001, 1.0002, 0.9998, 0.4, -0.12, 0.02),
            ]),
        lens!("sony-e-16-50", "Sony E 16-50mm f/3.5-5.6 PZ OSS",
            &["E PZ 16-50mm F3.5-5.6 OSS", "SELP1650"],
            "E-APS", (16.0, 50.0), vec![
                fp!(16.0, -0.065, 0.028, -0.009, 1.0009, 0.9991, 1.3, -0.55, 0.12),
                fp!(35.0, -0.005, 0.001, 0.0, 1.0004, 0.9996, 0.5, -0.15, 0.02),
                fp!(50.0, 0.012, -0.004, 0.001, 1.0003, 0.9997, 0.4, -0.1, 0.02),
            ]),
        // Sigma Art
        lens!("sigma-35-1.4-art", "Sigma 35mm f/1.4 DG HSM Art",
            &["35mm F1.4 DG HSM | Art", "Sigma 35mm f/1.4 Art"],
            "Multi", (35.0, 35.0), vec![
                fp!(35.0, -0.005, 0.001, 0.0, 1.0003, 0.9997, 0.5, -0.15, 0.02),
            ]),
        lens!("sigma-50-1.4-art", "Sigma 50mm f/1.4 DG HSM Art",
            &["50mm F1.4 DG HSM | Art", "Sigma 50mm f/1.4 Art"],
            "Multi", (50.0, 50.0), vec![
                fp!(50.0, 0.003, -0.001, 0.0, 1.0002, 0.9998, 0.45, -0.13, 0.02),
            ]),
        lens!("sigma-24-70-2.8-art", "Sigma 24-70mm f/2.8 DG DN Art",
            &["24-70mm F2.8 DG DN | Art", "Sigma 24-70mm f/2.8 Art"],
            "Multi", (24.0, 70.0), vec![
                fp!(24.0, -0.028, 0.009, -0.002, 1.0004, 0.9996, 0.65, -0.22, 0.04),
                fp!(35.0, -0.006, 0.002, 0.0, 1.0003, 0.9997, 0.4, -0.12, 0.02),
                fp!(50.0, 0.004, -0.002, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(70.0, 0.010, -0.004, 0.001, 1.0002, 0.9998, 0.35, -0.1, 0.02),
            ]),
        lens!("sigma-18-35-1.8-art", "Sigma 18-35mm f/1.8 DC HSM Art",
            &["18-35mm F1.8 DC HSM | Art", "Sigma 18-35mm f/1.8 Art"],
            "Multi-APS", (18.0, 35.0), vec![
                fp!(18.0, -0.045, 0.018, -0.005, 1.0006, 0.9994, 1.0, -0.4, 0.08),
                fp!(24.0, -0.020, 0.007, -0.002, 1.0004, 0.9996, 0.7, -0.25, 0.04),
                fp!(35.0, -0.005, 0.001, 0.0, 1.0003, 0.9997, 0.5, -0.15, 0.02),
            ]),
        lens!("sigma-100-400", "Sigma 100-400mm f/5-6.3 DG DN OS",
            &["100-400mm F5-6.3 DG DN OS", "Sigma 100-400mm"],
            "Multi", (100.0, 400.0), vec![
                fp!(100.0, 0.006, -0.002, 0.0, 1.0002, 0.9998, 0.35, -0.1, 0.02),
                fp!(200.0, 0.014, -0.005, 0.001, 1.0003, 0.9997, 0.4, -0.12, 0.02),
                fp!(400.0, 0.022, -0.008, 0.002, 1.0004, 0.9996, 0.5, -0.15, 0.03),
            ]),
        // Tamron
        lens!("tamron-28-75-2.8-g2", "Tamron 28-75mm f/2.8 Di III VXD G2",
            &["28-75mm F/2.8 Di III VXD G2", "Tamron 28-75mm f/2.8 G2"],
            "E", (28.0, 75.0), vec![
                fp!(28.0, -0.025, 0.008, -0.002, 1.0004, 0.9996, 0.6, -0.2, 0.03),
                fp!(50.0, 0.004, -0.002, 0.0, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(75.0, 0.010, -0.004, 0.001, 1.0002, 0.9998, 0.35, -0.1, 0.02),
            ]),
        lens!("tamron-70-180-2.8", "Tamron 70-180mm f/2.8 Di III VXD",
            &["70-180mm F/2.8 Di III VXD", "Tamron 70-180mm f/2.8"],
            "E", (70.0, 180.0), vec![
                fp!(70.0, 0.006, -0.002, 0.0, 1.0003, 0.9997, 0.4, -0.12, 0.02),
                fp!(100.0, 0.009, -0.003, 0.001, 1.0002, 0.9998, 0.3, -0.08, 0.01),
                fp!(180.0, 0.016, -0.006, 0.002, 1.0003, 0.9997, 0.45, -0.13, 0.03),
            ]),
        lens!("tamron-17-28-2.8", "Tamron 17-28mm f/2.8 Di III RXD",
            &["17-28mm F/2.8 Di III RXD", "Tamron 17-28mm f/2.8"],
            "E", (17.0, 28.0), vec![
                fp!(17.0, -0.050, 0.020, -0.006, 1.0007, 0.9993, 1.0, -0.4, 0.08),
                fp!(24.0, -0.025, 0.008, -0.002, 1.0004, 0.9996, 0.65, -0.22, 0.04),
                fp!(28.0, -0.015, 0.005, -0.001, 1.0003, 0.9997, 0.5, -0.15, 0.02),
            ]),
    ]
});

pub fn get_all_profiles() -> &'static [LensProfile] {
    &PROFILES
}

pub fn get_all_summaries() -> Vec<LensProfileSummary> {
    PROFILES.iter().map(LensProfileSummary::from).collect()
}

pub fn find_profile_by_id(id: &str) -> Option<&'static LensProfile> {
    PROFILES.iter().find(|p| p.lens_id == id)
}

/// Auto-detect a lens profile from EXIF lens name string
pub fn find_profile_by_name(lens_name: &str) -> Option<&'static LensProfile> {
    let needle = normalize(lens_name);
    if needle.is_empty() { return None; }

    // Exact alias match first
    for profile in PROFILES.iter() {
        if normalize(&profile.lens_name) == needle {
            return Some(profile);
        }
        for alias in &profile.aliases {
            if normalize(alias) == needle {
                return Some(profile);
            }
        }
    }

    // Fuzzy token-based matching
    let needle_tokens = tokenize(&needle);
    let mut best: Option<(&LensProfile, f64)> = None;

    for profile in PROFILES.iter() {
        let mut max_score = token_similarity(&needle_tokens, &tokenize(&normalize(&profile.lens_name)));
        for alias in &profile.aliases {
            let score = token_similarity(&needle_tokens, &tokenize(&normalize(alias)));
            if score > max_score { max_score = score; }
        }
        if max_score > 0.5 {
            if best.is_none() || max_score > best.unwrap().1 {
                best = Some((profile, max_score));
            }
        }
    }

    best.map(|(p, _)| p)
}

/// Interpolate correction coefficients for a specific focal length
pub fn interpolate_focal(profile: &LensProfile, focal_length: f64) -> FocalProfile {
    let fps = &profile.profiles;
    if fps.is_empty() {
        return FocalProfile {
            focal_length,
            distortion: DistortionCoeffs { k1: 0.0, k2: 0.0, k3: 0.0 },
            ca: CaCoeffs { red_scale: 1.0, blue_scale: 1.0 },
            vignette: VignetteCoeffs { v1: 0.0, v2: 0.0, v3: 0.0 },
        };
    }
    if fps.len() == 1 || focal_length <= fps[0].focal_length {
        return fps[0].clone();
    }
    let last = &fps[fps.len() - 1];
    if focal_length >= last.focal_length {
        return last.clone();
    }

    // Find bracketing pair
    let mut lo = 0;
    for i in 0..fps.len() - 1 {
        if focal_length >= fps[i].focal_length && focal_length <= fps[i + 1].focal_length {
            lo = i;
            break;
        }
    }
    let hi = lo + 1;
    let range = fps[hi].focal_length - fps[lo].focal_length;
    let t = if range.abs() < 1e-6 { 0.0 } else { (focal_length - fps[lo].focal_length) / range };

    let lerp = |a: f64, b: f64| a + (b - a) * t;
    FocalProfile {
        focal_length,
        distortion: DistortionCoeffs {
            k1: lerp(fps[lo].distortion.k1, fps[hi].distortion.k1),
            k2: lerp(fps[lo].distortion.k2, fps[hi].distortion.k2),
            k3: lerp(fps[lo].distortion.k3, fps[hi].distortion.k3),
        },
        ca: CaCoeffs {
            red_scale: lerp(fps[lo].ca.red_scale, fps[hi].ca.red_scale),
            blue_scale: lerp(fps[lo].ca.blue_scale, fps[hi].ca.blue_scale),
        },
        vignette: VignetteCoeffs {
            v1: lerp(fps[lo].vignette.v1, fps[hi].vignette.v1),
            v2: lerp(fps[lo].vignette.v2, fps[hi].vignette.v2),
            v3: lerp(fps[lo].vignette.v3, fps[hi].vignette.v3),
        },
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '.' && c != '-', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize(s: &str) -> Vec<String> {
    s.split_whitespace().map(|t| t.to_string()).collect()
}

fn token_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    // Count matches: a token matches if it equals or is a prefix/suffix of a token in the other set
    let matches = a.iter().filter(|at| {
        b.iter().any(|bt| {
            at.as_str() == bt.as_str() || bt.starts_with(at.as_str()) || at.starts_with(bt.as_str())
        })
    }).count() as f64;
    let total = a.len().max(b.len()) as f64;
    matches / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_count() {
        assert!(get_all_profiles().len() >= 20);
    }

    #[test]
    fn test_find_by_id() {
        let p = find_profile_by_id("canon-ef-24-70-2.8-ii");
        assert!(p.is_some());
        assert_eq!(p.unwrap().lens_name, "Canon EF 24-70mm f/2.8L II USM");
    }

    #[test]
    fn test_find_by_exact_alias() {
        let p = find_profile_by_name("EF24-70mm f/2.8L II USM");
        assert!(p.is_some());
        assert!(p.unwrap().lens_id.contains("canon-ef-24-70"));
    }

    #[test]
    fn test_find_fuzzy() {
        let p = find_profile_by_name("Canon EF 24-70 f2.8 L II");
        assert!(p.is_some());
    }

    #[test]
    fn test_no_match() {
        let p = find_profile_by_name("Totally Unknown Lens XYZ");
        assert!(p.is_none());
    }

    #[test]
    fn test_interpolation_exact() {
        let profile = find_profile_by_id("canon-ef-24-70-2.8-ii").unwrap();
        let fp = interpolate_focal(profile, 24.0);
        assert!((fp.distortion.k1 - (-0.035)).abs() < 0.001);
    }

    #[test]
    fn test_interpolation_midpoint() {
        let profile = find_profile_by_id("canon-ef-24-70-2.8-ii").unwrap();
        let fp = interpolate_focal(profile, 29.5);
        // Midpoint between 24 and 35
        assert!(fp.distortion.k1 < 0.0); // Still negative (barrel)
        assert!(fp.distortion.k1 > -0.035); // Less than at 24mm
    }
}
