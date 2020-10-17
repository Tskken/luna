use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::mpsc::{Sender, Receiver, channel};

use winapi::{
    shared::windef::HWND,
};

use crate::app::App;
use crate::error::{Error, Result};

thread_local!(pub static EVENT_HANDLER: RefCell<Option<Handler>> = RefCell::new(None));

pub type Callback = Box<dyn FnMut(&mut App) -> Result<()> + Send + Sync + 'static>;

pub fn new_callback<F>(mut f: F) -> Callback 
where
    F: FnMut(&mut App) -> Result<()> + Send + Sync + 'static,
{
    Box::new(
        move |a: &mut App| f(a)
    )
}

pub struct Handler {
    pub events: HashMap<(HWND, u32), Callback>,
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

    pub fn insert<F>(&mut self, hwnd: HWND, item_index: u32, f: F) -> Result<()>
    where 
        F: FnMut(&mut App) -> Result<()> + Send + Sync + 'static,
    {  
        match self.events.insert((hwnd, item_index), new_callback(f)) {
            Some(_) => Err(Error::CallbackFound((hwnd, item_index))),
            None => Ok(())
        }
    }

    pub fn remove(&mut self, hwnd: HWND, item_index: u32) -> Result<()> {
        match self.events.remove(&(hwnd, item_index)) {
            Some(_) => Ok(()),
            None => Err(Error::NoCallbackFound((hwnd, item_index)))
        }
    }

    pub fn run(&mut self, app: &mut App, hwnd: HWND, item_index: u32) -> Result<()> {
        match self.events.get_mut(&(hwnd, item_index)) {
            Some(f) => f(app),
            None => Err(Error::NoCallbackFound((hwnd, item_index))),
        }
    }
}