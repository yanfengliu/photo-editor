#![allow(dead_code)]

mod state;
mod commands;
mod catalog;
mod imaging;
mod gpu;
mod ai;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::catalog::import_folder,
            commands::catalog::import_paths,
            commands::catalog::get_images,
            commands::catalog::search_images,
            commands::catalog::set_rating,
            commands::catalog::set_color_label,
            commands::catalog::set_flag,
            commands::catalog::add_tags,
            commands::catalog::remove_tag,
            commands::catalog::create_collection,
            commands::catalog::add_to_collection,
            commands::catalog::get_collections,
            commands::catalog::delete_images,
            commands::image::load_thumbnail,
            commands::image::load_preview,
            commands::image::load_full_resolution,
            commands::image::get_exif_data,
            commands::develop::apply_edits,
            commands::develop::get_edit_params,
            commands::develop::reset_edits,
            commands::develop::save_snapshot,
            commands::develop::load_snapshot,
            commands::develop::get_history,
            commands::develop::copy_edits,
            commands::develop::paste_edits,
            commands::export::export_image,
            commands::export::batch_export,
            commands::export::export_xmp_sidecar,
            commands::system::browse_folder,
            commands::system::get_gpu_info,
            commands::system::get_app_config,
            commands::system::set_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
