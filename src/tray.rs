use std;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;
use std::convert::TryInto;
use winapi::{
    ctypes::{c_ulong, c_ushort},
    shared::{
        basetsd::ULONG_PTR,
        guiddef::GUID,
        minwindef::{DWORD, HINSTANCE, LPARAM, LRESULT, PBYTE, TRUE, FALSE, UINT, WPARAM},
        ntdef::LPCWSTR,
        windef::{HBITMAP, HBRUSH, HICON, HMENU, HWND, POINT},
    },
    um::{
        errhandlingapi, libloaderapi,
        shellapi::{
            self, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
        },
        winuser::{
            self, CW_USEDEFAULT, IMAGE_ICON, LR_DEFAULTCOLOR, LR_LOADFROMFILE, MENUINFO,
            MENUITEMINFOW, MFT_SEPARATOR, MFT_STRING, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING,
            MIM_APPLYTOSUBMENUS, MIM_STYLE, MNS_NOTIFYBYPOS, WM_DESTROY, WM_USER, WNDCLASSW,
            WS_OVERLAPPEDWINDOW, WM_MENUCOMMAND, WM_LBUTTONUP, WM_RBUTTONUP
        },
    },
};

struct WS {}

impl WS {
    fn to_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
    }
}

struct NID {}

impl NID {
    fn nid(hwnd: HWND, flag: DWORD) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
            hWnd: hwnd,
            uID: 0x1 as UINT,
            uFlags: flag,
            uCallbackMessage: 0 as UINT,
            hIcon: 0 as HICON,
            szTip: [0 as u16; 128],
            dwState: 0 as DWORD,
            dwStateMask: 0 as DWORD,
            szInfo: [0 as u16; 256],
            u: Default::default(),
            szInfoTitle: [0 as u16; 64],
            dwInfoFlags: 0 as UINT,
            guidItem: GUID {
                Data1: 0 as c_ulong,
                Data2: 0 as c_ushort,
                Data3: 0 as c_ushort,
                Data4: [0; 8],
            },
            hBalloonIcon: 0 as HICON,
        }
    }
}

struct MII {}

impl MII {
    fn mii(mask: UINT, typ: UINT, index: u32, name: *mut u16, size: u32) -> MENUITEMINFOW {
        MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as UINT,
            fMask: mask,
            fType: typ,
            fState: 0 as UINT,
            wID: index,
            hSubMenu: 0 as HMENU,
            hbmpChecked: 0 as HBITMAP,
            hbmpUnchecked: 0 as HBITMAP,
            dwItemData: 0 as ULONG_PTR,
            dwTypeData: name,
            cch: size,
            hbmpItem: 0 as HBITMAP,
        }
    }

}

#[derive(Clone)]
struct WindowInfo {
    pub hwnd: HWND,
    pub hinstance: HINSTANCE,
    pub hmenu: HMENU,
}

