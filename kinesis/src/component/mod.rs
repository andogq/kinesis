mod controller;
mod identifier;

use web_sys::Event;

pub use self::controller::Controller;
pub use self::identifier::Identifier;
use crate::fragment::FragmentBuilder;

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>>;

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> FragmentBuilder<Self>
    where
        Self: Sized;
}
