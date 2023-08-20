use std::{cell::RefCell, rc::Rc};

use crate::{
    component::{Component, ComponentWrapper},
    fragment::{Fragment, FragmentBuilder, Node},
};
use web_sys::Event;

pub struct BoldText(String);
impl BoldText {
    pub fn new() -> ComponentWrapper<Self> {
        let component = Rc::new(RefCell::new(Self(String::new())));

        let fragment = Fragment::build()
            .with_element("p", None)
            .with_text("Bolded this text: ", Some(0))
            .with_element("b", Some(0))
            .with_updatable(&[0], Some(2), {
                let component = Rc::clone(&component);
                move || {
                    let component = component.borrow();
                    Fragment::build().with_text(component.0.clone(), None)
                }
            });

        ComponentWrapper::new(component, fragment)
    }
}
impl Component for BoldText {
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>> {
        None
    }
}

#[derive(Default)]
pub struct Simple {
    count: usize,
}

impl Simple {
    pub fn new() -> ComponentWrapper<Self> {
        let component_ref = Rc::new(RefCell::new(Self { count: 0 }));

        let bold_text = BoldText::new();
        let bold_text_ref = bold_text.clone_component();

        let fragment = Fragment::build()
            .with_element("p", None)
            .with_text("some content: ", Some(0))
            .with_updatable(&[0], Some(0), {
                let ctx = Rc::clone(&component_ref);
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
                    let ctx = Rc::clone(&component_ref);
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
                let ctx = Rc::clone(&component_ref);
                move || {
                    let ctx = ctx.borrow();
                    Box::new((0..ctx.count).map(|val| {
                        Fragment::build()
                            .with_element("p", None)
                            .with_text(format!("counting {val}"), Some(0))
                    })) as Box<dyn Iterator<Item = FragmentBuilder>>
                }
            })
            .with_component(&[0], None, bold_text, {
                let component = Rc::clone(&component_ref);
                let bold_text = bold_text_ref;

                move |changed| {
                    let component = component.borrow();
                    let mut bold_text = bold_text.borrow_mut();

                    changed.iter().for_each(|changed| match changed {
                        0 => bold_text.0 = component.count.to_string(),
                        _ => (),
                    });

                    changed.to_vec()
                }
            });

        ComponentWrapper::new(component_ref, fragment)
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
