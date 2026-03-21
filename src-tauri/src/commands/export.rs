use tauri::State;
use crate::state::AppState;
use crate::catalog::models::EditParams;

#[derive(serde::Deserialize)]
pub struct ExportSettings {
    pub format: String,
    pub quality: u8,
    pub output_path: String,
    pub max_dimension: Option<u32>,
}

fn process_image(state: &AppState, image_id: &str) -> Result<(Vec<u8>, u32, u32), String> {
    let file_path = state.get_image_file_path(image_id)?;
    let edit_params = state.get_image_edit_params(image_id)?;

    let image = crate::imaging::loader::load_full_rgba(&file_path)
        .map_err(|e| e.to_string())?;
    let mut gpu = state.gpu.lock().map_err(|e| e.to_string())?;
    let processed = crate::gpu::pipeline::apply_edits_with_backend(
        gpu.as_mut(),
        &image.data,
        image.width,
        image.height,
        &edit_params,
    );
    let (result, out_w, out_h) = crate::gpu::pipeline::apply_crop_rotation(
        processed,
        image.width,
        image.height,
        &edit_params,
    );
    Ok((result, out_w, out_h))
}

#[tauri::command]
pub async fn export_image(
    state: State<'_, AppState>,
    image_id: String,
    settings: ExportSettings,
) -> Result<String, String> {
    let (processed, width, height) = process_image(&state, &image_id)?;

    crate::imaging::export::export_pixels(
        &processed,
        width,
        height,
        &settings.output_path,
        &settings.format,
        settings.quality,
        settings.max_dimension,
    ).map_err(|e| e.to_string())?;

    Ok(settings.output_path)
}

#[tauri::command]
pub async fn batch_export(
    state: State<'_, AppState>,
    image_ids: Vec<String>,
    settings: ExportSettings,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for image_id in &image_ids {
        let file_path = state.get_image_file_path(image_id)?;
        let file_name = std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        let ext = match settings.format.as_str() {
            "png" => "png",
            "tiff" => "tiff",
            _ => "jpg",
        };
        let output_path = format!("{}/{}.{}", settings.output_path, file_name, ext);

        let (processed, width, height) = process_image(&state, image_id)?;

        crate::imaging::export::export_pixels(
            &processed,
            width,
            height,
            &output_path,
            &settings.format,
            settings.quality,
            settings.max_dimension,
        ).map_err(|e| e.to_string())?;

        results.push(output_path);
    }
    Ok(results)
}

#[tauri::command]
pub async fn export_xmp_sidecar(
    state: State<'_, AppState>,
    image_id: String,
    output_path: Option<String>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let record = crate::catalog::queries::get_image_by_id(&db, &image_id)
        .map_err(|e| e.to_string())?;

    let edit_params: EditParams = match record.edit_params {
        Some(ref json) => serde_json::from_str(json).map_err(|e| e.to_string())?,
        None => EditParams::default(),
    };

    let xmp_path = match output_path {
        Some(p) => p,
        None => {
            let src = std::path::Path::new(&record.file_path);
            src.with_extension("xmp").to_string_lossy().to_string()
        }
    };

    let xmp = generate_xmp(&record, &edit_params);
    std::fs::write(&xmp_path, xmp).map_err(|e| e.to_string())?;
    Ok(xmp_path)
}

fn generate_xmp(record: &crate::catalog::models::ImageRecord, params: &EditParams) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:crs="http://ns.adobe.com/camera-raw-settings/1.0/"
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    crs:Version="15.0"
    crs:Exposure2012="{exposure:.2}"
    crs:Contrast2012="{contrast}"
    crs:Highlights2012="{highlights}"
    crs:Shadows2012="{shadows}"
    crs:Whites2012="{whites}"
    crs:Blacks2012="{blacks}"
    crs:Temperature="{temperature}"
    crs:Tint="{tint}"
    crs:Saturation="{saturation}"
    crs:Vibrance="{vibrance}"
    crs:Clarity2012="{clarity}"
    crs:Dehaze="{dehaze}"
    crs:Sharpness="{sharpening_amount}"
    crs:SharpenRadius="{sharpening_radius:.1}"
    crs:LuminanceSmoothing="{denoise_luminance}"
    crs:ColorNoiseReduction="{denoise_color}"
    crs:PostCropVignetteAmount="{vignette_amount}"
    crs:GrainAmount="{grain_amount}"
    crs:GrainSize="{grain_size}"
    xmp:Rating="{rating}">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#,
        exposure = params.exposure,
        contrast = params.contrast as i32,
        highlights = params.highlights as i32,
        shadows = params.shadows as i32,
        whites = params.whites as i32,
        blacks = params.blacks as i32,
        temperature = params.temperature as i32,
        tint = params.tint as i32,
        saturation = params.saturation as i32,
        vibrance = params.vibrance as i32,
        clarity = params.clarity as i32,
        dehaze = params.dehaze as i32,
        sharpening_amount = params.sharpening_amount as i32,
        sharpening_radius = params.sharpening_radius,
        denoise_luminance = params.denoise_luminance as i32,
        denoise_color = params.denoise_color as i32,
        vignette_amount = params.vignette_amount as i32,
        grain_amount = params.grain_amount as i32,
        grain_size = params.grain_size as i32,
        rating = record.rating,
    )
}