impl WindowInfo {
    unsafe fn new() -> Result<WindowInfo, Error> {
        let class_name = WS::to_string("my_window");
        let hinstance: HINSTANCE = libloaderapi::GetModuleHandleA(std::ptr::null_mut());
        let wnd = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(WindowInfo::window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: winuser::LoadIconW(0 as HINSTANCE, winuser::IDI_APPLICATION),
            hCursor: winuser::LoadCursorW(0 as HINSTANCE, winuser::IDI_APPLICATION),
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: class_name.as_ptr(),
        };
        if winuser::RegisterClassW(&wnd) == 0 {
            return Err(Error::OsError(format!("Error creating window class: {}", errhandlingapi::GetLastError())));
        }
        let hwnd = winuser::CreateWindowExW(
            0,
            class_name.as_ptr(),
            WS::to_string("rust_systray_window").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            0,
            CW_USEDEFAULT,
            0,
            0 as HWND,
            0 as HMENU,
            0 as HINSTANCE,
            std::ptr::null_mut(),
        );
        if hwnd == std::ptr::null_mut() {
            return Err(Error::OsError(format!("Error creating window: {}", errhandlingapi::GetLastError())));
        }

        // Setup menu
        let hmenu = winuser::CreatePopupMenu();
        let m = MENUINFO {
            cbSize: std::mem::size_of::<MENUINFO>() as DWORD,
            fMask: MIM_APPLYTOSUBMENUS | MIM_STYLE,
            dwStyle: MNS_NOTIFYBYPOS,
            cyMax: 0 as UINT,
            hbrBack: 0 as HBRUSH,
            dwContextHelpID: 0 as DWORD,
            dwMenuData: 0 as ULONG_PTR,
        };
        if winuser::SetMenuInfo(hmenu, &m as *const MENUINFO) == 0 {
            return Err(Error::OsError(format!("Error setting up menu: {}", errhandlingapi::GetLastError())));
        }

        let mut nid = NID::nid(hwnd, NIF_MESSAGE);
        nid.uID = 0x1;
        nid.uCallbackMessage = WM_USER;
        if shellapi::Shell_NotifyIconW(NIM_ADD, &mut nid as *mut NOTIFYICONDATAW) == 0 {
            return Err(Error::OsError(format!("Error adding menu icon: {}", errhandlingapi::GetLastError())));
        }
    
        Ok(WindowInfo {
            hwnd: hwnd,
            hmenu: hmenu,
            hinstance: hinstance,
        })
    }

    unsafe extern "system" fn window_proc(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        match msg {
            WM_MENUCOMMAND => {
                WININFO_STASH.with(|stash| {
                    if let Some(stash) = stash.borrow().as_ref() {
                        let id = winuser::GetMenuItemID(stash.info.hmenu, w_param as i32) as i32;
                        if id != -1 {
                            stash.tx.send(TrayEvent::from(id)).ok();
                        }
                    }
                });
            },
            WM_USER => {
                if l_param as UINT == WM_LBUTTONUP || l_param as UINT == WM_RBUTTONUP {
                    let mut p = POINT { x: 0, y: 0 };
                    if winuser::GetCursorPos(&mut p as *mut POINT) == FALSE {
                        return 1
                    }
                    winuser::SetForegroundWindow(h_wnd);
                    WININFO_STASH.with(|stash| {
                        if let Some(stash) = stash.borrow().as_ref() {
                            winuser::TrackPopupMenu(
                                stash.info.hmenu,
                                0,
                                p.x,
                                p.y,
                                (winuser::TPM_BOTTOMALIGN | winuser::TPM_LEFTALIGN) as i32,
                                h_wnd,
                                std::ptr::null_mut(),
                            );
                        }
                    });
                }
            },
            WM_DESTROY => winuser::PostQuitMessage(0),
            _ => (),
        }
    
        winuser::DefWindowProcW(h_wnd, msg, w_param, l_param)
    }
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

#[derive(Clone)]
struct WindowsData {
    pub info: WindowInfo,
    pub tx: Sender<TrayEvent>,
}

thread_local!(static WININFO_STASH: RefCell<Option<WindowsData>> = RefCell::new(None));

struct Window {
    info: WindowInfo,
    windows_loop: Option<thread::JoinHandle<()>>,
}

impl Window {
    fn new(event_tx: Sender<TrayEvent>) -> Result<Window, Error> {
        let (tx, rx) = channel();
        let windows_loop = thread::spawn(move || {
            unsafe {
                let info = match WindowInfo::new() {
                    Ok(i) => {
                        tx.send(Ok(i.clone())).ok();
                        i
                    },
                    Err(e) => {
                        // If creation didn't work, return out of the thread.
                        tx.send(Err(e)).ok();
                        return;
                    },
                };

                WININFO_STASH.with(|stash| {
                    let data = WindowsData {
                        info,
                        tx: event_tx,
                    };
                    (*stash.borrow_mut()) = Some(data);
                });

                let mut msg = winuser::MSG {
                    hwnd: 0 as HWND,
                    message: 0 as UINT,
                    wParam: 0 as WPARAM,
                    lParam: 0 as LPARAM,
                    time: 0 as DWORD,
                    pt: POINT { x: 0, y: 0 },
                };

                loop {
                    winuser::GetMessageW(&mut msg, 0 as HWND, 0, 0);
                    if msg.message == winuser::WM_QUIT {
                        break;
                    }
                    winuser::TranslateMessage(&mut msg);
                    winuser::DispatchMessageW(&mut msg);
                }
            }
        });

        match rx.recv().unwrap() {
            Ok(info) => {
                return Ok(Window {
                    info,
                    windows_loop: Some(windows_loop),
                })
            },
            Err(e) => {
                return Err(e);
            }
        };
    }

