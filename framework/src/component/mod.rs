mod controller;
mod identifier;

pub use controller::*;
use web_sys::Event;

pub use self::identifier::Identifier;
use crate::dom::{renderable::Renderable, EventType};

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(
        &mut self,
        id: Identifier,
        event_type: EventType,
        event: Event,
    ) -> Option<Vec<usize>>;

    fn handle_update(&self, update_type: usize) -> Option<String>;

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> Vec<Box<dyn Renderable>>;
}
