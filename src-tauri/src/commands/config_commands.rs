use std::str::FromStr;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use crate::models::AppConfig;
use crate::state::ConfigState;
use crate::services::config_service::modify_config;

#[tauri::command]
pub fn get_config(config_state: tauri::State<ConfigState>) -> AppConfig {
    config_state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(
    app: tauri::AppHandle,
    config_state: tauri::State<ConfigState>,
    shortcut: String,
    images_path: String,
) -> Result<AppConfig, String> {
    let clean_shortcut = shortcut.trim().to_lowercase();
    let parsed_shortcut = Shortcut::from_str(&clean_shortcut)
        .map_err(|e| crate::tr!(
            en: format!("Invalid shortcut: {}", e),
            pt: format!("Atalho inválido: {}", e),
        ))?;

    let _ = app.global_shortcut().unregister_all();
    if let Err(e) = app.global_shortcut().register(parsed_shortcut) {
        let old_shortcut = config_state.0.lock().unwrap().shortcut.clone();
        if let Ok(old_parsed) = Shortcut::from_str(&old_shortcut) {
            let _ = app.global_shortcut().register(old_parsed);
        }
        return Err(crate::tr!(
            en: format!("Could not register the shortcut: {}. It may already be in use by another app.", e),
            pt: format!("Não foi possível registrar o atalho: {}. Talvez outro app já esteja usando ele.", e),
        ));
    }

    let images_path = images_path.trim().to_string();
    modify_config(&app, |config| {
        config.shortcut = clean_shortcut;
        config.images_path = images_path;
        Ok(())
    })
}
