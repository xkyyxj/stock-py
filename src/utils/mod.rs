mod compress;
pub mod time_utils;

use winapi::um::shellapi:: { Shell_NotifyIconW, NIM_MODIFY, NIF_INFO, NOTIFYICONDATAW };
use winapi::um::winuser::{ CreateWindowExW, RegisterClassW, WNDCLASSW, LoadIconW, LoadCursorW, IDC_ARROW,
                           LoadImageW, IDI_APPLICATION, WS_OVERLAPPED, WS_SYSMENU, CW_USEDEFAULT, UpdateWindow };
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{GetStockObject, WHITE_BRUSH};
use winapi::shared::windef::HBRUSH;
use winapi::_core::iter::once;
use winapi::ctypes::c_void;
use std::ptr::{null_mut, null};
use std::ffi::OsStr;

pub(crate) fn show_win_toast(title: String, content: String) {
    unsafe {
        let null_str: Vec<u16> = to_wide("");
        let class_name: Vec<u16> = to_wide("PythonTaskbar");
        let hinst = GetModuleHandleW(null_str.as_ptr());
        let style = WS_OVERLAPPED | WS_SYSMENU;
        let icon = LoadIconW(hinst, IDI_APPLICATION);
        let cursor = LoadCursorW(hinst,IDC_ARROW);
        let background = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: None,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: icon,
            hCursor: cursor,
            hbrBackground: background,
            lpszMenuName: null_str.as_ptr(),
            lpszClassName: class_name.as_ptr()
        };

        let class_atom = RegisterClassW(&wc);
        let mut wide: Vec<u16> = to_wide("Taskbar");
        let hwnd = CreateWindowExW(class_atom as u32, wide.as_ptr(), null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                   CW_USEDEFAULT,
                                   0, 0, hinst, Box::into_raw(Box::new(3)) as *mut c_void);
        UpdateWindow(hwnd);

        let title_ptr = to_wide(title.as_str());
        let mut params = NOTIFYICONDATAW::default();
        params.hWnd = hwnd;
        params.szInfoTitle = &title_ptr;
        Shell_NotifyIconW(NIM_MODIFY,&mut params);
    }
}

fn to_wide(str: &str) -> Vec<u16> {
    str.encode_utf16().collect()
}

