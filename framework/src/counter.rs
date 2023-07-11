use crate::dom::DomNode;

use web_sys::console;

#[derive(Default)]
pub struct Counter {
    count: usize,
}
impl Counter {
    pub fn handle_event(&mut self) {
        self.count += 1;
    }

    pub fn render(&self) -> Option<DomNode> {
        console::log_1(&format!("rendering {}", self.count).into());
        Some(DomNode::p().text_content(&format!("The current count is {}", self.count)))
    }
}
