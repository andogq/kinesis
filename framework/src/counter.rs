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
    fn handle_event(&mut self) {
        self.count += 1;
    }

    fn render(&self) -> Vec<DomNode> {
        vec![DomNode::div(0)
            .child(DomNode::p(1).text_content(&format!("The current count is {}", self.count)))
            .child(
                DomNode::button(2)
                    .text_content("Click me!")
                    .listen(EventType::Click),
            )]
    }
}
