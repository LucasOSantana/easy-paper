use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};
use tauri::Manager;
use windows::core::{s, w, GUID, HRESULT, HSTRING, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, ERROR_NOT_FOUND, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleW, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowExW, FindWindowW, GetMessageW,
    GetWindowThreadProcessId, PostMessageW, RegisterClassW, RegisterWindowMessageW, HWND_MESSAGE,
    MSG, WINDOW_EX_STYLE, WM_APP, WNDCLASSW, WS_POPUP,
};
use crate::state::ConfigState;

const TAP_DLL_NAME: &str = "taskbar_tap.dll";
const TAP_CLSID: GUID = GUID::from_u128(0x6f3c1a52_8e4b_4d17_9a2c_3b7e5d901f48);
const CONTROL_WINDOW_CLASS: PCWSTR = w!("EasyPaperTaskbarTapControl");
const WM_SET_TRANSPARENT: u32 = WM_APP + 1;
const INIT_TRANSPARENT: PCWSTR = w!("1");

// XAML rejects endpoint names not in the form `VisualDiagConnection{N}` with ERROR_NOT_FOUND.
const DIAGNOSTICS_ENDPOINT: PCWSTR = w!("VisualDiagConnection1");
const INJECT_ATTEMPTS: u32 = 20;
const INJECT_RETRY_DELAY: Duration = Duration::from_millis(500);
const TAP_READY_TIMEOUT: Duration = Duration::from_secs(5);
const TAP_READY_POLL_INTERVAL: Duration = Duration::from_millis(50);

const RESTART_WATCHER_CLASS: PCWSTR = w!("EasyPaperExplorerRestartWatcher");

type InitializeXamlDiagnosticsEx = unsafe extern "system" fn(
    endpoint_name: PCWSTR,
    process_id: u32,
    xaml_diagnostics_dll: PCWSTR,
    tap_dll: PCWSTR,
    tap_clsid: GUID,
    initialization_data: PCWSTR,
) -> HRESULT;

pub fn set_taskbar_transparency<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    transparent: bool,
) -> Result<(), String> {
    static APPLY_LOCK: Mutex<()> = Mutex::new(());
    let _serialized = APPLY_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

    if !send_to_tap(transparent) && transparent {
        inject_into_explorer(&tap_dll_path(app)?)?;
        wait_until_tap_ready()?;
    }

    Ok(())
}

pub fn apply_saved_taskbar_transparency<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let transparent = app.state::<ConfigState>().0.lock().unwrap().taskbar_transparent;
    set_taskbar_transparency(app, transparent)
}

pub fn watch_explorer_restarts<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    static ON_TASKBAR_CREATED: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

    let app = app.clone();
    let _ = ON_TASKBAR_CREATED.set(Box::new(move || {
        let _ = apply_saved_taskbar_transparency(&app);
    }));

    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) } {
            if let Some(callback) = ON_TASKBAR_CREATED.get() {
                callback();
            }
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }

    let _ = std::thread::Builder::new()
        .name("explorer-restart-watcher".into())
        .spawn(|| unsafe {
            let Ok(instance) = GetModuleHandleW(None) else {
                return;
            };
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance.into(),
                lpszClassName: RESTART_WATCHER_CLASS,
                ..Default::default()
            };
            RegisterClassW(&class);

            let window = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                RESTART_WATCHER_CLASS,
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance.into()),
                None,
            );
            if window.is_err() {
                return;
            }

            let mut message = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).as_bool() {
                DispatchMessageW(&message);
            }
        });
}

fn tap_dll_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to locate application resources: {}", e))?
        .join(TAP_DLL_NAME);
    let bytes = fs::read(&bundled)
        .map_err(|e| format!("Taskbar component not found at \"{}\": {}", bundled.display(), e))?;

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let directory = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to obtain application data directory: {}", e))?;
    let copy = directory.join(format!("taskbar_tap-{:016x}.dll", hasher.finish()));

    if !copy.is_file() {
        fs::create_dir_all(&directory)
            .and_then(|_| fs::write(&copy, &bytes))
            .map_err(|e| format!("Failed to prepare the taskbar component: {}", e))?;
    }

    for entry in fs::read_dir(&directory).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("taskbar_tap-") && entry.path() != copy {
            let _ = fs::remove_file(entry.path());
        }
    }

    Ok(copy)
}

fn send_to_tap(transparent: bool) -> bool {
    let windows = control_windows();
    for window in &windows {
        let _ = unsafe {
            PostMessageW(Some(*window), WM_SET_TRANSPARENT, WPARAM(transparent as usize), LPARAM(0))
        };
    }
    !windows.is_empty()
}

fn control_windows() -> Vec<HWND> {
    let mut windows = Vec::new();
    let mut previous = None;
    while let Ok(window) =
        unsafe { FindWindowExW(Some(HWND_MESSAGE), previous, CONTROL_WINDOW_CLASS, PCWSTR::null()) }
    {
        windows.push(window);
        previous = Some(window);
    }
    windows
}

fn wait_until_tap_ready() -> Result<(), String> {
    let deadline = Instant::now() + TAP_READY_TIMEOUT;
    while control_windows().is_empty() {
        if Instant::now() >= deadline {
            return Err("The taskbar component was loaded but did not find the taskbar".to_string());
        }
        std::thread::sleep(TAP_READY_POLL_INTERVAL);
    }
    Ok(())
}

fn inject_into_explorer(tap_dll: &Path) -> Result<(), String> {
    let mut explorer_pid = 0;
    let taskbar = unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }
        .map_err(|_| "Windows taskbar was not found. Is Explorer running?")?;
    unsafe { GetWindowThreadProcessId(taskbar, Some(&mut explorer_pid)) };

    let xaml = unsafe { LoadLibraryExW(w!("Windows.UI.Xaml.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32) }
        .map_err(|e| format!("Failed to load the Windows XAML runtime: {}", e))?;
    let com_initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();

    let result = match unsafe { GetProcAddress(xaml, s!("InitializeXamlDiagnosticsEx")) } {
        Some(address) => {
            let initialize: InitializeXamlDiagnosticsEx = unsafe { std::mem::transmute(address) };
            let tap_dll = HSTRING::from(tap_dll);
            let mut result = HRESULT(0);
            for _ in 0..INJECT_ATTEMPTS {
                result = unsafe {
                    initialize(
                        DIAGNOSTICS_ENDPOINT,
                        explorer_pid,
                        w!(""),
                        PCWSTR(tap_dll.as_ptr()),
                        TAP_CLSID,
                        INIT_TRANSPARENT,
                    )
                };
                if result != ERROR_NOT_FOUND.to_hresult() {
                    break;
                }
                std::thread::sleep(INJECT_RETRY_DELAY);
            }
            result.ok().map_err(|e| format!("Failed to attach to the Windows taskbar: {}", e))
        }
        None => Err("This version of Windows does not support XAML diagnostics".to_string()),
    };

    let _ = unsafe { FreeLibrary(xaml) };
    if com_initialized {
        unsafe { CoUninitialize() };
    }
    result
}
