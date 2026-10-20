use std::path::PathBuf;
use tauri::Manager;
use crate::commands::open_settings_window;
use crate::services::config_service::get_default_images_directory;
use crate::services::shortcut_service::reload_shortcut_from_config;
use crate::services::window_service::toggle_main_window;
use crate::state::ConfigState;

fn open_configured_images_folder<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let folder = {
        if let Some(state) = app.try_state::<ConfigState>() {
            let config_lock = state.0.lock().unwrap();
            if config_lock.images_path.trim().is_empty() {
                get_default_images_directory(app)
            } else {
                let path = PathBuf::from(config_lock.images_path.trim());
                if !path.exists() {
                    let _ = std::fs::create_dir_all(&path);
                }
                path
            }
        } else {
            get_default_images_directory(app)
        }
    };

    let _ = std::process::Command::new("explorer.exe")
        .arg(&folder)
        .spawn();
}

pub fn setup_system_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let add_pictures_item = tauri::menu::MenuItem::with_id(app, "add_pictures", "Add pictures", true, None::<&str>)?;
    let settings_item = tauri::menu::MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let reload_item = tauri::menu::MenuItem::with_id(app, "reload", "Reload Shortcut", true, None::<&str>)?;
    let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = tauri::menu::MenuBuilder::new(app)
        .item(&add_pictures_item)
        .item(&settings_item)
        .item(&reload_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let mut tray_builder = tauri::tray::TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Easy Paper")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "add_pictures" => {
                    open_configured_images_folder(app);
                }
                "settings" => {
                    open_settings_window(app.clone());
                }
                "reload" => {
                    reload_shortcut_from_config(app);
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    } else {
        let icon_bytes = include_bytes!("../../icons/32x32.png");
        if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
            tray_builder = tray_builder.icon(icon);
        }
    }

    let _tray = tray_builder.build(app)?;
    Ok(())
}
