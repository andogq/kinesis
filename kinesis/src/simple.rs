use web_sys::{console, Event};

use crate::{
    component::{Component, Identifier, RenderType},
    dom::{dynamic::Dynamic, renderable::Renderable, text::Text, DomNode, EventType},
};

#[derive(Default)]
pub struct Simple {
    count: usize,
}

impl Component for Simple {
    fn handle_event(
        &mut self,
        id: Identifier,
        event_type: EventType,
        event: Event,
    ) -> Option<Vec<usize>> {
        // WARN: Bad matching of event here, just for testing
        self.count += 1;
        Some(vec![0])
    }

    fn render(&self, update_type: RenderType) -> Option<Vec<Box<dyn Renderable>>> {
        match update_type {
            RenderType::Root => Some(vec![
                Box::new(DomNode::p().child(Box::new(Text::new("Simple component"))))
                    as Box<dyn Renderable>,
                Box::new(
                    DomNode::p()
                        .child(Box::new(Dynamic::new(0).depends_on(0)))
                        .listen(EventType::Click),
                ),
                Box::new(Dynamic::new(1).depends_on(0)),
            ]),
            RenderType::Partial(0) => {
                Some(vec![
                    Box::new(Text::new(format!("Dynamic text: {}", self.count)))
                        as Box<dyn Renderable>,
                ])
            }
            RenderType::Partial(1) => {
                console::log_1(&format!("rendering, {}", self.count % 2 == 0).into());

                Some(vec![Box::new((self.count % 2 == 0).then(|| {
                    Box::new(
                        DomNode::h1().child(Box::new(Text::new("Even")) as Box<dyn Renderable>),
                    ) as Box<dyn Renderable>
                })) as Box<dyn Renderable>])
            }
            _ => None,
        }
    }
}
