use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use crate::models::AppConfig;
use crate::state::ConfigState;

pub fn get_default_images_directory<R: tauri::Runtime>(app: &impl Manager<R>) -> PathBuf {
    let directory = app
        .path()
        .picture_dir()
        .unwrap_or_else(|_| {
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                PathBuf::from(user_profile).join("Pictures")
            } else {
                PathBuf::from(".")
            }
        })
        .join("Easy paper");

    let _ = fs::create_dir_all(&directory);
    directory
}

pub fn get_images_directory<R: tauri::Runtime>(app: &impl Manager<R>) -> PathBuf {
    let configured = app
        .try_state::<ConfigState>()
        .map(|state| state.0.lock().unwrap().images_path.trim().to_string())
        .unwrap_or_default();

    if configured.is_empty() {
        return get_default_images_directory(app);
    }

    let path = PathBuf::from(configured);
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn get_config_path<R: tauri::Runtime>(app: &impl Manager<R>) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to obtain application configuration directory: {}", e))?;

    if !directory.exists() {
        fs::create_dir_all(&directory)
            .map_err(|e| format!("Failed to create configuration directory: {}", e))?;
    }

    Ok(directory.join("config.json"))
}

pub fn load_config<R: tauri::Runtime>(app: &impl Manager<R>) -> AppConfig {
    let default_images_dir = get_default_images_directory(app);
    let default_images_str = default_images_dir.to_string_lossy().to_string();

    if let Ok(path) = get_config_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    if config.images_path.trim().is_empty() {
                        config.images_path = default_images_str;
                        let _ = save_config(app, &config);
                    }
                    return config;
                }
            }
        } else {
            let mut default_config = AppConfig::default();
            default_config.images_path = default_images_str;
            let _ = save_config(app, &default_config);
            return default_config;
        }
    }

    let mut fallback = AppConfig::default();
    fallback.images_path = default_images_str;
    fallback
}

pub fn save_config<R: tauri::Runtime>(
    app: &impl Manager<R>,
    config: &AppConfig,
) -> Result<(), String> {
    let path = get_config_path(app)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize application configuration: {}", e))?;

    fs::write(path, json)
        .map_err(|e| format!("Failed to write configuration file to disk: {}", e))?;

    Ok(())
}
