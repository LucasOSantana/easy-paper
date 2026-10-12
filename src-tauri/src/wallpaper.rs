use std::path::Path;

pub fn set_desktop_wallpaper(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err("File not found".into());
        }

        let full_path = std::fs::canonicalize(path_obj).unwrap_or_else(|_| path_obj.to_path_buf());
        let full_path_str = full_path.to_string_lossy().replace(r"\\?\", "");

        let path_wide: Vec<u16> = OsStr::new(&full_path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe extern "system" {
            fn SystemParametersInfoW(
                uiAction: u32,
                uiParam: u32,
                pvParam: *mut std::ffi::c_void,
                fWinIni: u32,
            ) -> i32;
        }

        const SPI_SETDESKWALLPAPER: u32 = 0x0014;
        const SPIF_UPDATEINIFILE: u32 = 0x01;
        const SPIF_SENDCHANGE: u32 = 0x02;

        let res = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                path_wide.as_ptr() as *mut std::ffi::c_void,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
        };

        if res != 0 {
            Ok(())
        } else {
            Err("Error setting desktop wallpaper".into())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}
