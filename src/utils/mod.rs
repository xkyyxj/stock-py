mod compress;
pub mod time_utils;

use winapi::um::shellapi:: { Shell_NotifyIconW, NIM_MODIFY, NIF_INFO, NOTIFYICONDATAW };
use winapi::um::winuser::{ CreateWindowExW, RegisterClassW, WNDCLASSW, WS_OVERLAPPED, WS_SYSMENU, CW_USEDEFAULT, UpdateWindow };
use winapi::um::libloaderapi::GetModuleHandleW;
use std::ffi::OsStr;
use winapi::_core::iter::once;
use winapi::ctypes::c_void;
use std::ptr::{null_mut, null};

pub(crate) fn show_win_toast(title: String, content: String) {
    unsafe {
        let wc = WNDCLASSW::default();
        let classAtom = RegisterClassW(&wc);
        let null_str: Vec<u16> = OsStr::new("").encode_wide().chain(once(0)).collect();
        let hinst = GetModuleHandleW(null_str.as_ptr());
        let style = WS_OVERLAPPED | WS_SYSMENU;

        let mut wide: Vec<u16> = OsStr::new("Taskbar").encode_wide().chain(once(0)).collect();
        let hwnd = CreateWindowExW(classAtom as u32, wide.as_ptr(), null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                   CW_USEDEFAULT,
                                   0, 0, hinst, null());
        UpdateWindow(hwnd);

        let title_ptr = OsStr::new(title.as_str()).encode_wide().chain(once(0)).collect();
        let mut params = NOTIFYICONDATAW::default();
        params.hWnd = hwnd;
        params.szInfoTitle = &title_ptr;
        Shell_NotifyIconW(NIM_MODIFY,&mut params);
    }
}

