use web_sys::Event;

use crate::{
    component::{Component, EventType},
    dom::DomNode,
};

pub struct Counter {
    count: usize,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Component for Counter {
    fn handle_event(&mut self, id: usize, event_type: EventType, _event: Event) {
        match (id, event_type) {
            (2, EventType::Click) => self.count -= 1,
            (3, EventType::Click) => self.count += 1,
            _ => (),
        }
    }

    fn render(&self) -> Vec<DomNode> {
        vec![DomNode::div(0)
            .child(DomNode::p(1).text_content(&format!("The current count is {}", self.count)))
            .child(
                DomNode::button(2)
                    .text_content("Decrease")
                    .listen(EventType::Click),
            )
            .child(
                DomNode::button(3)
                    .text_content("Increase")
                    .listen(EventType::Click),
            )]
    }
}
