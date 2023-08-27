mod component;
mod controller;
mod dynamic;
mod event_registry;
mod fragment;
mod util;

mod simple;

use controller::{Controller, ControllerRef};
use simple::Simple;
use wasm_bindgen::prelude::*;
use web_sys::window;

use crate::fragment::Location;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Configure the panic hook to log to console.error
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("body to exist");

    let controller_ref = ControllerRef::new();
    let component = Controller::<Simple>::new(&document, Simple::new(&controller_ref), None);
    controller_ref.replace_with(&component);
    component.borrow_mut().mount(&Location::parent(&body));

    Ok(())
}
