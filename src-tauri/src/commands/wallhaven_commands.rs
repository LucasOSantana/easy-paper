use crate::models::{WallhavenPage, WallhavenSearch};
use crate::services::config_service::get_images_directory;
use crate::services::thumbnail_service::{ensure_thumbnail, thumbnail_cache_dir};
use crate::services::wallhaven_service;

#[tauri::command]
pub async fn search_wallhaven(
    app: tauri::AppHandle,
    search: WallhavenSearch,
) -> Result<WallhavenPage, String> {
    let min_resolution = if search.match_screen {
        app.primary_monitor()
            .ok()
            .flatten()
            .map(|monitor| (monitor.size().width, monitor.size().height))
    } else {
        None
    };

    tauri::async_runtime::spawn_blocking(move || wallhaven_service::search(&search, min_resolution))
        .await
        .map_err(|e| format!("Wallhaven search was interrupted: {}", e))?
}

#[tauri::command]
pub async fn download_wallhaven(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let folder = get_images_directory(&app);
    let download_folder = folder.clone();

    let path = tauri::async_runtime::spawn_blocking(move || wallhaven_service::download(&url, &download_folder))
        .await
        .map_err(|e| format!("Wallpaper download was interrupted: {}", e))??;

    tauri::async_runtime::spawn_blocking(move || ensure_thumbnail(&path, &thumbnail_cache_dir(&folder)));
    Ok(())
}
