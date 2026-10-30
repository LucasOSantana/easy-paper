use std::time::Instant;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use windows::core::Interface;
use crate::state::WindowDisplayState;
use crate::services::shortcut_service::reload_shortcut_from_config;

#[cfg(target_os = "windows")]
pub fn get_cursor_position() -> Option<(i32, i32)> {
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    unsafe extern "system" {
        fn GetCursorPos(point: *mut POINT) -> i32;
    }

    let mut pt = POINT { x: 0, y: 0 };
    let res = unsafe { GetCursorPos(&mut pt) };
    if res != 0 {
        Some((pt.x, pt.y))
    } else {
        None
    }
}

pub fn resize_and_center_main_window(window: &tauri::WebviewWindow) {
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

pub fn toggle_main_window(app: &tauri::AppHandle) {
    reload_shortcut_from_config(app);

    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            hide_main_window(app);
        } else {
            resize_and_center_main_window(&window);
            if let Some(state) = app.try_state::<WindowDisplayState>() {
                *state.0.lock().unwrap() = Instant::now();
            }
            set_webview_active(&window, true);
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
}

pub fn hide_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        set_webview_active(&window, false);
    }
}

fn set_webview_active<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, active: bool) {
    let _ = window.with_webview(move |webview| unsafe {
        let controller = webview.controller();
        let _ = controller.SetIsVisible(active);

        let level = if active {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        };
        if let Ok(core) = controller.CoreWebView2().and_then(|core| core.cast::<ICoreWebView2_19>()) {
            let _ = core.SetMemoryUsageTargetLevel(level);
        }
    });
}

pub fn show_settings_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::default())
        .title("Easy Paper Settings")
        .inner_size(860.0, 580.0)
        .center()
        .build();
}
