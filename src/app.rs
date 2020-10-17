use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError };

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
            self, CW_USEDEFAULT, WM_DESTROY, WNDCLASSW, MSG, WM_CLOSE,
            WS_OVERLAPPEDWINDOW, PAINTSTRUCT, COLOR_WINDOW, WM_PAINT, VK_ESCAPE, WM_CHAR, WM_LBUTTONUP
        },
        errhandlingapi,
    },
};

use log::{info, debug};

use crate::util::to_os_string;
use crate::event::{EVENT_HANDLER, Handler};
use crate::window::Window;
use crate::error::{Error, Result};

pub struct App {
    windows: HashMap<HWND, Window>,

    receiver: Receiver<HWND>,

    #[allow(dead_code)]
    window_count: u32,

    #[allow(dead_code)]
    instance: HINSTANCE,
    
    #[allow(dead_code)]
    class: WNDCLASSW,
}

impl App {
    pub fn new() -> Result<App> {
        //let (sender, receiver) = channel::<HWND>();
        unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
            match msg {
                WM_DESTROY => {
                    //winuser::SendMessageW(hwnd, WM_CLOSE, 0, 0 as LPARAM);
                    //winuser::DestroyWindow(hwnd);
                    EVENT_HANDLER.with(|handler| {
                        if let Some(h) = handler.borrow().as_ref() {
                            h.sender.send(hwnd).ok();
                        }
                    });
                    //winuser::PostQuitMessage(0);
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
                        //winuser::SendMessageW(hwnd, WM_CLOSE, 0, 0 as LPARAM);
                        winuser::DestroyWindow(hwnd);
                        //winuser::PostQuitMessage(0);
                        return 0;
                    }
        
                    return 0;
                },
                WM_LBUTTONUP => {
                    info!("window clicked: {:?}", hwnd);
                    return 0;
                }
                _ => return winuser::DefWindowProcW(hwnd, msg, w_param, l_param),
            };
        }
        let class_name = to_os_string("Sample Window Class");

        unsafe {
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
                return Err(Error::from("Error creating window class".to_string()));
            }

            let (handler, receiver) = Handler::new();

            EVENT_HANDLER.with(|h| {
                (*h.borrow_mut()) = Some(handler);
            });
            
            let mut windows = HashMap::new();

            let window = Window::new(wnd, "test window", None)?;

            debug!("window 1: {:?}", window);

            windows.insert(window.hwnd, window);

            let window2 = Window::new(wnd, "test window 2", None)?;

            debug!("window 2: {:?}", window2);

            windows.insert(window2.hwnd, window2);

            debug!("{}", windows.len());

            Ok(App {
                windows,
                receiver,
                window_count: 0,
                instance: hinstance,
                class: wnd,
            })
        }
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe {
            for (_, window) in self.windows.iter() {
                winuser::ShowWindow(window.hwnd, 1);
            }

            let mut msg = MSG {
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
                        return Err(Error::from(format!("error on GetMessage: {}", errhandlingapi::GetLastError())));
                    },
                    _ => (),
                }

                match self.receiver.try_recv() {
                    Ok(hwnd) => {
                        self.windows.remove(&hwnd);
                        debug!("{}", self.windows.len());
                        // match self.windows.remove(&hwnd) {
                        //     Some(w) => debug!("{:?}", w),
                        //     None => panic!("remove of same window twice"),
                        // }
                    },
                    Err(TryRecvError::Empty) => (),
                    Err(e) => panic!(e),
                }

                if self.windows.len() == 0 {
                    debug!("no windows, closing program");
                    winuser::PostQuitMessage(0);
                    break;
                }

                winuser::TranslateMessage(&mut msg);
                winuser::DispatchMessageW(&mut msg);
            }   
        }

        Ok(())
    }
}