mod component;
mod dom;
mod fragment;
mod util;

mod counter;
mod simple;

use std::{cell::RefCell, rc::Rc};

use component::ControllerRef;
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

    // let c = ComponentControllerRef::new(Counter::new(), &document, body.into());
    // c.render()?;

    let _c = ControllerRef::new(Simple::default(), &document);
    // c.render(&body.into())?;

    struct Ctx {
        count: usize,
    }
    let mut context = Rc::new(RefCell::new(Ctx { count: 0 }));

    let mut container: Rc<RefCell<Option<Fragment<Ctx>>>> = Rc::new(RefCell::new(None));

    let mut fragment = Fragment::<Ctx>::build()
        .with_piece(Node::element("p"), None)
        .with_piece(Node::text("some content: "), Some(0))
        .with_updatable(&[0], Some(0), |ctx: &Ctx| ctx.count.to_string())
        .with_piece(Node::element("button").with_event("click", 0), None)
        .with_piece(Node::text("click me"), Some(2))
        .with_conditional(
            &[0],
            None,
            Fragment::build()
                .with_piece(Node::element("p"), None)
                .with_piece(Node::text("showing!"), Some(0)),
            |ctx| ctx.count % 2 == 0,
        )
        .with_each(&[0], None, |ctx| {
            Box::new((0..ctx.count).map(|val| {
                Fragment::build()
                    .with_piece(Node::element("p"), None)
                    .with_piece(Node::text(format!("counting {val}")), Some(0))
            })) as Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>
        })
        .build(&document, {
            let context = Rc::clone(&context);
            let container = Rc::clone(&container);

            Rc::new(move |_| {
                console::log_1(&"here".into());

                // Mutate state
                let mut context = context.borrow_mut();
                context.count += 1;

                // Trigger update
                if let Some(fragment) = container.borrow_mut().as_mut() {
                    fragment.update(&context, &[0]);
                }
            })
        });

    {
        // Perform inital mount
        let context = context.borrow();

        let body = Location::parent(&body);

        // Mount test component
        fragment.mount(&body);
        fragment.full_update(&context);
    }

    // Move fragment into container
    *container.borrow_mut() = Some(fragment);

    Ok(())
}
