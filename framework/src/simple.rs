use web_sys::Event;

use crate::{
    component::{Component, Identifier, RenderType},
    dom::{renderable::Renderable, text::Text, DomNode, EventType},
};

pub struct Simple;

impl Component for Simple {
    fn handle_event(
        &mut self,
        id: Identifier,
        event_type: EventType,
        event: Event,
    ) -> Option<Vec<usize>> {
        None
    }

    fn render(&self, update_type: RenderType) -> Option<Vec<Box<dyn Renderable>>> {
        Some(vec![
            Box::new(DomNode::p().child(Box::new(Text::new("Simple component"))))
                as Box<dyn Renderable>,
        ])
    }
}