    fn quit(&mut self) {
        unsafe {
            winuser::PostMessageW(self.info.hwnd, WM_DESTROY, 0 as WPARAM, 0 as LPARAM);
        }
        if let Some(t) = self.windows_loop.take() {
            t.join().ok();
        }
    }

    fn set_tooltip(&self, tooltip: &str) -> Result<(), Error> {
        if tooltip.len() > 128 {
            return Err(Error::OsError(format!("Error tooltip is larger then 128 bytes")));
        }       
        let mut nid = NID::nid(self.info.hwnd, NIF_TIP);
        for (i, v) in tooltip.as_bytes().iter().map(|x| *x as u16).enumerate() {
            nid.szTip[i] = v;
        }
        unsafe {
            if shellapi::Shell_NotifyIconW(NIM_MODIFY, &mut nid) == 0 {
                return Err(Error::OsError(format!("Error setting tooltip: {}", errhandlingapi::GetLastError())));
            }
        }
        Ok(())
    }

    fn add_menu_entry(&self, item_idx: u32, item_name: &str) -> Result<(), Error> {
        let mut st = WS::to_string(item_name);
        let item = MII::mii(
            MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE,
            MFT_STRING,
            item_idx,
            st.as_mut_ptr(),
            (st.len() * 2) as u32,
        );
        unsafe {
            if winuser::InsertMenuItemW(self.info.hmenu, item_idx, 1, &item as *const MENUITEMINFOW) == 0 {
                return Err(Error::OsError(format!("Error inserting menu item: {}", errhandlingapi::GetLastError())));
            }
        }
        Ok(())
    }

    fn add_menu_separator(&self, item_idx: u32) -> Result<(), Error> {
        let item = MII::mii(
            MIIM_FTYPE,
            MFT_SEPARATOR,
            item_idx,
            std::ptr::null_mut(),
            0 as u32,
        );
        unsafe {
            if winuser::InsertMenuItemW(self.info.hmenu, item_idx, 1, &item as *const MENUITEMINFOW) == 0 {
                return Err(Error::OsError(format!("Error inserting separator: {}", errhandlingapi::GetLastError())));
            }
        }
        Ok(())
    }

    fn set_icon(&self, icon_file: &str) -> Result<(), Error> {
        unsafe {
            let hicon = winuser::LoadImageW(
                std::ptr::null_mut() as HINSTANCE,
                WS::to_string(&icon_file).as_ptr(),
                IMAGE_ICON,
                128,
                128,
                LR_LOADFROMFILE,
            ) as HICON;

            if hicon == std::ptr::null_mut() as HICON {
                return Err(Error::OsError(format!("Error setting icon from file: {}", errhandlingapi::GetLastError())));
            }

            let mut nid = NID::nid(self.info.hwnd, NIF_ICON);
            nid.hIcon = hicon;

            if shellapi::Shell_NotifyIconW(NIM_MODIFY, &mut nid as *mut NOTIFYICONDATAW) == 0 {
                return Err(Error::OsError(format!("Error setting icon: {}", errhandlingapi::GetLastError())));
            }
        }

        Ok(())
    }

