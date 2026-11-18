use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use crate::models::{AppConfig, SyncedSettings};
use crate::state::ConfigState;

/// Event sent to every window whenever the configuration changes, so an open
/// Settings window stays current with changes made from the tray or by sync.
pub const CONFIG_CHANGED_EVENT: &str = "config-changed";

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
        .map_err(|e| crate::tr!(
            en: format!("Could not find the settings folder: {}", e),
            pt: format!("Não foi possível encontrar a pasta de configurações: {}", e),
        ))?;

    if !directory.exists() {
        fs::create_dir_all(&directory)
            .map_err(|e| crate::tr!(
                en: format!("Could not create the settings folder: {}", e),
                pt: format!("Não foi possível criar a pasta de configurações: {}", e),
            ))?;
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
        .map_err(|e| crate::tr!(
            en: format!("Could not prepare the settings: {}", e),
            pt: format!("Não foi possível preparar as configurações: {}", e),
        ))?;

    fs::write(path, json)
        .map_err(|e| crate::tr!(
            en: format!("Could not write the settings file: {}", e),
            pt: format!("Não foi possível gravar o arquivo de configurações: {}", e),
        ))?;

    Ok(())
}

/// Current Unix time in milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// How a configuration change marks the synced settings.
pub enum SyncStamp {
    /// A local change: stamp "now" when synced settings changed and push
    /// them to the sync folder.
    Local,
    /// Use exactly this timestamp and do not push (settings received from
    /// the sync folder keep the timestamp they were written with).
    Exact(u64),
}

/// Applies `change` to the configuration, saves it and notifies the rest of
/// the app. The lock is held while changing and saving, so concurrent
/// changes never overwrite each other.
pub fn modify_config<R: Runtime>(
    app: &AppHandle<R>,
    change: impl FnOnce(&mut AppConfig) -> Result<(), String>,
) -> Result<AppConfig, String> {
    modify_config_with_stamp(app, SyncStamp::Local, change)
}

pub fn modify_config_with_stamp<R: Runtime>(
    app: &AppHandle<R>,
    stamp: SyncStamp,
    change: impl FnOnce(&mut AppConfig) -> Result<(), String>,
) -> Result<AppConfig, String> {
    let state = app.state::<ConfigState>();
    let (updated, push, language_changed) = {
        let mut current = state.0.lock().unwrap();
        let mut next = current.clone();
        change(&mut next)?;
        let language_changed = current.language != next.language;

        let synced_changed = SyncedSettings::from(&*current) != SyncedSettings::from(&next);
        let push = match stamp {
            SyncStamp::Local if synced_changed => {
                next.settings_updated_at = now_ms();
                true
            }
            SyncStamp::Local => false,
            SyncStamp::Exact(updated_at) => {
                next.settings_updated_at = updated_at;
                false
            }
        };

        save_config(app, &next)
            .map_err(|e| crate::tr!(
                en: format!("Failed to save the settings: {}", e),
                pt: format!("Não foi possível salvar as configurações: {}", e),
            ))?;
        *current = next.clone();
        (next, push, language_changed)
    };

    // Outside the lock: these may wait on the main thread.
    if language_changed {
        crate::i18n::apply_config(&updated);
        crate::services::window_service::retitle_settings_window(app);
    }
    let _ = app.emit(CONFIG_CHANGED_EVENT, &updated);
    crate::tray::refresh_tray_menu(app, &updated);
    if push && !updated.sync.folder.trim().is_empty() {
        crate::services::sync_service::push_in_background(app.clone());
    }

    Ok(updated)
}
