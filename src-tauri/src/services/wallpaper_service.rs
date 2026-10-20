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
    use super::WallpaperServiceError;
    use std::ffi::c_void;
    use std::path::Path;

    use windows::core::{HSTRING, PCWSTR, PWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper};
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
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

    struct MonitorId(PWSTR);

    impl MonitorId {
        fn as_pcwstr(&self) -> PCWSTR {
            PCWSTR(self.0.as_ptr())
        }
    }

    impl Drop for MonitorId {
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
            .map(MonitorId::as_pcwstr)
            .unwrap_or(PCWSTR::null());

        unsafe { desktop_wallpaper.SetWallpaper(monitor_ptr, PCWSTR(wallpaper.as_ptr())) }
    }

    fn find_target_monitor(
        dw: &IDesktopWallpaper,
        (px, py): (i32, i32),
    ) -> windows::core::Result<Option<MonitorId>> {
        let count = unsafe { dw.GetMonitorDevicePathCount()? };

        for i in 0..count {
            let id = MonitorId(unsafe { dw.GetMonitorDevicePathAt(i)? });
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