    fn shutdown(&self) -> Result<(), Error> {
        unsafe {
            let mut nid = NID::nid(self.info.hwnd, NIF_ICON);
            if shellapi::Shell_NotifyIconW(NIM_DELETE, &mut nid as *mut NOTIFYICONDATAW) == 0 {
                return Err(Error::OsError(format!("Error deleting icon from menu: {}", errhandlingapi::GetLastError())));
            }
        }
        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}

use std::{
    collections::HashMap,
    error, fmt,
    sync::mpsc::Receiver,
};

type BoxedError = Box<dyn error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    OsError(String),
    NotImplementedError,
    UnknownError,
    Error(BoxedError),
    TryRecvError(TryRecvError),
}

impl From<BoxedError> for Error {
    fn from(value: BoxedError) -> Self {
        Error::Error(value)
    }
}

pub struct TrayEvent {
    id: u32,
}

impl TrayEvent {
    pub fn new(id: u32) -> TrayEvent {
        TrayEvent {
            id
        }
    }
}

impl From<i32> for TrayEvent {
    fn from(v: i32) -> TrayEvent {
        TrayEvent {id: v as u32}
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Error::*;

        match *self {
            OsError(ref err_str) => write!(f, "OsError: {}", err_str),
            NotImplementedError => write!(f, "Functionality is not implemented yet"),
            UnknownError => write!(f, "Unknown error occurred"),
            Error(ref e) => write!(f, "Error: {}", e),
            TryRecvError(ref e) => write!(f, "Error: {}", e),
        }
    }
}

type Callback =
    Box<(dyn FnMut(&mut Application) -> Result<(), BoxedError> + Send + Sync + 'static)>;

fn make_callback<F, E>(mut f: F) -> Callback
where
    F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
    E: error::Error + Send + Sync + 'static,
{
    Box::new(move |a: &mut Application| match f(a) {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e) as BoxedError),
    }) as Callback
}


pub struct Application {
    window: Window,
    menu_idx: u32,
    callback: HashMap<u32, Callback>,   
    rx: Receiver<TrayEvent>,
}

impl Application {
    pub fn new() -> Result<Application, Error> {
        let (event_tx, event_rx) = channel();
        match Window::new(event_tx) {
            Ok(w) => Ok(Application {
                window: w,
                menu_idx: 0,
                callback: HashMap::new(),
                rx: event_rx,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn add_menu_item<F, E>(&mut self, item_name: &str, f: F) -> Result<u32, Error>
    where
        F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
        E: error::Error + Send + Sync + 'static,
    {
        let idx = self.menu_idx;
        if let Err(e) = self.window.add_menu_entry(idx, item_name) {
            return Err(e);
        }
        self.callback.insert(idx, make_callback(f));
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn add_menu_separator(&mut self) -> Result<u32, Error> {
        let idx = self.menu_idx;
        if let Err(e) = self.window.add_menu_separator(idx) {
            return Err(e);
        }
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn set_icon(&self, file: &str) -> Result<(), Error> {
        self.window.set_icon(file)
    }

    pub fn shutdown(&self) -> Result<(), Error> {
        self.window.shutdown()
    }

    pub fn set_tooltip(&self, tooltip: &str) -> Result<(), Error> {
        self.window.set_tooltip(tooltip)
    }

    pub fn quit(&mut self) {
        self.window.quit()
    }

    pub fn update(&mut self) -> Result<(), Error> {
        match self.rx.try_recv() {
            Ok(m) => {
                if self.callback.contains_key(&m.id) {
                    if let Some(mut f) = self.callback.remove(&m.id) {
                        f(self)?;
                        self.callback.insert(m.id, f);
                    }
                }
                Ok(())
            },
            Err(TryRecvError::Empty) => Ok(()),
            Err(_) => {
                self.quit();
                Ok(())
            }
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
