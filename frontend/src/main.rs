extern crate sycamore;
extern crate gloo_net;
extern crate futures;
extern crate web_sys;

mod app;
mod front_structs;
mod table;
use app::App;

pub fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(App);
}
