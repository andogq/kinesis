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
use web_sys::{console, window};

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
    let context = Ctx { count: 3 };

    let mut fragment = Fragment::new(&document, || &context)
        .with_piece(Piece::new(Kind::Element(ElementKind::P), Location::Target))
        .with_piece(Piece::new(
            Kind::Text("some content: ".into()),
            Location::Append(NodeOrReference::Reference(0)),
        ))
        .with_updatable(&[0], |ctx| {
            Piece::new(
                Kind::Text(ctx.count.to_string()),
                Location::Append(NodeOrReference::Reference(0)),
            )
        });

    fragment.mount(&body.into(), None);

    Ok(())
}
