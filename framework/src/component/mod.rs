mod controller;
pub use controller::ComponentControllerRef;

use crate::dom::DomNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self);

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> Vec<DomNode>;
}
