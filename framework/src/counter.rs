use web_sys::Event;

use crate::{
    component::Component,
    dom::{DomNode, EventType, TextContent},
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
        id: usize,
        event_type: EventType,
        _event: Event,
    ) -> Option<Vec<usize>> {
        match (id, event_type) {
            (3, EventType::Click) => {
                self.count -= 1;
                Some(vec![0])
            }
            (4, EventType::Click) => {
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

    fn render(&self) -> Vec<DomNode<usize>> {
        vec![DomNode::div(0)
            .child(DomNode::p(1).text_content(TextContent::Dynamic {
                dependencies: vec![0],
                update_type: 0,
            }))
            .child(DomNode::p(2).text_content(TextContent::Dynamic {
                dependencies: vec![1],
                update_type: 1,
            }))
            .child(
                DomNode::button(3)
                    .text_content(TextContent::Static("Decrease".to_string()))
                    .listen(EventType::Click),
            )
            .child(
                DomNode::button(4)
                    .text_content(TextContent::Static("Increase".to_string()))
                    .listen(EventType::Click),
            )]
    }
}
