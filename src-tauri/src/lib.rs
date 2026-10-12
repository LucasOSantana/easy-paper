mod config;
mod images;
mod wallpaper;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Instant;

use config::{load_config, save_config, AppConfig};
use images::{get_images_from_folder, ImageItem};
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use wallpaper::set_desktop_wallpaper;

pub struct ConfigState(pub Mutex<AppConfig>);
struct ShownAt(pub Mutex<Instant>);

#[tauri::command]
fn get_images(app: tauri::AppHandle, config_state: tauri::State<ConfigState>) -> Vec<ImageItem> {
    let folder_path = {
        let state = config_state.0.lock().unwrap();
        if state.images_path.trim().is_empty() {
            config::get_default_images_dir(&app)
        } else {
            let p = PathBuf::from(state.images_path.trim());
            if !p.exists() {
                let _ = std::fs::create_dir_all(&p);
            }
            p
        }
    };

    get_images_from_folder(&folder_path)
}

#[tauri::command]
fn set_wallpaper(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
    let target_point = if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
        Some((
            pos.x + (size.width as i32) / 2,
            pos.y + (size.height as i32) / 2,
        ))
    } else {
        None
    };

    set_desktop_wallpaper(&path, target_point).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn open_images_folder<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let folder = {
        if let Some(state) = app.try_state::<ConfigState>() {
            let st = state.0.lock().unwrap();
            if st.images_path.trim().is_empty() {
                config::get_default_images_dir(app)
            } else {
                let p = PathBuf::from(st.images_path.trim());
                if !p.exists() {
                    let _ = std::fs::create_dir_all(&p);
                }
                p
            }
        } else {
            config::get_default_images_dir(app)
        }
    };

    let _ = std::process::Command::new("explorer.exe")
        .arg(&folder)
        .spawn();
}

fn open_config_file<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Ok(config_path) = config::get_config_path(app) {
        if !config_path.exists() {
            let default_config = AppConfig::default();
            let _ = save_config(app, &default_config);
        }

        let path_str = config_path.to_string_lossy().to_string();

        #[cfg(target_os = "windows")]
        {
            let res = std::process::Command::new("cmd")
                .args(["/c", "start", "", &path_str])
                .spawn();

            if res.is_err() {
                let _ = std::process::Command::new("notepad.exe")
                    .arg(&config_path)
                    .spawn();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = app.opener().open_path(&path_str, None::<&str>);
        }
    }
}

fn reload_shortcut_from_config<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
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

#[cfg(target_os = "windows")]
fn get_cursor_position() -> Option<(i32, i32)> {
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    unsafe extern "system" {
        fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    }

    let mut pt = POINT { x: 0, y: 0 };
    let res = unsafe { GetCursorPos(&mut pt) };
    if res != 0 {
        Some((pt.x, pt.y))
    } else {
        None
    }
}

fn resize_and_center_window(window: &tauri::WebviewWindow) {
    let monitors = window.available_monitors().unwrap_or_default();

    #[cfg(target_os = "windows")]
    let cursor_pos = get_cursor_position();
    #[cfg(not(target_os = "windows"))]
    let cursor_pos: Option<(i32, i32)> = None;

    let target_monitor = if let Some((cx, cy)) = cursor_pos {
        monitors.into_iter().find(|m| {
            let pos = m.position();
            let size = m.size();
            cx >= pos.x
                && cx < pos.x + (size.width as i32)
                && cy >= pos.y
                && cy < pos.y + (size.height as i32)
        })
    } else {
        None
    };

    let monitor = target_monitor
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());

    if let Some(mon) = monitor {
        let size = mon.size();
        let mon_pos = mon.position();
        let mon_w = size.width as f64;
        let mon_h = size.height as f64;

        let win_w = mon_w;
        let win_h = mon_h * 0.70;
        let pos_x = mon_pos.x as f64;
        let pos_y = (mon_pos.y as f64) + (mon_h - win_h) / 2.0;

        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: pos_x as i32,
            y: pos_y as i32,
        }));
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: win_w as u32,
            height: win_h as u32,
        }));
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: pos_x as i32,
            y: pos_y as i32,
        }));
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    reload_shortcut_from_config(app);

    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
        } else {
            resize_and_center_window(&window);
            if let Some(state) = app.try_state::<ShownAt>() {
                *state.0.lock().unwrap() = Instant::now();
            }
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}

#[cfg(target_os = "windows")]
fn enable_autostart() {
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_str = exe_path.to_string_lossy().to_string();
        let _ = std::process::Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "easy-paper",
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\"", exe_str),
                "/f",
            ])
            .spawn();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ShownAt(Mutex::new(Instant::now())))
        .setup(|app| {
            let app_config = load_config(app);
            let initial_shortcut = app_config.shortcut.clone();
            app.manage(ConfigState(Mutex::new(app_config)));

            if let Some(window) = app.get_webview_window("main") {
                resize_and_center_window(&window);
            }

            #[cfg(target_os = "windows")]
            enable_autostart();

            #[cfg(desktop)]
            {
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
                                open_images_folder(app);
                            }
                            "settings" => {
                                open_config_file(app);
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
                    let icon_bytes = include_bytes!("../icons/32x32.png");
                    if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
                        tray_builder = tray_builder.icon(icon);
                    }
                }

                let _tray = tray_builder.build(app)?;

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
                    let state = window.state::<ShownAt>();
                    let t = state.0.lock().unwrap();
                    if t.elapsed().as_millis() < 300 {
                        return;
                    }

                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_images, set_wallpaper, hide_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
