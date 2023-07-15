use web_sys::Event;

use crate::{
    component::{Component, EventType},
    dom::{Content, DomNode},
};

pub struct Counter {
    count: isize,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Component for Counter {
    fn handle_event(
        &mut self,
        id: usize,
        event_type: EventType,
        _event: Event,
    ) -> Option<Vec<usize>> {
        match (id, event_type) {
            (2, EventType::Click) => {
                self.count -= 1;
                Some(vec![0])
            }
            (3, EventType::Click) => {
                self.count += 1;
                Some(vec![0])
            }
            _ => None,
        }
    }

    fn render(&self) -> Vec<DomNode> {
        vec![DomNode::div(0)
            .child(DomNode::p(1).text_content(Content::Dynamic {
                dependencies: vec![0],
                render: Box::new(|count| format!("The current count is {}", count)),
            }))
            .child(
                DomNode::button(2)
                    .text_content(Content::Static("Decrease".to_string()))
                    .listen(EventType::Click),
            )
            .child(
                DomNode::button(3)
                    .text_content(Content::Static("Increase".to_string()))
                    .listen(EventType::Click),
            )]
    }

    fn get_counter_temp(&self) -> isize {
        self.count
    }
}
