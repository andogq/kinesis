use std::{cell::RefCell, rc::Rc};

use crate::{
    component::Component,
    fragment::{Fragment, FragmentBuilder, Node},
};
use web_sys::Event;

pub struct Nested;
impl Nested {
    pub fn new() -> (Rc<RefCell<Self>>, FragmentBuilder) {
        let component = Rc::new(RefCell::new(Self));

        let fragment = Fragment::build()
            .with_element("p", None)
            .with_text("nested", Some(0));

        (component, fragment)
    }
}
impl Component for Nested {
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>> {
        todo!()
    }
}

#[derive(Default)]
pub struct Simple {
    count: usize,
}

impl Simple {
    pub fn new() -> (Rc<RefCell<Self>>, FragmentBuilder) {
        let component = Rc::new(RefCell::new(Self { count: 0 }));

        let fragment = Fragment::build()
            .with_element("p", None)
            .with_text("some content: ", Some(0))
            .with_updatable(&[0], Some(0), {
                let ctx = Rc::clone(&component);
                move || {
                    let ctx = ctx.borrow();
                    Fragment::build().with_text(ctx.count.to_string(), None)
                }
            })
            .with_node(Node::element("button").with_event("click", 0), None)
            .with_text("decrement", Some(2))
            .with_node(Node::element("button").with_event("click", 1), None)
            .with_text("increment", Some(4))
            .with_conditional(
                &[0],
                None,
                {
                    let ctx = Rc::clone(&component);
                    move || {
                        let ctx = ctx.borrow();
                        ctx.count % 2 == 0
                    }
                },
                || {
                    Fragment::build()
                        .with_element("p", None)
                        .with_text("showing!", Some(0))
                },
            )
            .with_iter(&[0], None, {
                let ctx = Rc::clone(&component);
                move || {
                    let ctx = ctx.borrow();
                    Box::new((0..ctx.count).map(|val| {
                        Fragment::build()
                            .with_element("p", None)
                            .with_text(format!("counting {val}"), Some(0))
                    })) as Box<dyn Iterator<Item = FragmentBuilder>>
                }
            })
            .with_component(&[], None, Nested::new(), |_| {});

        (component, fragment)
    }
}

impl Component for Simple {
    fn handle_event(&mut self, event_id: usize, _event: Event) -> Option<Vec<usize>> {
        match event_id {
            0 => {
                self.count -= 1;
                Some(vec![0])
            }
            1 => {
                self.count += 1;
                Some(vec![0])
            }
            _ => None,
        }
    }
}
