mod r#ref;

pub use self::r#ref::ControllerRef;
use crate::component::{Component, ComponentWrapper};
use crate::dynamic::{Dynamic, UpdateFn};
use crate::event_registry::EventRegistry;
use crate::fragment::{Fragment, Location};

use std::{cell::RefCell, rc::Rc};
use web_sys::{console, Document};

/// A component controller, responsible for controlling the top level [`Fragment`] for a component,
/// in addition to the initial mount and update, and passing of updates from events into the
/// component.
pub struct Controller<C>
where
    C: Component + ?Sized,
{
    /// The component to be rendered. This will be used as the context for the [`Fragment`].
    /// Wrapping it in [`Rc<RefCell<T>>`] allows for access to the component in callbacks and in
    /// response to events, so it can be mutated as required.
    pub component: Rc<RefCell<C>>,

    pub bound_update: Option<Rc<Box<UpdateFn>>>,

    /// The [`EventRegistry`] for this component. Responsible for creating [`js_sys::Function`]s
    /// for a given `event_id`, and caching it so it can be re-used for future renders. Wrapped in
    /// an [`Rc<RefCell<T>>`] in order to share the same instance with children [`Fragment`]s.
    event_registry: Rc<RefCell<EventRegistry>>,

    /// The top level fragment that
    fragment: RefCell<Fragment>,
}

impl<C> Controller<C>
where
    C: Component + ?Sized + 'static,
{
    /// Create a new controller, returning a shared reference to the controller.
    pub fn new(
        document: &Document,
        component: ComponentWrapper<C>,
        bound_update: Option<Box<UpdateFn>>,
    ) -> Rc<RefCell<Self>> {
        // Create a reference to this controller. Initially contains `None`, however once the
        // controller is constructed it will be swapped in.
        let controller_reference = ControllerRef::new();

        // Create the event registry.
        let event_registry = EventRegistry::new({
            // Clone references that are required for use in the event registry.
            let controller_reference = controller_reference.clone();
            let component = component.clone_component();

            move |event_id, event| {
                // Perform a callback on the component
                let changed = { component.borrow_mut().handle_event(event_id, event) };

                if let Some(changed) = changed {
                    console::log_1(&"about to notify".into());
                    controller_reference.notify_changed(&changed);
                }
            }
        });

        // Create the fragment for the component, passing it a reference to the event registry.
        let fragment = component.fragment_builder.build(document, &event_registry);

        // Create the controller within a shared reference.
        let controller = Rc::new(RefCell::new(Self {
            component: component.component,
            event_registry,
            fragment: RefCell::new(fragment),
            bound_update: bound_update.map(|bound_update| Rc::new(bound_update)),
        }));

        // Place the reference to the controller within the shared self-reference.
        controller_reference.replace_with(&controller);

        controller
    }

    /// Mount the component to the provided [`Location`].
    pub fn mount(&self, location: &Location) {
        let mut fragment = self.fragment.borrow_mut();

        // Mount the fragment at the provided location
        fragment.mount(location);

        // Perform an update to get ensure the state is correct
        fragment.full_update();
    }

    pub fn update_fragment(&self, changed: &[usize]) {
        console::log_1(&"borrowing fragment".into());
        self.fragment.borrow_mut().update(changed);
        console::log_1(&"freeing fragment".into());
    }
}

impl<C> Dynamic for Controller<C>
where
    C: Component + ?Sized,
{
    fn mount(&mut self, location: &Location) {
        // Mount the fragment to the specified location
        self.fragment.borrow_mut().mount(location);
    }

    fn detach(&mut self, top_level: bool) {
        self.fragment.borrow_mut().detach(top_level);
    }

    fn update(&mut self, changed: &[usize]) {
        self.fragment.borrow_mut().update(changed);
    }
}
