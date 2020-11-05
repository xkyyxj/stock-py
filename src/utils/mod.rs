mod compress;
pub mod time_utils;

use winapi::um::shellapi::{Shell_NotifyIconW, NIM_MODIFY, NIF_INFO, NOTIFYICONDATAW, NOTIFYICONDATAW_u, NIM_ADD, NIF_ICON, NIF_MESSAGE, NIF_TIP};
use winapi::um::winuser::{WS_EX_TOPMOST, CreateWindowExW, RegisterClassW, WNDCLASSW, LoadIconW, WM_DESTROY, LoadCursorW, IDC_ARROW, LoadImageW, IDI_APPLICATION, WS_OVERLAPPED, WS_SYSMENU, CW_USEDEFAULT, ShowWindow, UpdateWindow, WM_USER, SW_MINIMIZE, WNDCLASSEXW, RegisterClassExA, RegisterClassExW};
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
use winapi::_core::ptr;

pub(crate) fn show_win_toast(title: String, content: String) {
    unsafe {
        // let val2222 = OsStr::new("12312").encode_wide().chain(once(0)).collect();
        let str = String::from("Taskbar");
        let str_null = String::from("");
        let null_str: Vec<u16> = to_wide(&str_null);
        let class_name: Vec<u16> = to_wide(&str);
        let hinst = GetModuleHandleW(0 as LPCWSTR);
        // if hinst as u32 == 0 {
        //     println!("faild get module");
        // }
        let style = WS_OVERLAPPED | WS_SYSMENU;
        // let icon = LoadIconW(0 as HINSTANCE, IDI_APPLICATION);
        // //let cursor = LoadCursorW(hinst,IDC_ARROW);
        // let background = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;
        //
        let mut wc = WNDCLASSEXW::default();
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        let class_atom = RegisterClassExW(&wc);
        // println!("OK");
        // if class_atom == 0 {
        //     println!("register cclaass failed！")
        // }

        let str2 = String::from("Toolbar");
        let mut wide: Vec<u16> = to_wide(&str2);
        let hwnd = CreateWindowExW(0, wc.lpszClassName, null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                   CW_USEDEFAULT,
                                   0 as HWND, 0 as HMENU, hinst, ptr::null_mut());
        // UpdateWindow(hwnd);

        // let title_ptr = to_wide(title.as_str());
        // let content_p = to_wide(content.as_str());
        // let mut content_arr:[u16; 256] = [0; 256];
        // for i in 0..content_p.len() {
        //     content_arr[i] = content_p[i];
        // }
        // let mut title_arr:[u16; 64] = [0; 64];
        // for i in 0..title_ptr.len() {
        //     title_arr[i] = title_ptr[i];
        // }
        // let mut tip_arr: [u16; 128] = [0; 128];
        // let mut tip_ptr = to_wide("Tooltip");
        // for i in 0..tip_ptr.len() {
        //     // println!("{}", tip_ptr[i] as char);
        //     tip_arr[i] = tip_ptr[i];
        // }
        //
        // let mut params = NOTIFYICONDATAW::default();
        // params.hWnd = hwnd;
        // params.szInfo = content_arr;
        // params.szInfoTitle = title_arr;
        // params.szTip = tip_arr;
        // params.hIcon = icon;
        // let flags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        // params.uID = WM_USER + 20;
        // params.uFlags = flags;
        // Shell_NotifyIconW(NIM_ADD, &mut params);
        //
        // let mut tip_arr2: [u16; 128] = [0; 128];
        // tip_ptr = to_wide("Balloon Tooltip");
        // for i in 0..tip_ptr.len() {
        //     // println!("{}", tip_ptr[i] as char);
        //     tip_arr2[i] = tip_ptr[i];
        // }
        // let mut params2 = NOTIFYICONDATAW::default();
        // params2.hWnd = hwnd;
        // params2.szInfo = content_arr;
        // params2.szInfoTitle = title_arr;
        // params2.szTip = tip_arr2;
        // params2.hIcon = icon;
        // params2.uFlags = NIF_INFO;
        // params.uID = WM_USER + 20;
        // // params2.u = NOTIFYICONDATAW_u{ 0: [-1] };
        // Shell_NotifyIconW(NIM_MODIFY,&mut params2);
        // let ten_millis = time::Duration::from_secs(10);
        // thread::sleep(ten_millis);
    }
}

fn destroy(hwnd: HWND,val1: UINT,param1: WPARAM,lparam: LPARAM) -> LRESULT {
    0 as LRESULT
}

fn to_wide(str: &String) -> Vec<u16> {
    let a = ATime{ 0: [0] };
    str.as_str().encode_utf16().collect()
}

