mod compress;
pub mod time_utils;

use winapi::um::shellapi::{Shell_NotifyIconW, NIM_MODIFY, NIF_INFO, NOTIFYICONDATAW, NOTIFYICONDATAW_u, NIM_ADD, NIF_ICON, NIF_MESSAGE, NIF_TIP};
use winapi::um::winuser::{CreateWindowExW, RegisterClassW, WNDCLASSW, LoadIconW, LoadCursorW, IDC_ARROW, LoadImageW, IDI_APPLICATION, WS_OVERLAPPED, WS_SYSMENU, CW_USEDEFAULT, UpdateWindow, WM_USER};
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
use winapi::shared::minwindef::HINSTANCE;

pub(crate) fn show_win_toast(title: String, content: String) {
    unsafe {
        let null_str: Vec<u16> = to_wide("");
        let class_name: Vec<u16> = to_wide("PythonTaskbar");
        println!("runing here!!00");
        let hinst = GetModuleHandleW(0 as LPCWSTR);
        let style = WS_OVERLAPPED | WS_SYSMENU;
        println!("runing here!!0011111");
        let icon = LoadIconW(0 as HINSTANCE, IDI_APPLICATION);
        let cursor = LoadCursorW(hinst,IDC_ARROW);
        let background = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;

        let mut wc = WNDCLASSW::default();
        wc.hInstance = hinst;
        wc.lpszClassName = class_name.as_ptr();
        // let wc = WNDCLASSW {
        //     style: 0,
        //     lpfnWndProc: None,
        //     cbClsExtra: 0,
        //     cbWndExtra: 0,
        //     hInstance: hinst,
        //     hIcon: icon,
        //     hCursor: cursor,
        //     hbrBackground: background,
        //     lpszMenuName: null_str.as_ptr(),
        //     lpszClassName: class_name.as_ptr()
        // };

        println!("runing here!!01");

        let class_atom = RegisterClassW(&wc);
        let mut wide: Vec<u16> = to_wide("Taskbar");
        let hwnd = CreateWindowExW(class_atom as u32, wide.as_ptr(), null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                   CW_USEDEFAULT,
                                   0 as HWND, 0 as HMENU, hinst, Box::into_raw(Box::new(0)) as *mut c_void);
        UpdateWindow(hwnd);

        println!("runing here!!02");
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

        // let mut params = NOTIFYICONDATAW {
        //     cbSize: 0,
        //     hWnd: hwnd,
        //     uID: 0,
        //     uFlags: 0,
        //     uCallbackMessage: 0,
        //     hIcon: icon,
        //     szTip: tip_arr,
        //     dwState: 0,
        //     dwStateMask: 0,
        //     szInfo: content_arr,
        //     u: a,
        //     szInfoTitle: title_arr,
        //     dwInfoFlags: 0,
        //     guidItem: GUID {
        //         Data1: 0,
        //         Data2: 0,
        //         Data3: 0,
        //         Data4: [0; 8]
        //     },
        //     hBalloonIcon: icon
        // };
        println!("runing here!!22");
        let mut params = NOTIFYICONDATAW::default();
        params.hWnd = hwnd;
        params.szInfo = content_arr;
        params.szInfoTitle = title_arr;
        params.szTip = tip_arr;
        params.hIcon = icon;
        let flags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        //params.uID = WM_USER + 20;
        params.uFlags = flags;
        Shell_NotifyIconW(NIM_ADD, &mut params);

        let mut tip_arr2: [u16; 128] = [0; 128];
        tip_ptr = to_wide("Balloon Tooltip");
        for i in 0..tip_ptr.len() {
            // println!("{}", tip_ptr[i] as char);
            tip_arr2[i] = tip_ptr[i];
        }
        let mut params2 = NOTIFYICONDATAW::default();
        params2.hWnd = hwnd;
        params2.szInfo = content_arr;
        params2.szInfoTitle = title_arr;
        params2.szTip = tip_arr2;
        params2.hIcon = icon;
        let flags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        //params.uID = WM_USER + 20;
        // *(params2.u) = 200;
        Shell_NotifyIconW(NIM_MODIFY,&mut params2);
        let ten_millis = time::Duration::from_secs(10);
        thread::sleep(ten_millis);
    }
}

fn to_wide(str: &str) -> Vec<u16> {
    str.encode_utf16().collect()
}

