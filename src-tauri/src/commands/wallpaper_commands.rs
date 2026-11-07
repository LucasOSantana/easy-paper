use tauri::Manager;
use crate::models::WallpaperImageItem;
use crate::services::config_service::get_images_directory;
use crate::services::thumbnail_service::{get_wallpaper_images_from_folder, thumbnail_cache_dir};
use crate::services::rotation_service::reset_rotation_timer;
use crate::services::wallpaper_service::set_desktop_wallpaper;

#[tauri::command]
pub fn get_images(app: tauri::AppHandle) -> Vec<WallpaperImageItem> {
    let folder_path = get_images_directory(&app);

    let _ = app
        .asset_protocol_scope()
        .allow_directory(thumbnail_cache_dir(&folder_path), false);

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

    set_desktop_wallpaper(&path, target_point).map_err(|e| e.to_string())?;

    // A manual pick restarts the automatic rotation countdown.
    reset_rotation_timer(window.app_handle());
    Ok(())
}
