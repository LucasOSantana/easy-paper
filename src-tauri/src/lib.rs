pub mod models;
pub mod state;
pub mod services;
pub mod commands;
pub mod tray;

use std::str::FromStr;
use std::sync::Mutex;
use std::time::Instant;

use tauri::webview::PageLoadEvent;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use state::{ConfigState, WindowDisplayState};
use services::config_service::load_config;
use services::window_service::{hide_main_window, resize_and_center_main_window, toggle_main_window};
use services::autostart_service::enable_windows_autostart;
use services::taskbar_service::{
    apply_saved_taskbar_transparency, set_taskbar_transparency, watch_explorer_restarts,
};
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

            watch_explorer_restarts(app.handle());
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn_blocking(move || {
                let _ = apply_saved_taskbar_transparency(&app_handle);
            });

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
        .on_page_load(|webview, payload| {
            let hidden = !webview.window().is_visible().unwrap_or(true);
            if webview.label() == "main" && matches!(payload.event(), PageLoadEvent::Finished) && hidden {
                hide_main_window(webview.app_handle());
            }
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::Focused(false) = event {
                    let state = window.state::<WindowDisplayState>();
                    let elapsed = state.0.lock().unwrap().elapsed().as_millis();
                    if elapsed < 300 {
                        return;
                    }

                    hide_main_window(window.app_handle());
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
            open_folder_path,
            set_taskbar_transparent,
            search_wallhaven,
            download_wallhaven
        ])
        .build(tauri::generate_context!())
        .expect("Fatal error while executing Tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                let _ = set_taskbar_transparency(app, false);
            }
        });
}
