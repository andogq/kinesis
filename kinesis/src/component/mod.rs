mod controller;
mod identifier;

use std::cell::RefCell;
use std::rc::Rc;

use web_sys::Event;

use crate::fragment::FragmentBuilder;

pub use self::controller::Controller;
pub use self::identifier::Identifier;

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self, event_id: usize, event: Event) -> Option<Vec<usize>>;
}

/// Helper type to easily pass a constructed component around.
pub struct ComponentWrapper<C: ?Sized + Component> {
    /// A shared reference to the component.
    pub component: Rc<RefCell<C>>,

    /// The builder to construct the nodes for the component.
    pub fragment_builder: FragmentBuilder,
}

impl<C: ?Sized + Component> ComponentWrapper<C> {
    /// Construct a new component wrapper
    pub fn new(component: Rc<RefCell<C>>, fragment_builder: FragmentBuilder) -> Self {
        Self {
            component,
            fragment_builder,
        }
    }

    /// Clone the reference to the component
    pub fn clone_component(&self) -> Rc<RefCell<C>> {
        Rc::clone(&self.component)
    }
}

impl<C: Component + 'static> ComponentWrapper<C> {
    pub fn into_any(self) -> ComponentWrapper<dyn Component> {
        ComponentWrapper::<dyn Component> {
            component: self.component as Rc<RefCell<dyn Component>>,
            fragment_builder: self.fragment_builder,
        }
    }
}
