mod component;
mod dom;
mod util;

mod counter;
mod simple;

use component::controller_v2::ControllerRef;

use simple::Simple;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Configure the panic hook to log to console.error
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("body to exist");

    // let c = ComponentControllerRef::new(Counter::new(), &document, body.into());
    // c.render()?;

    let c = ControllerRef::new(Simple::default(), &document);
    c.render(&body.into())?;

    Ok(())
}
