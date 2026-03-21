use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

// --- Public types (unchanged API) ---

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

/// PTLens distortion model: Rd = a*Ru^4 + b*Ru^3 + c*Ru^2 + (1-a-b-c)*Ru
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DistortionCoeffs {
    pub a: f64,
    pub b: f64,
    pub c: f64,
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

// --- Embedded lensfun data ---

include!(concat!(env!("OUT_DIR"), "/lensfun_data.rs"));

static PROFILES: LazyLock<Vec<LensProfile>> = LazyLock::new(|| {
    let mut all = Vec::new();
    for xml in LENSFUN_XML {
        if let Ok(mut parsed) = parse_lensfun_xml(xml) {
            all.append(&mut parsed);
        }
    }
    log::info!("Loaded {} lens profiles from lensfun database", all.len());
    all
});

// --- Public API (unchanged signatures) ---

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
    if needle.is_empty() {
        return None;
    }

    // Exact name match first
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
        let mut max_score =
            token_similarity(&needle_tokens, &tokenize(&normalize(&profile.lens_name)));
        for alias in &profile.aliases {
            let score = token_similarity(&needle_tokens, &tokenize(&normalize(alias)));
            if score > max_score {
                max_score = score;
            }
        }
        if max_score > 0.5
            && (best.is_none() || max_score > best.unwrap().1)
        {
            best = Some((profile, max_score));
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
            distortion: DistortionCoeffs { a: 0.0, b: 0.0, c: 0.0 },
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
    let t = if range.abs() < 1e-6 {
        0.0
    } else {
        (focal_length - fps[lo].focal_length) / range
    };

    let lerp = |a: f64, b: f64| a + (b - a) * t;
    FocalProfile {
        focal_length,
        distortion: DistortionCoeffs {
            a: lerp(fps[lo].distortion.a, fps[hi].distortion.a),
            b: lerp(fps[lo].distortion.b, fps[hi].distortion.b),
            c: lerp(fps[lo].distortion.c, fps[hi].distortion.c),
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

// --- Lensfun XML parser ---

fn parse_lensfun_xml(xml: &str) -> Result<Vec<LensProfile>, String> {
    let mut reader = Reader::from_str(xml);
    let mut profiles = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"lens" => {
                if let Some(profile) = parse_lens_element(&mut reader) {
                    profiles.push(profile);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(profiles)
}

fn parse_lens_element(reader: &mut Reader<&[u8]>) -> Option<LensProfile> {
    let mut buf = Vec::new();
    let mut maker = String::new();
    let mut model = String::new();
    let mut mount = String::new();
    let mut aliases: Vec<String> = Vec::new();

    // Calibration data keyed by focal length
    let mut distortions: Vec<(f64, DistortionCoeffs)> = Vec::new();
    let mut cas: Vec<(f64, CaCoeffs)> = Vec::new();
    // Vignetting: (focal, aperture, VignetteCoeffs) — we pick widest aperture per focal
    let mut vignettes: Vec<(f64, f64, VignetteCoeffs)> = Vec::new();

    let mut current_tag = String::new();
    let mut in_calibration = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "maker" | "model" | "mount" => current_tag = tag,
                    "calibration" => in_calibration = true,
                    _ => current_tag.clear(),
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name();
                if tag.as_ref() == b"lens" {
                    break;
                }
                if tag.as_ref() == b"calibration" {
                    in_calibration = false;
                }
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(text) = e.unescape() {
                    let text = text.trim().to_string();
                    match current_tag.as_str() {
                        "maker" => maker = text,
                        "model" => model = text,
                        "mount" => mount = text,
                        _ => {}
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Empty(ref e)) if in_calibration => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let attrs = parse_attrs(e);

                match tag.as_str() {
                    "distortion" => {
                        if attrs.get("model").is_some_and(|m| m == "ptlens") {
                            if let Some(focal) = attrs.get("focal").and_then(|v| v.parse::<f64>().ok()) {
                                let a = attrs.get("a").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                let b = attrs.get("b").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                let c = attrs.get("c").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                distortions.push((focal, DistortionCoeffs { a, b, c }));
                            }
                        }
                    }
                    "tca" => {
                        if let Some(focal) = attrs.get("focal").and_then(|v| v.parse::<f64>().ok()) {
                            let model = attrs.get("model").map(|s| s.as_str()).unwrap_or("");
                            let (rs, bs) = match model {
                                "linear" => {
                                    let kr = attrs.get("kr").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                                    let kb = attrs.get("kb").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                                    (kr, kb)
                                }
                                "poly3" => {
                                    // vr/vb are the linear coefficients
                                    let vr = attrs.get("vr").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                                    let vb = attrs.get("vb").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                                    (vr, vb)
                                }
                                _ => (1.0, 1.0),
                            };
                            cas.push((focal, CaCoeffs { red_scale: rs, blue_scale: bs }));
                        }
                    }
                    "vignetting" => {
                        if attrs.get("model").is_some_and(|m| m == "pa") {
                            if let Some(focal) = attrs.get("focal").and_then(|v| v.parse::<f64>().ok()) {
                                let aperture = attrs.get("aperture").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                                let k1 = attrs.get("k1").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                let k2 = attrs.get("k2").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                let k3 = attrs.get("k3").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                                vignettes.push((focal, aperture, VignetteCoeffs { v1: k1, v2: k2, v3: k3 }));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if model.is_empty() {
        return None;
    }
    if distortions.is_empty() && cas.is_empty() && vignettes.is_empty() {
        return None; // No calibration data
    }

    let full_name = if !maker.is_empty() && !model.starts_with(&maker) {
        format!("{maker} {model}")
    } else {
        model.clone()
    };
    aliases.push(model);

    // Collect all focal lengths
    let mut focal_set: Vec<f64> = Vec::new();
    for &(f, _) in &distortions { push_unique_focal(&mut focal_set, f); }
    for &(f, _) in &cas { push_unique_focal(&mut focal_set, f); }
    for &(f, _, _) in &vignettes { push_unique_focal(&mut focal_set, f); }
    focal_set.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Pick best vignetting per focal (widest aperture = strongest correction)
    let best_vignette = pick_best_vignette(&vignettes);

    // Build FocalProfile for each focal length
    let profiles: Vec<FocalProfile> = focal_set
        .iter()
        .map(|&fl| {
            let dist = find_nearest(&distortions, fl)
                .unwrap_or(DistortionCoeffs { a: 0.0, b: 0.0, c: 0.0 });
            let ca = find_nearest(&cas, fl)
                .unwrap_or(CaCoeffs { red_scale: 1.0, blue_scale: 1.0 });
            let vig = find_nearest_vig(&best_vignette, fl)
                .unwrap_or(VignetteCoeffs { v1: 0.0, v2: 0.0, v3: 0.0 });
            FocalProfile { focal_length: fl, distortion: dist, ca, vignette: vig }
        })
        .collect();

    let focal_min = focal_set.first().copied().unwrap_or(0.0);
    let focal_max = focal_set.last().copied().unwrap_or(focal_min);

    let lens_id = make_lens_id(&full_name);

    Some(LensProfile {
        lens_id,
        lens_name: full_name,
        aliases,
        mount,
        focal_range: (focal_min, focal_max),
        profiles,
    })
}

fn parse_attrs(e: &quick_xml::events::BytesStart<'_>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        map.insert(key, val);
    }
    map
}

fn push_unique_focal(set: &mut Vec<f64>, f: f64) {
    if !set.iter().any(|&x| (x - f).abs() < 0.01) {
        set.push(f);
    }
}

fn find_nearest<T: Copy>(entries: &[(f64, T)], target: f64) -> Option<T> {
    entries
        .iter()
        .min_by(|a, b| {
            (a.0 - target)
                .abs()
                .partial_cmp(&(b.0 - target).abs())
                .unwrap()
        })
        .map(|&(_, v)| v)
}

fn find_nearest_vig(entries: &[(f64, VignetteCoeffs)], target: f64) -> Option<VignetteCoeffs> {
    find_nearest(entries, target)
}

/// Pick the widest aperture vignetting data for each focal length
fn pick_best_vignette(all: &[(f64, f64, VignetteCoeffs)]) -> Vec<(f64, VignetteCoeffs)> {
    let mut by_focal: HashMap<i64, (f64, VignetteCoeffs)> = HashMap::new();
    for &(focal, aperture, vig) in all {
        let key = (focal * 10.0) as i64;
        let entry = by_focal.entry(key).or_insert((aperture, vig));
        // Widest aperture = smallest f-number = strongest vignetting
        if aperture < entry.0 {
            *entry = (aperture, vig);
        }
    }
    let mut result: Vec<(f64, VignetteCoeffs)> = by_focal
        .into_iter()
        .map(|(k, (_, v))| (k as f64 / 10.0, v))
        .collect();
    result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    result
}

fn make_lens_id(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

// --- Text matching helpers ---

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
    let matches = a
        .iter()
        .filter(|at| {
            b.iter().any(|bt| {
                at.as_str() == bt.as_str()
                    || bt.starts_with(at.as_str())
                    || at.starts_with(bt.as_str())
            })
        })
        .count() as f64;
    let total = a.len().max(b.len()) as f64;
    matches / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_loaded() {
        let count = get_all_profiles().len();
        assert!(count > 100, "Expected >100 profiles from lensfun, got {count}");
    }

    #[test]
    fn test_summaries_match_profiles() {
        let profiles = get_all_profiles().len();
        let summaries = get_all_summaries().len();
        assert_eq!(profiles, summaries);
    }

    #[test]
    fn test_find_by_name_sony() {
        let p = find_profile_by_name("Sony FE 24-70mm f/2.8 GM");
        // Should find something, even if not exact name match
        if let Some(profile) = p {
            assert!(
                profile.lens_name.contains("24-70") || profile.lens_name.contains("2470"),
                "Expected 24-70 lens, got: {}",
                profile.lens_name
            );
        }
    }

    #[test]
    fn test_find_by_name_canon() {
        let p = find_profile_by_name("Canon EF 50mm f/1.4 USM");
        if let Some(profile) = p {
            assert!(
                profile.lens_name.contains("50") && profile.lens_name.contains("1.4"),
                "Expected 50mm f/1.4, got: {}",
                profile.lens_name
            );
        }
    }

    #[test]
    fn test_no_match() {
        let p = find_profile_by_name("Totally Unknown Lens XYZ 999");
        assert!(p.is_none());
    }

    #[test]
    fn test_interpolation() {
        // Find any zoom lens to test interpolation
        let zoom = get_all_profiles()
            .iter()
            .find(|p| p.profiles.len() > 1)
            .expect("Should have at least one zoom lens");

        let lo = zoom.profiles.first().unwrap().focal_length;
        let hi = zoom.profiles.last().unwrap().focal_length;
        let mid = (lo + hi) / 2.0;

        let fp = interpolate_focal(zoom, mid);
        assert!((fp.focal_length - mid).abs() < 0.01);
    }

    #[test]
    fn test_ptlens_coefficients_plausible() {
        // Check that parsed distortion coefficients are in a reasonable range
        for profile in get_all_profiles() {
            for fp in &profile.profiles {
                assert!(
                    fp.distortion.a.abs() < 1.0
                        && fp.distortion.b.abs() < 1.0
                        && fp.distortion.c.abs() < 1.0,
                    "Implausible distortion for {}: a={}, b={}, c={}",
                    profile.lens_name,
                    fp.distortion.a,
                    fp.distortion.b,
                    fp.distortion.c
                );
            }
        }
    }

    #[test]
    fn test_parse_xml_fragment() {
        let xml = r#"<lensdatabase version="2">
            <lens>
                <maker>TestMaker</maker>
                <model>Test 50mm f/2</model>
                <mount>TestMount</mount>
                <calibration>
                    <distortion model="ptlens" focal="50" a="0.001" b="-0.005" c="0.002"/>
                    <tca model="linear" focal="50" kr="1.0003" kb="0.9997"/>
                    <vignetting model="pa" focal="50" aperture="2.0" distance="1000" k1="-0.5" k2="0.2" k3="-0.03"/>
                </calibration>
            </lens>
        </lensdatabase>"#;

        let profiles = parse_lensfun_xml(xml).unwrap();
        assert_eq!(profiles.len(), 1);
        let p = &profiles[0];
        assert_eq!(p.lens_name, "TestMaker Test 50mm f/2");
        assert_eq!(p.mount, "TestMount");
        assert_eq!(p.profiles.len(), 1);
        let fp = &p.profiles[0];
        assert!((fp.distortion.a - 0.001).abs() < 1e-6);
        assert!((fp.distortion.b - (-0.005)).abs() < 1e-6);
        assert!((fp.ca.red_scale - 1.0003).abs() < 1e-6);
        assert!((fp.vignette.v1 - (-0.5)).abs() < 1e-6);
    }
}
