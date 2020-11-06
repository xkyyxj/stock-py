mod compress;
pub mod time_utils;
mod win_toast;

use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

use winapi::um::shellapi::{Shell_NotifyIconW, NIM_MODIFY, NIF_INFO, NOTIFYICONDATAW, NOTIFYICONDATAW_u, NIM_ADD, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIIF_INFO};
use winapi::um::winuser::{UnregisterClassW, DestroyWindow, WS_EX_TOPMOST, CreateWindowExW, RegisterClassW, WNDCLASSEXW, WNDCLASSW, LoadIconW, WM_DESTROY, LoadCursorW, IDC_ARROW, LoadImageW, IDI_APPLICATION, WS_OVERLAPPED, WS_SYSMENU, CW_USEDEFAULT, ShowWindow, UpdateWindow, WM_USER, SW_MINIMIZE, WS_EX_LEFT, WS_EX_ACCEPTFILES, RegisterClassExW, DefWindowProcW};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{GetStockObject, WHITE_BRUSH};
use winapi::shared::windef::{HBRUSH, HWND, HMENU};
use winapi::_core::iter::once;
use winapi::ctypes::c_void;
use std::ptr::{null_mut, null};
use std::ffi::OsStr;
use winapi::shared::guiddef::GUID;
use winapi::um::winnt::LPCWSTR;
use std::{time, thread};
use winapi::shared::minwindef::{HINSTANCE, UINT, WPARAM, LPARAM, LRESULT};

use crate::time::ATime;
use winapi::_core::{ptr, mem};
pub use win_toast::WinToast;

static CLASS_NAME: &str = "Taskbar01";

#[cfg(windows)]
pub(crate) fn show_win_toast(title: String, content: String) {
    unsafe {
        let val222: Vec<u16> = OsStr::new("123213213").encode_wide().chain(once(0)).collect();
        let null_str: Vec<u16> = OsStr::new("555").encode_wide().chain(once(0)).collect();
        let class_name: Vec<u16> = OsStr::new(CLASS_NAME).encode_wide().chain(once(0)).collect();
        let hinst = GetModuleHandleW(ptr::null());
        let style = WS_OVERLAPPED | WS_SYSMENU;
        let icon = LoadIconW(hinst, IDI_APPLICATION);
        //let cursor = LoadCursorW(hinst,IDC_ARROW);
        // let background = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;

        println!("failed 01 ");
        let mut wc = WNDCLASSEXW::default();
        // 原先报错可能是因为下面这一行
        wc.cbSize = mem::size_of::<WNDCLASSEXW>() as u32;
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        wc.hIcon = icon;
        wc.style = 0;
        wc.cbWndExtra = 0;
        wc.lpfnWndProc = Some(DefWindowProcW);
        let class_atom = RegisterClassExW(&wc);
        println!("class atom is {}", class_atom);
        if class_atom == 0 {
            println!("riririirri");
        }
        println!("failed 0201 ");
        let mut wide: Vec<u16> = OsStr::new(CLASS_NAME).encode_wide().chain(once(0)).collect();
        println!("failed 0202 ");
        // class_atom as u32
        let hwnd = CreateWindowExW(0, wc.lpszClassName, null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                   CW_USEDEFAULT,
                                   ptr::null_mut(), ptr::null_mut(), hinst, ptr::null_mut());
        println!("failed 0203 ");
        UpdateWindow(hwnd);
        println!("failed 02 ");

        let title_ptr = to_wide(title.as_str());
        let content_p = to_wide(content.as_str());
        let mut content_arr:[u16; 256] = [0; 256];
        for i in 0..content_p.len() {
            content_arr[i] = content_p[i];
        }
        let mut title_arr:[u16; 64] = [0; 64];
        for i in 0..title_ptr.len() {
            title_arr[i] = title_ptr[i];
        }
        let mut tip_arr: [u16; 128] = [0; 128];
        let mut tip_ptr = to_wide("Tooltip");
        for i in 0..tip_ptr.len() {
            // println!("{}", tip_ptr[i] as char);
            tip_arr[i] = tip_ptr[i];
        }

        let mut params = NOTIFYICONDATAW::default();
        params.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        params.hWnd = hwnd;
        params.szInfo = content_arr;
        params.szInfoTitle = title_arr;
        params.szTip = tip_arr;
        params.hIcon = icon;
        let flags = NIF_ICON | NIF_TIP;
        params.uID = 0;//WM_USER + 20;
        params.uFlags = flags;
        if Shell_NotifyIconW(NIM_ADD, &mut params) == 0 {
            println!("Add failed");
        }
        println!("failed 03 ");
        params.uFlags = NIF_INFO;
        params.dwInfoFlags = NIIF_INFO;
        //params.u = NOTIFYICONDATAW_u{ 0: [100000] };
        if Shell_NotifyIconW(NIM_MODIFY,&mut params) == 0 {
            println!("modi failed");
        }
        println!("all finished!");
        let ten_millis = time::Duration::from_secs(10);
        thread::sleep(ten_millis);
        if Shell_NotifyIconW(NIM_MODIFY,&mut params) == 0 {
            println!("modi failed");
        }
        DestroyWindow(hwnd);
        UnregisterClassW(wide.as_ptr(), hinst);
        println!("finished!!");
    }
}

#[cfg(windows)]
fn destroy(hwnd: HWND,val1: UINT,param1: WPARAM,lparam: LPARAM) -> LRESULT {
    0 as LRESULT
}

fn to_wide(str: &str) -> Vec<u16> {
    let a = ATime{ 0: [0] };
    str.encode_utf16().collect()
}

