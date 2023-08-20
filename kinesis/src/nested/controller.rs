use super::UpdateComponentFn;
use crate::component::{Component, Controller};
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

impl<C> NestedController<C>
where
    C: Component + ?Sized,
{
    /// Update the internal state of the [`Component`] with the provided context.
    pub fn update(&self, changed: &[usize]) {
        (self.update)(changed)
    }
}
