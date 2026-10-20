use std::path::PathBuf;
use crate::models::WallpaperImageItem;
use crate::state::ConfigState;
use crate::services::config_service::get_default_images_directory;
use crate::services::thumbnail_service::get_wallpaper_images_from_folder;
use crate::services::wallpaper_service::set_desktop_wallpaper;

#[tauri::command]
pub fn get_images(
    app: tauri::AppHandle,
    config_state: tauri::State<ConfigState>,
) -> Vec<WallpaperImageItem> {
    let folder_path = {
        let state = config_state.0.lock().unwrap();
        if state.images_path.trim().is_empty() {
            get_default_images_directory(&app)
        } else {
            let path = PathBuf::from(state.images_path.trim());
            if !path.exists() {
                let _ = std::fs::create_dir_all(&path);
            }
            path
        }
    };

    get_wallpaper_images_from_folder(&folder_path)
}

#[tauri::command]
pub fn set_wallpaper(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
    let target_point = if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
        Some((
            pos.x + (size.width as i32) / 2,
            pos.y + (size.height as i32) / 2,
        ))
    } else {
        None
    };

    set_desktop_wallpaper(&path, target_point).map_err(|e| e.to_string())
}
