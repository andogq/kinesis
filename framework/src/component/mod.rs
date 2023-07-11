use crate::dom::DomNode;

pub struct EventListener {
    pub element_path: Vec<usize>,
    pub event_type: String,
}
impl EventListener {
    pub fn new(el_id: usize, event_type: &str) -> Self {
        Self {
            element_path: vec![el_id],
            event_type: event_type.to_string(),
        }
    }
}

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self);

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> Option<(DomNode, Vec<EventListener>)>;
}
