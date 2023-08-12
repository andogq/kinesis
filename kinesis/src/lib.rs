mod component;
mod dom;
mod fragment;
mod util;

mod counter;
mod simple;

use component::ControllerRef;
use fragment::{Fragment, Node};

use simple::Simple;
use wasm_bindgen::prelude::*;
use web_sys::window;

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
    let mut context = Ctx { count: 3 };

    let mut fragment = Fragment::<Ctx>::build()
        .with_piece(Node::element("p"), None)
        .with_piece(Node::text("some content: "), Some(0))
        .with_updatable(&[0], Some(0), |ctx: &Ctx| ctx.count.to_string())
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
        .build(&document);

    let body = Location::parent(&body);

    // Mount test component
    fragment.mount(&body);
    fragment.full_update(&context);

    // Update state
    context.count = 5;
    fragment.update(&context, &[0]);

    // Detach from DOM
    fragment.detach(true);

    // Update while detached
    context.count = 11;
    fragment.update(&context, &[0]);

    // Mount to DOM
    fragment.mount(&body);
    fragment.full_update(&context);

    context.count = 10;
    fragment.update(&context, &[0]);

    Ok(())
}
