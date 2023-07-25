use web_sys::Event;

use crate::{
    component::{Component, Identifier, RenderType},
    dom::{dynamic::Dynamic, renderable::Renderable, text::Text, DomNode, EventType},
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
        match update_type {
            RenderType::Root => Some(vec![
                Box::new(DomNode::p().child(Box::new(Text::new("Simple component"))))
                    as Box<dyn Renderable>,
                Box::new(DomNode::p().child(Box::new(Dynamic::new(0)))),
            ]),
            RenderType::Partial(0) => Some(vec![
                Box::new(Text::new("Dynamic text")) as Box<dyn Renderable>
            ]),
            _ => None,
        }
    }
}
