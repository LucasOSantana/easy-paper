use std::str::FromStr;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use crate::models::AppConfig;
use crate::state::ConfigState;
use crate::services::config_service::save_config;

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
        .map_err(|e| format!("Invalid shortcut syntax: {}", e))?;

    let _ = app.global_shortcut().unregister_all();
    if let Err(e) = app.global_shortcut().register(parsed_shortcut) {
        let old_shortcut = config_state.0.lock().unwrap().shortcut.clone();
        if let Ok(old_parsed) = Shortcut::from_str(&old_shortcut) {
            let _ = app.global_shortcut().register(old_parsed);
        }
        return Err(format!("Failed to register shortcut: {}. It might be already in use by another application.", e));
    }

    let updated_config = AppConfig {
        shortcut: clean_shortcut,
        images_path: images_path.trim().to_string(),
        ..config_state.0.lock().unwrap().clone()
    };

    save_config(&app, &updated_config)
        .map_err(|e| format!("Failed to save configuration to disk: {}", e))?;

    {
        let mut state = config_state.0.lock().unwrap();
        *state = updated_config.clone();
    }

    Ok(updated_config)
}
