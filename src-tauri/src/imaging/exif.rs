use std::path::Path;
use std::fs::File;
use std::io::BufReader;

pub struct ExifBasic {
    pub date_taken: Option<String>,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<f64>,
    pub aperture: Option<f64>,
    pub shutter_speed: Option<String>,
}

impl ExifBasic {
    fn empty() -> Self {
        Self { date_taken: None, camera: None, lens: None, iso: None, focal_length: None, aperture: None, shutter_speed: None }
    }
}

pub fn read_exif_basic(path: &Path) -> ExifBasic {
    let file = match File::open(path) { Ok(f) => f, Err(_) => return ExifBasic::empty() };
    let mut reader = BufReader::new(file);
    let exif_reader = exif::Reader::new();
    let data = match exif_reader.read_from_container(&mut reader) { Ok(e) => e, Err(_) => return ExifBasic::empty() };

    ExifBasic {
        date_taken: data.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY).map(|f| f.display_value().to_string()),
        camera: data.get_field(exif::Tag::Model, exif::In::PRIMARY).map(|f| f.display_value().to_string().trim_matches('"').to_string()),
        lens: data.get_field(exif::Tag::LensModel, exif::In::PRIMARY).map(|f| f.display_value().to_string().trim_matches('"').to_string()),
        iso: data.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY).and_then(|f| f.value.get_uint(0)),
        focal_length: data.get_field(exif::Tag::FocalLength, exif::In::PRIMARY).and_then(|f| match &f.value { exif::Value::Rational(v) if !v.is_empty() => Some(v[0].to_f64()), _ => None }),
        aperture: data.get_field(exif::Tag::FNumber, exif::In::PRIMARY).and_then(|f| match &f.value { exif::Value::Rational(v) if !v.is_empty() => Some(v[0].to_f64()), _ => None }),
        shutter_speed: data.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY).map(|f| f.display_value().to_string()),
    }
}

pub fn read_exif(file_path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let exif_reader = exif::Reader::new();
    let data = exif_reader.read_from_container(&mut reader)?;
    let mut map = serde_json::Map::new();
    for field in data.fields() {
        map.insert(format!("{}", field.tag), serde_json::Value::String(field.display_value().to_string()));
    }
    Ok(serde_json::Value::Object(map))
}
