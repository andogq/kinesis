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

use crate::component::fragment::Updatable;

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

    let mut fragment = Fragment::new(&document)
        .with_piece(Piece::new(Kind::Element(ElementKind::P), Location::Target))
        .with_piece(Piece::new(
            Kind::Text("some content: ".into()),
            Location::Append(NodeOrReference::Reference(0)),
        ))
        .with_updatable_text(
            &[0],
            Updatable::new(
                Location::Append(NodeOrReference::Reference(0)),
                |ctx: &Ctx| ctx.count.to_string(),
            ),
        );

    let body = Node::from(body);

    // Mount test component
    fragment.mount(&context, &body, None);

    // Update state
    context.count = 5;
    fragment.update(&[0], &context);

    // Detach from DOM
    fragment.detach();

    // Update while detached
    context.count = 10;
    fragment.update(&[0], &context);

    // Mount to DOM
    fragment.mount(&context, &body, None);

    Ok(())
}