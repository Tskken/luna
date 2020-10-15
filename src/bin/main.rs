use std;
use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
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
            self, CW_USEDEFAULT, WM_DESTROY, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, PAINTSTRUCT, COLOR_WINDOW, WM_PAINT, VK_ESCAPE, WM_CHAR, WM_LBUTTONUP
        },
        errhandlingapi,
    },
};

use log::{error, info, debug};
use env_logger;


unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            winuser::PostQuitMessage(0);
            return 0;
        },
        WM_PAINT => {
            let mut ps = PAINTSTRUCT {
                hdc: 0 as HDC,
                fErase: FALSE,
                rcPaint: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                fRestore: FALSE,
                fIncUpdate: FALSE,
                rgbReserved: [0; 32],
            };
            let hdc = winuser::BeginPaint(hwnd, &mut ps);

            winuser::FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);

            winuser::EndPaint(hwnd, &ps);
            return 0;
        },
        WM_CHAR => {
            if w_param as c_int == VK_ESCAPE {
                winuser::PostQuitMessage(0);
                return 0;
            }

            return 0;
        },
        WM_LBUTTONUP => {
            info!("window clicked");
            return 0;
        }
        _ => return winuser::DefWindowProcW(hwnd, msg, w_param, l_param),
    };
}

struct WS {}

impl WS {
    fn to_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
    }
}

fn main () {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    unsafe {
        let class_name = WS::to_string("Sample Window Class");

        let hinstance: HINSTANCE = libloaderapi::GetModuleHandleW(std::ptr::null_mut());
        let wnd = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: 0 as HICON,
            hCursor: 0 as HCURSOR,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: class_name.as_ptr(),
        };
        if winuser::RegisterClassW(&wnd) == 0 {
            error!("Error creating window class");
        }
        let hwnd = winuser::CreateWindowExW(
            0,
            class_name.as_ptr(),
            WS::to_string("Learn to Program Windows").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0 as HWND,
            0 as HMENU,
            hinstance,
            std::ptr::null_mut(),
        );
        if hwnd == std::ptr::null_mut() {
            error!("Error creating window");
        }

        winuser::ShowWindow(hwnd, 1);

        let mut msg = winuser::MSG {
            hwnd: 0 as HWND,
            message: 0 as UINT,
            wParam: 0 as WPARAM,
            lParam: 0 as LPARAM,
            time: 0 as DWORD,
            pt: POINT { x: 0, y: 0 },
        };

        loop {
            match winuser::GetMessageW(&mut msg, 0 as HWND, 0, 0) {
                0 => {
                    debug!("closing window");
                    break;
                },
                -1 => {
                    error!("error on GetMessage: {}", errhandlingapi::GetLastError());
                    break;
                },
                _ => (),
            }
            winuser::TranslateMessage(&mut msg);
            winuser::DispatchMessageW(&mut msg);
        }     
    }
}