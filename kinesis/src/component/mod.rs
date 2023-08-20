mod wrapper;

use web_sys::Event;
pub use wrapper::ComponentWrapper;

/// Trait that represents a component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>>;
}
