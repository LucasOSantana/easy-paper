use tauri::menu::{CheckMenuItem, MenuItem};
use tauri::{Manager, Runtime, Wry};

use crate::models::AppConfig;
use crate::services::config_service::{get_images_directory, modify_config};
use crate::services::rotation_service::{reset_rotation_timer, rotate_next};
use crate::services::shortcut_service::reload_shortcut_from_config;
use crate::services::window_service::{show_settings_window, toggle_main_window};
use crate::state::ConfigState;

/// Menu items whose text or check mark follows the configuration.
pub struct TrayMenuState {
    show: MenuItem<Wry>,
    next: MenuItem<Wry>,
    auto_rotate: CheckMenuItem<Wry>,
    favorites_only: CheckMenuItem<Wry>,
    open_folder: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    reload: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

/// Menu texts in the current language.
struct Labels {
    show: String,
    next: String,
    auto_rotate: String,
    favorites_only: String,
    open_folder: String,
    settings: String,
    reload: String,
    quit: String,
}

fn labels() -> Labels {
    Labels {
        show: crate::tr!(en: "Choose wallpaper", pt: "Escolher papel de parede"),
        next: crate::tr!(en: "Next wallpaper", pt: "Próximo papel de parede"),
        auto_rotate: crate::tr!(en: "Automatic rotation", pt: "Rotação automática"),
        favorites_only: crate::tr!(en: "Rotate favorites only", pt: "Rotacionar só favoritos"),
        open_folder: crate::tr!(en: "Open wallpaper folder", pt: "Abrir pasta de papéis de parede"),
        settings: crate::tr!(en: "Settings", pt: "Configurações"),
        reload: crate::tr!(en: "Reload Shortcut", pt: "Recarregar atalho"),
        quit: crate::tr!(en: "Quit", pt: "Sair"),
    }
}

fn open_configured_images_folder<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let folder = get_images_directory(app);

    let _ = std::process::Command::new("explorer.exe")
        .arg(&folder)
        .spawn();
}

/// Updates the check marks and texts (language) after any configuration change.
pub fn refresh_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>, config: &AppConfig) {
    if let Some(menu) = app.try_state::<TrayMenuState>() {
        let _ = menu.auto_rotate.set_checked(config.auto_rotate.enabled);
        let _ = menu.favorites_only.set_checked(config.auto_rotate.favorites_only);

        let text = labels();
        let _ = menu.show.set_text(&text.show);
        let _ = menu.next.set_text(&text.next);
        let _ = menu.auto_rotate.set_text(&text.auto_rotate);
        let _ = menu.favorites_only.set_text(&text.favorites_only);
        let _ = menu.open_folder.set_text(&text.open_folder);
        let _ = menu.settings.set_text(&text.settings);
        let _ = menu.reload.set_text(&text.reload);
        let _ = menu.quit.set_text(&text.quit);
    }
}

fn next_wallpaper(app: &tauri::AppHandle) {
    let app = app.clone();
    // Applying a wallpaper can take a moment; keep the tray responsive.
    std::thread::spawn(move || {
        if let Err(err) = rotate_next(&app) {
            eprintln!("[tray] Next wallpaper failed: {}", err);
        }
    });
}

fn toggle_auto_rotate(app: &tauri::AppHandle) {
    let result = modify_config(app, |config| {
        config.auto_rotate.enabled = !config.auto_rotate.enabled;
        Ok(())
    });
    if result.is_ok() {
        reset_rotation_timer(app);
    }
}

fn toggle_favorites_only(app: &tauri::AppHandle) {
    let _ = modify_config(app, |config| {
        config.auto_rotate.favorites_only = !config.auto_rotate.favorites_only;
        Ok(())
    });
}

pub fn setup_system_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let config = app.state::<ConfigState>().0.lock().unwrap().clone();

    let text = labels();

    let show_item = MenuItem::with_id(app, "show", &text.show, true, None::<&str>)?;
    let next_item = MenuItem::with_id(app, "next", &text.next, true, None::<&str>)?;
    let auto_rotate_item = CheckMenuItem::with_id(
        app,
        "auto_rotate",
        &text.auto_rotate,
        true,
        config.auto_rotate.enabled,
        None::<&str>,
    )?;
    let favorites_only_item = CheckMenuItem::with_id(
        app,
        "favorites_only",
        &text.favorites_only,
        true,
        config.auto_rotate.favorites_only,
        None::<&str>,
    )?;
    let add_pictures_item = MenuItem::with_id(app, "add_pictures", &text.open_folder, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", &text.settings, true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", &text.reload, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", &text.quit, true, None::<&str>)?;

    let menu = tauri::menu::MenuBuilder::new(app)
        .item(&show_item)
        .item(&next_item)
        .separator()
        .item(&auto_rotate_item)
        .item(&favorites_only_item)
        .separator()
        .item(&add_pictures_item)
        .item(&settings_item)
        .item(&reload_item)
        .separator()
        .item(&quit_item)
        .build()?;

    app.manage(TrayMenuState {
        show: show_item,
        next: next_item,
        auto_rotate: auto_rotate_item,
        favorites_only: favorites_only_item,
        open_folder: add_pictures_item,
        settings: settings_item,
        reload: reload_item,
        quit: quit_item,
    });

    let mut tray_builder = tauri::tray::TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Easy Paper")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    toggle_main_window(app);
                }
                "next" => {
                    next_wallpaper(app);
                }
                "auto_rotate" => {
                    toggle_auto_rotate(app);
                }
                "favorites_only" => {
                    toggle_favorites_only(app);
                }
                "add_pictures" => {
                    open_configured_images_folder(app);
                }
                "settings" => {
                    show_settings_window(app);
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
