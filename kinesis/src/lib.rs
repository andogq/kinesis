mod component;
mod fragment;
mod util;

mod simple;

use std::{cell::RefCell, rc::Rc};

use component::Controller;
use fragment::{Fragment, Node};

use simple::Simple;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

use crate::fragment::{DomRenderable, FragmentBuilder, Location};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Configure the panic hook to log to console.error
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("body to exist");

    let component = Controller::new(&document, Simple::new());
    component.borrow_mut().mount(&Location::parent(&body));

    Ok(())
}
