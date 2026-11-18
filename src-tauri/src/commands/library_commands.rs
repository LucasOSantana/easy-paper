use crate::services::library_service;

#[tauri::command]
pub fn set_favorite(app: tauri::AppHandle, path: String, favorite: bool) -> Result<bool, String> {
    library_service::set_favorite(&app, &path, favorite)
}

#[tauri::command]
pub async fn delete_wallpaper(app: tauri::AppHandle, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || library_service::delete_wallpaper(&app, &path))
        .await
        .map_err(|e| crate::tr!(
            en: format!("Delete was interrupted: {}", e),
            pt: format!("A exclusão foi interrompida: {}", e),
        ))?
}

#[tauri::command]
pub fn reveal_wallpaper(app: tauri::AppHandle, path: String) -> Result<(), String> {
    library_service::reveal_in_explorer(&app, &path)
}
