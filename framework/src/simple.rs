use crate::{
    component::Component,
    dom::{Content, DomNode},
};

pub struct Simple;
impl Component for Simple {
    fn handle_event(
        &mut self,
        _id: usize,
        _event_type: crate::component::EventType,
        _event: web_sys::Event,
    ) -> Option<Vec<usize>> {
        None
    }

    fn render(&self) -> Vec<crate::dom::DomNode> {
        vec![DomNode::p(0).text_content(Content::Static("test".to_string()))]
    }

    fn get_counter_temp(&self) -> isize {
        0
    }
}
