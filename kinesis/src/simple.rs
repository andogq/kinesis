use std::{cell::RefCell, rc::Rc};

use crate::{
    component::{Component, ComponentWrapper},
    controller::{Controller, ControllerRef},
    fragment::{Fragment, FragmentBuilder, Node},
};
use web_sys::{console, Event};

pub struct BoldCount(usize);
impl BoldCount {
    pub fn new() -> ComponentWrapper<Self> {
        let component = Rc::new(RefCell::new(Self(0)));

        let fragment = Fragment::build()
            .with_element("p", None)
            .with_text("Bolded this text: ", Some(0))
            .with_element("b", Some(0))
            .with_updatable(&[0], Some(2), {
                console::log_1(&"updating bold".into());

                let component = Rc::clone(&component);
                move || {
                    let component = component.borrow();
                    Fragment::build().with_text(component.0.to_string(), None)
                }
            })
            .with_node(Node::element("button").with_event("click", 0), None)
            .with_text("Clear", Some(3));

        ComponentWrapper::new(component, fragment)
    }
}
impl Component for BoldCount {
    fn handle_event(&mut self, event_id: usize, _event: Event) -> Option<Vec<usize>> {
        match event_id {
            0 => {
                self.0 = 0;

                Some(vec![0])
            }
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Simple {
    count: usize,
}

impl Simple {
    pub fn new(controller_ref: &ControllerRef<Self>) -> ComponentWrapper<Self> {
        let component_ref = Rc::new(RefCell::new(Self { count: 0 }));
        let controller_ref = controller_ref.clone();

        let bold_text = BoldCount::new();
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
            .with_component(
                &[0],
                None,
                bold_text,
                {
                    let component = Rc::clone(&component_ref);
                    let bold_text = Rc::clone(&bold_text_ref);

                    move |changed| {
                        let component = component.borrow();
                        let mut bold_text = bold_text.borrow_mut();

                        // Update the nested component when the parent's fields change
                        let changed = changed
                            .iter()
                            .filter_map(|changed| match changed {
                                0 => {
                                    console::log_1(&"sending data to bold".into());

                                    bold_text.0 = component.count;
                                    Some(vec![0])
                                }
                                _ => None,
                            })
                            .flatten()
                            .collect::<Vec<_>>();

                        if changed.is_empty() {
                            None
                        } else {
                            Some(changed)
                        }
                    }
                },
                Some({
                    let component = Rc::clone(&component_ref);
                    let bold_text = Rc::clone(&bold_text_ref);

                    move |changed: &[usize]| {
                        console::log_1(&"running bound update".into());

                        let changed = {
                            let mut component = component.borrow_mut();
                            let bold_text = bold_text.borrow();

                            changed
                                .iter()
                                .filter_map(|changed| match changed {
                                    0 => {
                                        console::log_1(&"Updating component from bold".into());
                                        component.count = bold_text.0;
                                        Some(vec![0])
                                    }
                                    _ => None,
                                })
                                .flatten()
                                .collect::<Vec<_>>()
                        };

                        // TODO: Somehow prevent this component updating twice. Currently, it
                        // updates the parent component, notifying the parent component that
                        // something changed. Since it is bound to the same thing, the sub
                        // component is then udpated again.
                        // Either, the sub-component should somehow be freed before it's attempted
                        // to be updated (need to work out where this is), or need to prevent the
                        // double update happening (feels like a bad idea).
                        controller_ref.notify_changed(&changed);

                        if changed.is_empty() {
                            None
                        } else {
                            Some(changed)
                        }
                    }
                }),
            );

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
