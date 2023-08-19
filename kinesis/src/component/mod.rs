mod controller;
mod identifier;

use std::any::Any;

use web_sys::Event;

pub use self::controller::Controller;
pub use self::identifier::Identifier;
use crate::fragment::FragmentBuilder;

pub type AnyComponent = dyn Component<Ctx = dyn Any>;

/// Trait that represents a renderable component
pub trait Component {
    /// Type of the context that will be used inside the component. More than likely will be
    /// `Self`.
    type Ctx: Any + ?Sized;

    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>>;

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> FragmentBuilder<Self::Ctx>;

    /// Get the context for this component. For most instances this will return `&Self`.
    fn get_context(&self) -> &Self::Ctx;
}
