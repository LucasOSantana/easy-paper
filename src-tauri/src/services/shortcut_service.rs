use std::str::FromStr;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use crate::state::ConfigState;
use crate::services::config_service::load_config;

pub fn reload_shortcut_from_config<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let cfg = load_config(app);
    let trimmed = cfg.shortcut.trim().to_lowercase();
    if let Ok(parsed) = Shortcut::from_str(&trimmed) {
        let _ = app.global_shortcut().unregister_all();
        if let Ok(()) = app.global_shortcut().register(parsed) {
            if let Some(state) = app.try_state::<ConfigState>() {
                let mut st = state.0.lock().unwrap();
                st.shortcut = trimmed;
                st.images_path = cfg.images_path;
            }
        }
    }
}
