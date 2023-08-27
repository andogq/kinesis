use std::{cell::RefCell, rc::Rc};

use web_sys::console;

use super::Controller;
use crate::component::Component;

pub struct ControllerRef<C>(Rc<RefCell<Option<Rc<RefCell<Controller<C>>>>>>)
where
    C: Component + ?Sized;

impl<C> ControllerRef<C>
where
    C: Component + ?Sized + 'static,
{
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    pub fn replace_with(&self, controller: &Rc<RefCell<Controller<C>>>) {
        *self.0.borrow_mut() = Some(Rc::clone(controller));
    }

    pub fn get_ref(&self) -> Option<Rc<RefCell<Controller<C>>>> {
        self.0.borrow().as_ref().map(Rc::clone)
    }

    /// Handles updating all of the required parts of a component.
    ///
    /// Notably, this will update the fragment within the controller, and also trigger the
    /// `bound_update` for the controller, which is an optional closure passed in by the parent of
    /// the component. This has to be called from this method, as it's possible that the parent
    /// will attempt to gain a mutable borrow on the child, which cannot be done if the child is
    /// already mutably borrowed to run the update function.
    pub fn notify_changed(&self, changed: &[usize]) {
        let bound_update = {
            let controller_ref = self.0.borrow();

            let controller = controller_ref
                .as_ref()
                .expect("controller to be present")
                .borrow();

            controller.update_fragment(changed);

            controller.bound_update.clone()
        };

        // Update bound parent
        if let Some(bound_update) = bound_update {
            bound_update(changed);
        }
    }
}

impl<C> Clone for ControllerRef<C>
where
    C: Component + ?Sized,
{
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
