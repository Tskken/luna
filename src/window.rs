use std::sync::mpsc::Sender;

use winapi::{
    ctypes::c_int,
    shared::{
        minwindef::{DWORD, HINSTANCE, LPARAM, LRESULT, FALSE, UINT, WPARAM},
        ntdef::LPCWSTR,
        windef::{HBRUSH, HICON, HMENU, HWND, POINT, HDC, RECT, HCURSOR},
    },
    um::{
        libloaderapi,
        winuser::{
            self, CW_USEDEFAULT, WM_DESTROY, WNDCLASSW, MSG,
            WS_OVERLAPPEDWINDOW, PAINTSTRUCT, COLOR_WINDOW, WM_PAINT, VK_ESCAPE, WM_CHAR, WM_LBUTTONUP
        },
        errhandlingapi,
    },
};

use crate::util::to_os_string;
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Window {
    pub hwnd: HWND,
    pub hmenu: Option<HMENU>,
}

impl Window {
    pub fn new(class: WNDCLASSW, name: &str, _parent: Option<HWND>) -> Result<Window> {
        let hwnd;
        unsafe {
            hwnd = winuser::CreateWindowExW(
                0,
                class.lpszClassName,
                to_os_string(name).as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0 as HWND,
                0 as HMENU,
                class.hInstance,
                std::ptr::null_mut(),
            );

            if hwnd == std::ptr::null_mut() {
                return Err(Error::from("Error creating window".to_string()));
            }
        }

        Ok(Window {
            hwnd,
            hmenu: None,
        })
    }
}