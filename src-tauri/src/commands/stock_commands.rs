use crate::models::{StockPage, StockPhoto, StockProvider, StockSearch};
use crate::services::config_service::get_images_directory;
use crate::services::credentials_service::{delete_api_key, get_api_key, set_api_key};
use crate::services::stock_service;
use crate::services::thumbnail_service::{ensure_thumbnail, thumbnail_cache_dir};

fn require_key(provider: StockProvider) -> Result<String, String> {
    get_api_key(provider).ok_or_else(|| {
        format!("Add your {} API key in the {} tab first.", provider.name(), provider.name())
    })
}

/// Whether a key is stored. The key itself is never sent to the frontend.
#[tauri::command]
pub fn has_stock_api_key(provider: StockProvider) -> bool {
    get_api_key(provider).is_some()
}

/// Checks the key against the provider and stores it only if it works.
#[tauri::command]
pub async fn save_stock_api_key(provider: StockProvider, key: String) -> Result<(), String> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("Paste your API key first.".to_string());
    }

    tauri::async_runtime::spawn_blocking(move || {
        stock_service::verify_key(provider, &key)?;
        set_api_key(provider, &key)
    })
    .await
    .map_err(|e| format!("Key check was interrupted: {}", e))?
}

#[tauri::command]
pub fn remove_stock_api_key(provider: StockProvider) -> Result<(), String> {
    delete_api_key(provider)
}

#[tauri::command]
pub async fn search_stock(search: StockSearch) -> Result<StockPage, String> {
    let key = require_key(search.provider)?;

    tauri::async_runtime::spawn_blocking(move || stock_service::search(&search, &key))
        .await
        .map_err(|e| format!("Search was interrupted: {}", e))?
}

#[tauri::command]
pub async fn download_stock(
    app: tauri::AppHandle,
    provider: StockProvider,
    photo: StockPhoto,
) -> Result<(), String> {
    let key = require_key(provider)?;
    let folder = get_images_directory(&app);
    let download_folder = folder.clone();

    let path = tauri::async_runtime::spawn_blocking(move || {
        stock_service::download(provider, &photo, &key, &download_folder)
    })
    .await
    .map_err(|e| format!("Wallpaper download was interrupted: {}", e))??;

    tauri::async_runtime::spawn_blocking(move || ensure_thumbnail(&path, &thumbnail_cache_dir(&folder)));
    Ok(())
}
