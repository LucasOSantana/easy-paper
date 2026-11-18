use crate::models::{StockPage, StockPhoto, StockProvider, StockSearch};
use crate::services::config_service::get_images_directory;
use crate::services::credentials_service::{delete_api_key, get_api_key, set_api_key};
use crate::services::stock_service;
use crate::services::thumbnail_service::{ensure_thumbnail, thumbnail_cache_dir};

fn require_key(provider: StockProvider) -> Result<String, String> {
    get_api_key(provider).ok_or_else(|| {
        let name = provider.name();
        crate::tr!(
            en: format!("Add your {} API key in the {} tab first.", name, name),
            pt: format!("Adicione primeiro sua chave de API do {} na aba {}.", name, name),
        )
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
        return Err(crate::tr!(en: "Paste your API key first.", pt: "Cole sua chave de API primeiro."));
    }

    tauri::async_runtime::spawn_blocking(move || {
        stock_service::verify_key(provider, &key)?;
        set_api_key(provider, &key)
    })
    .await
    .map_err(|e| crate::tr!(
        en: format!("Key check was interrupted: {}", e),
        pt: format!("A verificação da chave foi interrompida: {}", e),
    ))?
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
        .map_err(|e| crate::tr!(
            en: format!("Search was interrupted: {}", e),
            pt: format!("A busca foi interrompida: {}", e),
        ))?
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
    .map_err(|e| crate::tr!(
        en: format!("Wallpaper download was interrupted: {}", e),
        pt: format!("O download do papel de parede foi interrompido: {}", e),
    ))??;

    tauri::async_runtime::spawn_blocking(move || ensure_thumbnail(&path, &thumbnail_cache_dir(&folder)));
    Ok(())
}
