mod counter;
mod dom;
mod scheduler;

use counter::Counter;
use scheduler::Scheduler;

use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{window, MouseEvent};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have body");

    let scheduler_rc = Rc::new(RefCell::new(Scheduler::new(document)));
    let mut scheduler = scheduler_rc.borrow_mut();

    let counter = Counter::default();
    let id = scheduler.add_component(counter, {
        let scheduler_rc = Rc::clone(&scheduler_rc);

        Box::new(move |document, counter, replace_with| {
            if let Some(el) = counter.render() {
                let el = el.build(document).unwrap();

                let scheduler = Rc::clone(&scheduler_rc);
                let callback = Closure::<dyn Fn(_)>::new(move |_event: MouseEvent| {
                    scheduler.borrow_mut().add_event(0);
                });

                el.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref());
                callback.forget();

                if let Some(target) = replace_with {
                    target.replace_with_with_node_1(&el);
                } else {
                    body.append_child(&el).unwrap();
                }

                return el;
            }

            unreachable!()
        })
    });

    // Jump start initial render
    scheduler.add_event(0);

    Ok(())
}
