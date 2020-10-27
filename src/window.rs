use std::sync::mpsc::Sender;
use std::convert::TryInto;

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
            self, CW_USEDEFAULT, WM_DESTROY, WNDCLASSW, MSG, GWL_EXSTYLE, GWL_STYLE, LWA_COLORKEY, LWA_ALPHA, SB_BOTH,
            PAINTSTRUCT, COLOR_WINDOW, WM_PAINT, VK_ESCAPE, WM_CHAR, WM_LBUTTONUP,
            WS_OVERLAPPEDWINDOW, WS_OVERLAPPED, WS_CHILDWINDOW, WS_POPUP, WS_SYSMENU, WS_CAPTION, WS_BORDER, WS_HSCROLL, SW_SHOW, WS_DISABLED,
            WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_EX_LAYERED, WS_EX_WINDOWEDGE,
        },
        errhandlingapi,
        wingdi::RGB,
    },
};

use crate::geometry::Rectangle;
use crate::util::to_os_string;
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Window {
    pub hwnd: HWND,
    pub hmenu: Option<HMENU>,
}

impl Window {
    pub fn new(class: WNDCLASSW, name: &str, bounds: Rectangle<c_int>, _parent: Option<HWND>) -> Result<Window> {
        let hwnd;
        unsafe {
            hwnd = winuser::CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                class.lpszClassName,
                to_os_string(name).as_ptr(),
                WS_POPUP,
                bounds.x(),
                bounds.y(),
                bounds.w(),
                bounds.h(),
                0 as HWND,
                0 as HMENU,
                class.hInstance,
                std::ptr::null_mut(),
            );

            if hwnd == std::ptr::null_mut() {
                return Err(Error::from("Error creating window".to_string()));
            }

            winuser::SetLayeredWindowAttributes(hwnd, 0, 100, LWA_ALPHA);

            //winuser::ShowWindow(hwnd, SW_SHOW);
        }

        Ok(Window {
            hwnd,
            hmenu: None,
        })
    }
}