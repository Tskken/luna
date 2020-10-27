#![allow(unused_imports)]

pub mod app;
pub mod window;
pub mod tray;
pub mod tray_old;
pub mod event;
pub mod geometry;
pub mod util;
pub mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
