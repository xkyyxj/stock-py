use std::ffi::OsStr;
use winapi::_core::iter::once;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::_core::{ptr, mem};
use winapi::um::winuser::{WS_SYSMENU, WS_OVERLAPPED, IDI_APPLICATION, LoadIconW, WNDCLASSEXW, DefWindowProcW, RegisterClassExW, CreateWindowExW, CW_USEDEFAULT, UpdateWindow, DestroyWindow, UnregisterClassW, WM_USER};
use winapi::um::shellapi::{NOTIFYICONDATAW, NIF_ICON, NIF_TIP, Shell_NotifyIconW, NIM_ADD, NIM_MODIFY, NIIF_INFO, NIF_INFO};
use std::{thread, time};
use winapi::shared::minwindef::HMODULE;
use winapi::shared::windef::HWND;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::os::windows::ffi::OsStrExt;

static CLASS_NAME: &str = "Taskbar01";
#[cfg(windows)]
pub struct WinToast {
    module_handler: HMODULE,
    class_handler: HWND,
    icon_id: u32,
}

impl WinToast {
    #[cfg(windows)]
    pub(crate) fn new() -> Self {
        unsafe {
            let null_str: Vec<u16> = OsStr::new("555").encode_wide().chain(once(0)).collect();
            let class_name: Vec<u16> = OsStr::new(CLASS_NAME).encode_wide().chain(once(0)).collect();
            let hinst = GetModuleHandleW(ptr::null());
            let style = WS_OVERLAPPED | WS_SYSMENU;
            let icon = LoadIconW(hinst, IDI_APPLICATION);
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
            // class_atom as u32
            let hwnd = CreateWindowExW(0, wc.lpszClassName, null_str.as_ptr(), style, 0, 0, CW_USEDEFAULT,
                                       CW_USEDEFAULT,
                                       ptr::null_mut(), ptr::null_mut(), hinst, ptr::null_mut());
            UpdateWindow(hwnd);

            let title_ptr: Vec<u16> = OsStr::new("").encode_wide().chain(once(0)).collect();
            let content_p: Vec<u16> = OsStr::new("").encode_wide().chain(once(0)).collect();
            let icon = LoadIconW(hinst, IDI_APPLICATION);
            let mut content_arr:[u16; 256] = [0; 256];
            for i in 0..content_p.len() {
                content_arr[i] = content_p[i];
            }
            let mut title_arr:[u16; 64] = [0; 64];
            for i in 0..title_ptr.len() {
                title_arr[i] = title_ptr[i];
            }
            let mut tip_arr: [u16; 128] = [0; 128];
            let mut tip_ptr: Vec<u16> = OsStr::new("Tooltip").encode_wide().chain(once(0)).collect();
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
            params.uID = WM_USER + 20;
            // if Shell_NotifyIconW(NIM_ADD, &mut params) == 0 {
            //     println!("Add failed");
            // }

            WinToast { module_handler: hinst, class_handler: hwnd, icon_id: WM_USER + 20 }

        }
    }

    #[cfg(windows)]
    pub(crate) fn show_win_toast(&mut self, title: String, content: String) {
        unsafe {
            let title_ptr: Vec<u16> = OsStr::new(title.as_str()).encode_wide().chain(once(0)).collect();
            let content_p: Vec<u16> = OsStr::new(content.as_str()).encode_wide().chain(once(0)).collect();
            let icon = LoadIconW(self.module_handler, IDI_APPLICATION);
            let mut content_arr:[u16; 256] = [0; 256];
            for i in 0..content_p.len() {
                content_arr[i] = content_p[i];
            }
            let mut title_arr:[u16; 64] = [0; 64];
            for i in 0..title_ptr.len() {
                title_arr[i] = title_ptr[i];
            }
            let mut tip_arr: [u16; 128] = [0; 128];
            let mut tip_ptr: Vec<u16> = OsStr::new("Tooltip").encode_wide().chain(once(0)).collect();
            for i in 0..tip_ptr.len() {
                // println!("{}", tip_ptr[i] as char);
                tip_arr[i] = tip_ptr[i];
            }

            let mut params = NOTIFYICONDATAW::default();
            params.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            params.hWnd = self.class_handler;
            params.szInfo = content_arr;
            params.szInfoTitle = title_arr;
            params.szTip = tip_arr;
            params.hIcon = icon;
            let flags = NIF_ICON | NIF_TIP;
            params.uID = WM_USER + 20;
            let flags = NIF_ICON | NIF_TIP;
            params.uFlags = flags;
            if Shell_NotifyIconW(NIM_ADD, &mut params) == 0 {
                println!("Add failed");
            }
            params.uFlags = NIF_INFO;
            params.dwInfoFlags = NIIF_INFO;
            if Shell_NotifyIconW(NIM_MODIFY,&mut params) == 0 {
                println!("modi failed");
            }
        }
        self.icon_id = self.icon_id + 1;
    }
}

impl Drop for WinToast {
    #[cfg(windows)]
    fn drop(&mut self) {
        unsafe {
            let class_name: Vec<u16> = OsStr::new(CLASS_NAME).encode_wide().chain(once(0)).collect();
            DestroyWindow(self.class_handler);
            UnregisterClassW(class_name.as_ptr(), self.module_handler);
        }
    }
}