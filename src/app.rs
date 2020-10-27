use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError };
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

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
            self, CW_USEDEFAULT, WM_DESTROY, WNDCLASSW, MSG, WM_CLOSE, SW_SHOW,
            WS_OVERLAPPEDWINDOW, WS_EX_TOOLWINDOW, PAINTSTRUCT, COLOR_WINDOW, COLOR_GRAYTEXT, WM_PAINT, VK_ESCAPE, WM_CHAR, WM_LBUTTONUP, WM_RBUTTONUP
        },
        errhandlingapi,
    },
};

use log::{info, debug};

use crate::geometry::Rectangle;
use crate::tray::Application;
use crate::util::to_os_string;
use crate::event::{EVENT_HANDLER, Handler};
use crate::window::Window;
use crate::error::{Error, Result};

pub struct App {
    windows: HashMap<HWND, Window>,

    tray: Application,

    receiver: Receiver<HWND>,

    running: Arc<AtomicBool>,

    #[allow(dead_code)]
    window_count: u32,

    #[allow(dead_code)]
    instance: HINSTANCE,
    
    #[allow(dead_code)]
    class: WNDCLASSW,
}

impl App {
    pub fn new() -> Result<App> {
        unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
            match msg {
                WM_DESTROY => {
                    EVENT_HANDLER.with(|handler| {
                        if let Some(h) = handler.borrow().as_ref() {
                            h.sender.send(hwnd).ok();
                        }
                    });
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
        
                    winuser::FillRect(hdc, &ps.rcPaint, COLOR_GRAYTEXT as HBRUSH);
        
                    winuser::EndPaint(hwnd, &ps);
                    return 0;
                },
                WM_CHAR => {
                    if w_param as c_int == VK_ESCAPE {
                        winuser::DestroyWindow(hwnd);
                        return 0;
                    }
        
                    return 0;
                },
                WM_LBUTTONUP => {
                    info!("window clicked: {:?}", hwnd);
                    return 0;
                },
                WM_RBUTTONUP => {
                    winuser::DestroyWindow(hwnd);
                    return 0;
                },
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
            {
                let window = Window::new(wnd, "test window", Rectangle::new(50, 50, 500, 500), None)?;

                debug!("window 1: {:?}", window);
    
                windows.insert(window.hwnd, window);
    
                let window2 = Window::new(wnd, "test window 2", Rectangle::new(600, 50, 500, 500), None)?;
    
                debug!("window 2: {:?}", window2);
    
                windows.insert(window2.hwnd, window2);
    
                debug!("{}", windows.len());
            }
            let mut tray = Application::new().unwrap();

            tray.set_icon("./src/rust.ico").unwrap();
        
            let running = Arc::new(AtomicBool::new(false));
        
            tray.add_menu_item("filler", |_| {
                println!("filler button");
                Ok::<_, Error>(())
            }).unwrap();
        
            tray.add_menu_separator().unwrap();
        
            let running_clone = running.clone();
            tray.add_menu_item("Quit", move |window| {
                println!("Quitting app");
                window.quit();
                running_clone.store(true, Ordering::SeqCst);
                Ok::<_, Error>(())
            }).unwrap();
        
            tray.set_tooltip("luna").unwrap();

            Ok(App {
                windows,
                tray,
                receiver,
                running,
                window_count: 0,
                instance: hinstance,
                class: wnd,
            })
        }
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe {
            for (_, window) in self.windows.iter() {
                winuser::ShowWindow(window.hwnd, SW_SHOW);
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
                self.tray.update().unwrap();
                match self.running.load(Ordering::SeqCst) {
                    true => {
                        debug!("closing luna from tray");
                        winuser::PostQuitMessage(0);
                    },
                    false => (),
                }

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
                        match self.windows.remove(&hwnd) {
                            Some(w) => debug!("{:?} removed", w),
                            None => panic!("remove of same window twice"),
                        }
                    },
                    Err(TryRecvError::Empty) => (),
                    Err(e) => panic!(e),
                }

                if self.windows.len() == 0 {
                    debug!("no windows, closing program");
                    self.tray.quit();
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