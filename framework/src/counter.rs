use web_sys::Event;

use crate::{
    component::{Component, Identifier},
    dom::{renderable::Renderable, DomNode, EventType, TextContent},
};

pub struct Counter {
    count: isize,
    only_up: usize,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            count: 0,
            only_up: 0,
        }
    }
}

impl Component for Counter {
    fn handle_event(
        &mut self,
        id: Identifier,
        event_type: EventType,
        _event: Event,
    ) -> Option<Vec<usize>> {
        match (id.as_ref(), event_type) {
            (&[0, 2], EventType::Click) => {
                self.count -= 1;
                Some(vec![0])
            }
            (&[0, 3], EventType::Click) => {
                self.count += 1;
                self.only_up += 1;
                Some(vec![0, 1])
            }
            _ => None,
        }
    }

    fn handle_update(&self, update_type: usize) -> Option<String> {
        match update_type {
            0 => Some(format!("The current count is {}", self.count)),
            1 => Some(format!("Only up is {}", self.only_up)),
            _ => None,
        }
    }

    fn render(&self) -> Vec<Box<dyn Renderable>> {
        vec![Box::new(
            DomNode::div()
                .child(Box::new(DomNode::p().text_content(TextContent::Dynamic {
                    dependencies: vec![0],
                    update_type: 0,
                })))
                .child(Box::new(DomNode::p().text_content(TextContent::Dynamic {
                    dependencies: vec![1],
                    update_type: 1,
                })))
                .child(Box::new(
                    DomNode::button()
                        .text_content(TextContent::Static("Decrease".to_string()))
                        .listen(EventType::Click),
                ))
                .child(Box::new(
                    DomNode::button()
                        .text_content(TextContent::Static("Increase".to_string()))
                        .listen(EventType::Click),
                ))
                .child(Box::new(true.then(|| {
                    Box::new(
                        DomNode::p()
                            .text_content(TextContent::Static("Temporary element".to_string())),
                    ) as Box<dyn Renderable>
                })))
                .child(Box::new((0..5).map(|i| {
                    Box::new(
                        DomNode::p()
                            .text_content(TextContent::Static(format!("Array element {i}"))),
                    ) as Box<dyn Renderable>
                })))
                .child(Box::new(
                    DomNode::button().text_content(TextContent::Static("Click".to_string())),
                )),
        )]
    }
}
