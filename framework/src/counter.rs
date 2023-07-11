use crate::{
    component::{Component, EventListener},
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

    fn render(&self) -> Option<(DomNode, Vec<EventListener>)> {
        Some((
            DomNode::p().text_content(&format!("The current count is {}", self.count)),
            vec![EventListener::new(0, "click")],
        ))
    }
}
