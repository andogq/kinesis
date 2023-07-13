mod controller;
pub use controller::ComponentControllerRef;

use std::collections::VecDeque;

use crate::dom::DomNode;

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Click,
}
impl From<EventType> for String {
    fn from(event: EventType) -> Self {
        use EventType::*;
        match event {
            Click => "click",
        }
        .to_string()
    }
}

pub struct EventListener {
    pub element_path: VecDeque<usize>,
    pub event_type: EventType,
}
impl EventListener {
    pub fn new(el_id: usize, event_type: EventType) -> Self {
        Self {
            element_path: VecDeque::from_iter([el_id]),
            event_type,
        }
    }

    pub fn nested_in(mut self, el_id: usize) -> Self {
        self.element_path.push_front(el_id);
        self
    }
}

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self);

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> Option<(Vec<DomNode>, Vec<EventListener>)>;
}
