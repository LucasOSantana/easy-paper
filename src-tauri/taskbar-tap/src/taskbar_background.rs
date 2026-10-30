use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, HWND_MESSAGE, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WNDCLASSW,
};
use windows_core::{Interface, PCWSTR, Result};

use crate::bindings::IUIElement;
use crate::{CONTROL_WINDOW_CLASS, WM_SET_TRANSPARENT};

const TASKBAR_BACKGROUND_TYPE: &str = "Taskbar.TaskbarBackground";
const RECTANGLE_TYPE: &str = "Windows.UI.Xaml.Shapes.Rectangle";
const BACKGROUND_ELEMENT_NAMES: [&str; 2] = ["BackgroundFill", "BackgroundStroke"];
const HIDDEN_OPACITY: f64 = 0.0;

static TRANSPARENT: AtomicBool = AtomicBool::new(false);
static REGISTER_CLASS: Once = Once::new();

struct BackgroundElement {
    handle: u64,
    element: IUIElement,
    original_opacity: f64,
}

#[derive(Default)]
struct ThreadRegistry {
    taskbar_handles: HashSet<u64>,
    backgrounds: Vec<BackgroundElement>,
    has_control_window: bool,
}

thread_local! {
    static REGISTRY: RefCell<ThreadRegistry> = RefCell::default();
}

pub fn set_transparent(enabled: bool) {
    TRANSPARENT.store(enabled, Ordering::Relaxed);
}

pub fn track_added(parent: u64, handle: u64, type_name: &str, name: &str) -> bool {
    REGISTRY.with_borrow_mut(|registry| {
        let inside_taskbar =
            type_name == TASKBAR_BACKGROUND_TYPE || registry.taskbar_handles.contains(&parent);
        if !inside_taskbar {
            return false;
        }

        registry.taskbar_handles.insert(handle);
        type_name == RECTANGLE_TYPE && BACKGROUND_ELEMENT_NAMES.contains(&name)
    })
}

pub fn register(handle: u64, element: IUIElement) {
    let Ok(original_opacity) = opacity(&element) else {
        return;
    };

    if TRANSPARENT.load(Ordering::Relaxed) {
        let _ = set_opacity(&element, HIDDEN_OPACITY);
    }

    let needs_control_window = REGISTRY.with_borrow_mut(|registry| {
        registry.backgrounds.push(BackgroundElement { handle, element, original_opacity });
        !std::mem::replace(&mut registry.has_control_window, true)
    });

    if needs_control_window {
        create_control_window();
    }
}

pub fn track_removed(handle: u64) {
    let removed: Vec<BackgroundElement> = REGISTRY.with_borrow_mut(|registry| {
        registry.taskbar_handles.remove(&handle);
        registry.backgrounds.extract_if(.., |background| background.handle == handle).collect()
    });
    drop(removed);
}

fn apply_to_tracked() {
    let targets: Vec<(IUIElement, f64)> = REGISTRY.with_borrow(|registry| {
        registry
            .backgrounds
            .iter()
            .map(|background| (background.element.clone(), background.original_opacity))
            .collect()
    });

    let transparent = TRANSPARENT.load(Ordering::Relaxed);
    for (element, original_opacity) in targets {
        let target = if transparent { HIDDEN_OPACITY } else { original_opacity };
        let _ = set_opacity(&element, target);
    }
}

fn opacity(element: &IUIElement) -> Result<f64> {
    let mut value = 0.0;
    unsafe { (Interface::vtable(element).Opacity)(Interface::as_raw(element), &mut value).ok()? };
    Ok(value)
}

fn set_opacity(element: &IUIElement, value: f64) -> Result<()> {
    unsafe { (Interface::vtable(element).SetOpacity)(Interface::as_raw(element), value).ok() }
}

fn create_control_window() {
    REGISTER_CLASS.call_once(|| {
        let class = WNDCLASSW {
            lpfnWndProc: Some(control_window_proc),
            lpszClassName: CONTROL_WINDOW_CLASS,
            ..Default::default()
        };
        unsafe { RegisterClassW(&class) };
    });

    let _ = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CONTROL_WINDOW_CLASS,
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
    };
}

unsafe extern "system" fn control_window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_SET_TRANSPARENT {
        set_transparent(wparam.0 != 0);
        apply_to_tracked();
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}
