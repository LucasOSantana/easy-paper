pub mod models;
pub mod state;
pub mod services;
pub mod commands;
pub mod tray;

use std::str::FromStr;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use state::{ConfigState, WindowDisplayState};
use services::config_service::load_config;
use services::window_service::{resize_and_center_main_window, toggle_main_window};
use services::autostart_service::enable_windows_autostart;
use tray::setup_system_tray;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(WindowDisplayState(Mutex::new(Instant::now())))
        .setup(|app| {
            let app_config = load_config(app);
            let initial_shortcut = app_config.shortcut.clone();
            app.manage(ConfigState(Mutex::new(app_config)));

            if let Some(window) = app.get_webview_window("main") {
                resize_and_center_main_window(&window);
            }

            enable_windows_autostart();

            #[cfg(desktop)]
            {
                setup_system_tray(app.handle())?;

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                toggle_main_window(app);
                            }
                        })
                        .build(),
                )?;

                if let Ok(parsed) = Shortcut::from_str(&initial_shortcut) {
                    let _ = app.global_shortcut().register(parsed);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::Focused(false) = event {
                    let state = window.state::<WindowDisplayState>();
                    let elapsed = state.0.lock().unwrap().elapsed().as_millis();
                    if elapsed < 300 {
                        return;
                    }

                    let _ = window.hide();
                }
            } else if window.label() == "settings" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_images,
            set_wallpaper,
            hide_window,
            get_config,
            update_config,
            pick_folder,
            open_settings_window,
            close_settings_window,
            open_folder_path
        ])
        .run(tauri::generate_context!())
        .expect("Fatal error while executing Tauri application");
}
