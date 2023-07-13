mod component;
mod dom;
mod scheduler;

mod counter;
mod simple;

use component::ComponentControllerRef;
use counter::Counter;

use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("body to exist");

    let c = ComponentControllerRef::new(Counter::new(), &document, body.into());

    c.render()?;

    Ok(())
}
