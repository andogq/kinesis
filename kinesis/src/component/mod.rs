mod controller;
mod identifier;

use std::any::Any;

use web_sys::Event;

pub use self::controller::Controller;
pub use self::identifier::Identifier;
use crate::fragment::FragmentBuilder;

pub type AnyComponent = dyn Component<Ctx = dyn Any, Child = dyn Any>;

/// Trait that represents a renderable component
pub trait Component: HasContext {
    /// Type representing all possible children components. Should be an enum. Will be passed into
    /// an update function to update that particular child.
    type Child: Any + ?Sized;

    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>>;

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> FragmentBuilder<Self::Ctx>;
}

pub trait HasContext {
    /// Type of the context that will be used inside the component. More than likely will be
    /// `Self`.
    type Ctx: Any + ?Sized;

    /// Get the context for this component. For most instances this will return `&Self`.
    fn get_context(&self) -> &Self::Ctx;
}

impl<C> HasContext for C
where
    C: Component<Ctx = C> + 'static,
{
    type Ctx = C;

    fn get_context(&self) -> &Self::Ctx {
        self
    }
}
