use super::UpdateComponentFn;
use crate::{
    component::{Component, Controller},
    fragment::{Dynamic, Location},
};
use std::{cell::RefCell, rc::Rc};

/// A [`Controller`] that is nested within another [`Controller`].
pub struct NestedController<C>
where
    C: Component + ?Sized,
{
    /// A reference to the [`Controller`] that is nested.
    pub controller: Rc<RefCell<Controller<C>>>,

    /// A method that will update state of the [`Component`] within the [`Controller`].
    pub update: Box<UpdateComponentFn>,
}

impl<C> Dynamic for NestedController<C>
where
    C: Component + ?Sized + 'static,
{
    fn update(&mut self, changed: &[usize]) {
        // Run attached update function
        (self.update)(changed);

        // TODO: Convert parent changed to component changed

        // Run controller update
        self.controller.borrow_mut().update(changed);
    }

    fn mount(&mut self, location: &Location) {
        self.controller.borrow_mut().mount(location);
    }

    fn detach(&mut self, top_level: bool) {
        self.controller.borrow_mut().detach(top_level);
    }
}
