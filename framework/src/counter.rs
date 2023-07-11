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

    fn render(&self) -> Option<(Vec<DomNode>, Vec<EventListener>)> {
        Some((
            vec![
                DomNode::p().text_content(&format!("The current count is {}", self.count)),
                DomNode::button().text_content("Click me!"),
            ],
            vec![EventListener::new(1, "click")],
        ))
    }
}
