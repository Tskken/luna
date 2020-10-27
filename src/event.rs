use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::mpsc::{Sender, Receiver, channel};

use winapi::{
    shared::windef::HWND,
};

use crate::window::Window;
use crate::error::{Error, Result};

thread_local!(pub static EVENT_HANDLER: RefCell<Option<Handler>> = RefCell::new(None));

pub type Callback = Box<dyn Fn() -> Result<()> + Send + Sync + 'static>;

pub fn new_callback<F>(f: F) -> Callback 
where
    F: Fn() -> Result<()> + Send + Sync + 'static,
{
    Box::new(f)
}

pub struct Handler {
    pub events: HashMap<(HWND, u32), Window>,
    pub sender: Sender<HWND>,
}

impl Handler {
    pub fn new() -> (Handler, Receiver<HWND>) {
        let(sender, receiver) = channel();
        (Handler {
            events: HashMap::new(),
            sender,
        }, receiver)
    }

    pub fn insert<F>(&mut self, hwnd: HWND, item_index: u32, window: Window) -> Result<()> {  
        match self.events.insert((hwnd, item_index), window) {
            Some(_) => Err(Error::WindowFound((hwnd, item_index))),
            None => Ok(())
        }
    }

    pub fn remove(&mut self, hwnd: HWND, item_index: u32) -> Result<()> {
        match self.events.remove(&(hwnd, item_index)) {
            Some(_) => Ok(()),
            None => Err(Error::NoWindowFound((hwnd, item_index)))
        }
    }

    pub fn run(&mut self, hwnd: HWND, item_index: u32) -> Result<()> {
        match self.events.get_mut(&(hwnd, item_index)) {
            Some(w) => {
                if let Some(f) = &w.callback {
                    return f()
                }
                Ok(())
            },
            None => Err(Error::NoWindowFound((hwnd, item_index))),
        }
    }
}