mod component;
mod dom;
mod util;

mod counter;
mod simple;

use component::{
    fragment::{ElementKind, Fragment, Kind, Location, NodeOrReference, Piece},
    ControllerRef,
};

use simple::Simple;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Node};

use crate::component::fragment::{Conditional, FragmentBuilder, Updatable};

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
    // c.render(&body.into())?;

    struct Ctx {
        count: usize,
    }
    let mut context = Ctx { count: 3 };

    let mut fragment = Fragment::build()
        .with_piece(Kind::Element(ElementKind::P), None)
        .with_piece(
            Kind::Text("some content: ".into()),
            Some(Location::append(NodeOrReference::Reference(0))),
        )
        .with_updatable(
            &[0],
            Some(Location::append(NodeOrReference::Reference(0))),
            |ctx: &Ctx| ctx.count.to_string(),
        )
        .with_conditional(
            &[0],
            None,
            Fragment::build()
                .with_piece(Kind::Element(ElementKind::P), None)
                .with_piece(
                    Kind::Text("showing!".into()),
                    Some(Location::append(NodeOrReference::Reference(0))),
                ),
            |ctx| ctx.count % 2 == 0,
        )
        .with_each(&[0], None, |ctx| {
            Box::new((0..ctx.count).map(|val| {
                Fragment::build()
                    .with_piece(Kind::Element(ElementKind::P), None)
                    .with_piece(
                        Kind::Text(format!("counting {val}")),
                        Some(Location::append(NodeOrReference::Reference(0))),
                    )
            })) as Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>
        })
        .build(&document);

    let body = Node::from(body);

    // Mount test component
    fragment.mount(&context, &body, None);

    // Update state
    context.count = 5;
    fragment.update(&[0], &context);

    // Detach from DOM
    fragment.detach();

    // Update while detached
    context.count = 11;
    fragment.update(&[0], &context);

    // Mount to DOM
    fragment.mount(&context, &body, None);

    context.count = 10;
    fragment.update(&[0], &context);

    Ok(())
}
