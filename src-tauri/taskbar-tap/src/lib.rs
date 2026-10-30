mod bindings {
    #![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, dead_code, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
mod tap_site;
mod taskbar_background;

use std::ffi::c_void;

use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_POINTER, S_FALSE};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::UI::WindowsAndMessaging::WM_APP;
use windows_core::{GUID, HRESULT, Interface, PCWSTR, w};

const TAP_CLSID: GUID = GUID::from_u128(0x6f3c1a52_8e4b_4d17_9a2c_3b7e5d901f48);
const CONTROL_WINDOW_CLASS: PCWSTR = w!("EasyPaperTaskbarTapControl");
const WM_SET_TRANSPARENT: u32 = WM_APP + 1;
const INIT_TRANSPARENT: &str = "1";

#[unsafe(no_mangle)]
extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    unsafe {
        *ppv = std::ptr::null_mut();
        if *rclsid != TAP_CLSID {
            return CLASS_E_CLASSNOTAVAILABLE;
        }

        let factory: IClassFactory = tap_site::TapClassFactory.into();
        factory.query(riid, ppv)
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}
