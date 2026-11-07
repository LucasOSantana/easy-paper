use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum WallpaperServiceError {
    FileNotFound,
    UnsupportedPlatform,
    #[cfg(target_os = "windows")]
    Windows(windows::core::Error),
}

impl fmt::Display for WallpaperServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound => write!(f, "Specified wallpaper image was not found"),
            Self::UnsupportedPlatform => write!(f, "Setting desktop wallpaper is unsupported on this operating system"),
            #[cfg(target_os = "windows")]
            Self::Windows(err) => write!(f, "Windows desktop wallpaper API error: {}", err),
        }
    }
}

impl std::error::Error for WallpaperServiceError {}

#[cfg(target_os = "windows")]
impl From<windows::core::Error> for WallpaperServiceError {
    fn from(err: windows::core::Error) -> Self {
        Self::Windows(err)
    }
}

/// Wallpaper currently shown on an active monitor.
pub struct MonitorWallpaper {
    /// 1-based position among the active monitors.
    pub monitor: u32,
    pub path: std::path::PathBuf,
}

/// Image currently shown on each active monitor. Monitors showing a solid
/// color (no image) are left out.
pub fn current_wallpapers() -> Vec<MonitorWallpaper> {
    #[cfg(target_os = "windows")]
    {
        windows_impl::current_wallpapers()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

/// Applies a wallpaper to every active monitor, one at a time. For each monitor
/// `choose` receives the wallpaper currently shown there (when known) and
/// returns the image to apply, or `None` to stop. Returns how many monitors
/// were updated. Falls back to a single desktop-wide wallpaper when the
/// per-monitor API is unavailable.
pub fn apply_to_each_monitor(
    choose: &mut dyn FnMut(Option<&Path>) -> Option<std::path::PathBuf>,
) -> Result<usize, WallpaperServiceError> {
    #[cfg(target_os = "windows")]
    {
        windows_impl::apply_to_each_monitor(choose)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = choose;
        Err(WallpaperServiceError::UnsupportedPlatform)
    }
}

pub fn set_desktop_wallpaper(
    path: &str,
    target_point: Option<(i32, i32)>,
) -> Result<(), WallpaperServiceError> {
    #[cfg(target_os = "windows")]
    {
        windows_impl::set_wallpaper(Path::new(path), target_point)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (path, target_point);
        Err(WallpaperServiceError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{MonitorWallpaper, WallpaperServiceError};
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};

    use windows::core::{HSTRING, PCWSTR, PWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper};
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
        SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    struct ComGuard(bool);

    impl ComGuard {
        fn new() -> Self {
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            Self(hr.is_ok())
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    struct CoTaskString(PWSTR);

    impl CoTaskString {
        fn as_pcwstr(&self) -> PCWSTR {
            PCWSTR(self.0.as_ptr())
        }
    }

    impl Drop for CoTaskString {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CoTaskMemFree(Some(self.0.as_ptr() as *const c_void)) };
            }
        }
    }

    pub fn set_wallpaper(
        path: &Path,
        target_point: Option<(i32, i32)>,
    ) -> Result<(), WallpaperServiceError> {
        if !path.exists() {
            return Err(WallpaperServiceError::FileNotFound);
        }

        let canonical_path = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let wallpaper_hstring = HSTRING::from(canonical_path.as_os_str());

        match set_via_com(&wallpaper_hstring, target_point) {
            Ok(()) => Ok(()),
            Err(_) => set_via_spi(&wallpaper_hstring),
        }
    }

    pub fn apply_to_each_monitor(
        choose: &mut dyn FnMut(Option<&Path>) -> Option<PathBuf>,
    ) -> Result<usize, WallpaperServiceError> {
        if let Ok(applied) = apply_each_via_com(choose) {
            if applied > 0 {
                return Ok(applied);
            }
        }

        match choose(None) {
            Some(path) => set_wallpaper(&path, None).map(|_| 1),
            None => Ok(0),
        }
    }

    fn apply_each_via_com(
        choose: &mut dyn FnMut(Option<&Path>) -> Option<PathBuf>,
    ) -> windows::core::Result<usize> {
        let _com = ComGuard::new();

        let desktop_wallpaper: IDesktopWallpaper =
            unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)? };

        let count = unsafe { desktop_wallpaper.GetMonitorDevicePathCount()? };
        let mut applied = 0;

        for i in 0..count {
            let Ok(raw_id) = (unsafe { desktop_wallpaper.GetMonitorDevicePathAt(i) }) else {
                continue;
            };
            let id = CoTaskString(raw_id);

            // Detached monitors are still listed but have no rectangle.
            if unsafe { desktop_wallpaper.GetMonitorRECT(id.as_pcwstr()) }.is_err() {
                continue;
            }

            let current = read_monitor_wallpaper(&desktop_wallpaper, &id);

            let Some(next) = choose(current.as_deref()) else {
                break;
            };

            let canonical = dunce::canonicalize(&next).unwrap_or(next);
            let wallpaper = HSTRING::from(canonical.as_os_str());

            if unsafe { desktop_wallpaper.SetWallpaper(id.as_pcwstr(), PCWSTR(wallpaper.as_ptr())) }
                .is_ok()
            {
                applied += 1;
            }
        }

        Ok(applied)
    }

    fn read_monitor_wallpaper(dw: &IDesktopWallpaper, id: &CoTaskString) -> Option<PathBuf> {
        unsafe { dw.GetWallpaper(id.as_pcwstr()) }
            .ok()
            .map(CoTaskString)
            .and_then(|value| unsafe { value.0.to_string() }.ok())
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
    }

    pub fn current_wallpapers() -> Vec<MonitorWallpaper> {
        match current_via_com() {
            Ok(list) if !list.is_empty() => list,
            _ => current_via_spi()
                .map(|path| vec![MonitorWallpaper { monitor: 1, path }])
                .unwrap_or_default(),
        }
    }

    fn current_via_com() -> windows::core::Result<Vec<MonitorWallpaper>> {
        let _com = ComGuard::new();

        let desktop_wallpaper: IDesktopWallpaper =
            unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)? };

        let count = unsafe { desktop_wallpaper.GetMonitorDevicePathCount()? };
        let mut list = Vec::new();
        let mut active = 0;

        for i in 0..count {
            let Ok(raw_id) = (unsafe { desktop_wallpaper.GetMonitorDevicePathAt(i) }) else {
                continue;
            };
            let id = CoTaskString(raw_id);

            if unsafe { desktop_wallpaper.GetMonitorRECT(id.as_pcwstr()) }.is_err() {
                continue;
            }
            active += 1;

            if let Some(path) = read_monitor_wallpaper(&desktop_wallpaper, &id) {
                list.push(MonitorWallpaper { monitor: active, path });
            }
        }

        Ok(list)
    }

    fn current_via_spi() -> Option<PathBuf> {
        let mut buffer = [0u16; 1024];
        unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .ok()?;
        }

        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        let value = String::from_utf16_lossy(&buffer[..len]);
        (!value.trim().is_empty()).then(|| PathBuf::from(value))
    }

    fn set_via_com(
        wallpaper: &HSTRING,
        target_point: Option<(i32, i32)>,
    ) -> windows::core::Result<()> {
        let _com = ComGuard::new();

        let desktop_wallpaper: IDesktopWallpaper =
            unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)? };

        let monitor = match target_point {
            Some(point) => find_target_monitor(&desktop_wallpaper, point)?,
            None => None,
        };

        let monitor_ptr = monitor
            .as_ref()
            .map(CoTaskString::as_pcwstr)
            .unwrap_or(PCWSTR::null());

        unsafe { desktop_wallpaper.SetWallpaper(monitor_ptr, PCWSTR(wallpaper.as_ptr())) }
    }

    fn find_target_monitor(
        dw: &IDesktopWallpaper,
        (px, py): (i32, i32),
    ) -> windows::core::Result<Option<CoTaskString>> {
        let count = unsafe { dw.GetMonitorDevicePathCount()? };

        for i in 0..count {
            let id = CoTaskString(unsafe { dw.GetMonitorDevicePathAt(i)? });
            let rect = unsafe { dw.GetMonitorRECT(id.as_pcwstr())? };

            if px >= rect.left && px < rect.right && py >= rect.top && py < rect.bottom {
                return Ok(Some(id));
            }
        }

        Ok(None)
    }

    fn set_via_spi(wallpaper: &HSTRING) -> Result<(), WallpaperServiceError> {
        unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wallpaper.as_ptr() as *mut c_void),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )?;
        }
        Ok(())
    }
}
