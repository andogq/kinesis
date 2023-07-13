use crate::{component::Component, dom::DomNode};

pub struct Simple;
impl Component for Simple {
    fn handle_event(
        &mut self,
        id: usize,
        event_type: crate::component::EventType,
        event: web_sys::Event,
    ) {
        ()
    }

    fn render(&self) -> Vec<crate::dom::DomNode> {
        vec![DomNode::p(0).text_content("test")]
    }
}
