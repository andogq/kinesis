use web_sys::Event;

use crate::{
    component::{Component, Identifier, RenderType},
    dom::{renderable::Renderable, text::Text, DomNode, EventType, TextContent},
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

    fn render(&self, render_type: RenderType) -> Option<Vec<Box<dyn Renderable>>> {
        match render_type {
            RenderType::Root => Some(vec![Box::new(
                DomNode::div()
                    // .child(Box::new(DomNode::p().text_content(TextContent::Dynamic {
                    //     dependencies: vec![0],
                    //     update_type: 0,
                    // })))
                    // .child(Box::new(DomNode::p().text_content(TextContent::Dynamic {
                    //     dependencies: vec![1],
                    //     update_type: 1,
                    // })))
                    .child(Box::new(
                        DomNode::button()
                            .child(Box::new(Text::new("Decrease")))
                            .listen(EventType::Click),
                    ))
                    .child(Box::new(
                        DomNode::button()
                            .child(Box::new(Text::new("Increase")))
                            .listen(EventType::Click),
                    ))
                    .child(Box::new(true.then(|| {
                        Box::new(DomNode::p().child(Box::new(Text::new("Temporary element"))))
                            as Box<dyn Renderable>
                    })))
                    .child(Box::new((0..5).map(|i| {
                        Box::new(
                            DomNode::p().child(Box::new(Text::new(format!("Array element {i}")))),
                        ) as Box<dyn Renderable>
                    }))),
            )]),
            RenderType::Partial(0) => Some(vec![Box::new(Text::new(format!(
                "The current count is {}",
                self.count
            )))]),
            RenderType::Partial(1) => Some(vec![Box::new(Text::new(format!(
                "Only up is {}",
                self.only_up
            )))]),
            _ => None,
        }
    }
}
