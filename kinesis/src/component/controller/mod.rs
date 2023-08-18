use super::Component;
use crate::fragment::{Dynamic, EventRegistry, Fragment, Location};
use std::{cell::RefCell, ops::Deref, rc::Rc};
use web_sys::Document;

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
    pub component: Rc<RefCell<Box<C>>>,

    /// The [`EventRegistry`] for this component. Responsible for creating [`js_sys::Function`]s
    /// for a given `event_id`, and caching it so it can be re-used for future renders. Wrapped in
    /// an [`Rc<RefCell<T>>`] in order to share the same instance with children [`Fragment`]s.
    event_registry: Rc<RefCell<EventRegistry>>,

    /// The top level fragment that
    fragment: Fragment<C::Ctx>,
}

impl<C> Controller<C>
where
    C: Component + ?Sized + 'static,
{
    /// Create a new controller, returning a shared reference to the controller.
    pub fn new<SizedC>(document: &Document, component: SizedC) -> Rc<RefCell<Controller<SizedC>>>
    where
        SizedC: Component + 'static,
    {
        Self::from_ref_cell(document, RefCell::new(Box::new(component)))
    }

    pub fn from_ref_cell<_C>(
        document: &Document,
        component: RefCell<Box<_C>>,
    ) -> Rc<RefCell<Controller<_C>>>
    where
        _C: Component + ?Sized + 'static,
    {
        // Create a reference to this controller. Initially contains `None`, however once the
        // controller is constructed it will be swapped in.
        let controller_reference =
            Rc::new(RefCell::new(Option::<Rc<RefCell<Controller<_C>>>>::None));

        // Place the component within a shared reference, so that it can be accessed within the
        // event registry.
        let component = Rc::new(component);

        // Create the event registry.
        let event_registry = EventRegistry::new({
            // Clone references that are required for use in the event registry.
            let controller_reference = Rc::clone(&controller_reference);
            let component = Rc::clone(&component);

            move |event_id, event| {
                // Perform a callback on the component
                let mut component = component.borrow_mut();
                let Some(changed) = component.handle_event(event_id, event) else { return };

                // Need to trigger an update on the fragment
                if let Some(controller) = controller_reference.borrow().as_ref() {
                    controller
                        .borrow_mut()
                        .fragment
                        .update(component.get_context(), &changed);
                }
            }
        });

        // Create the fragment for the component, passing it a reference to the event registry.
        let fragment = component.borrow().render().build(document, &event_registry);

        // Create the controller within a shared reference.
        let controller = Rc::new(RefCell::new(Controller::<_C> {
            component,
            event_registry,
            fragment,
        }));

        // Place the reference to the controller within the shared self-reference.
        *controller_reference.borrow_mut() = Some(Rc::clone(&controller));

        controller
    }

    /// Mount the component to the provided [`Location`].
    pub fn mount(&mut self, location: &Location) {
        let component = self.component.borrow_mut();

        // Mount the fragment at the provided location
        self.fragment.mount(location);

        // Perform an update to get ensure the state is correct
        self.fragment.full_update(component.deref().get_context());
    }
}

impl<C> Dynamic for Controller<C>
where
    C: Component + ?Sized,
{
    type Ctx = ();

    fn mount(&mut self, location: &Location) {
        // Mount the fragment to the specified location
        self.fragment.mount(location);
    }

    fn detach(&mut self, top_level: bool) {
        self.fragment.detach(top_level);
    }

    fn update(&mut self, context: &Self::Ctx, changed: &[usize]) {
        // TODO: Might be redundant if context is updated from the parent down
        // let component = self.component.borrow();
        // self.fragment.update(component.get_context(), &[]);
    }
}
