mod component;
mod counter;
mod dom;
mod scheduler;

use counter::Counter;
use scheduler::Scheduler;

use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let scheduler_rc = Scheduler::new_with_ref(document);

    let mut scheduler = scheduler_rc.borrow_mut();
    scheduler.add_component(Box::new(Counter::new()));

    scheduler.run();

    Ok(())
}
