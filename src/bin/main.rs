use std::env;
use env_logger;

use luna::app::App;

fn main () {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    App::new()
    .unwrap()
    .run()
    .unwrap();
}