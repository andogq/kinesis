use super::UpdateComponentFn;
use crate::component::{Component, Controller};
use std::{any::Any, cell::RefCell, rc::Rc};

/// A [`Controller`] that is nested within another [`Controller`].
pub struct NestedController<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    /// A reference to the [`Controller`] that is nested.
    pub controller: Rc<RefCell<Controller<C>>>,

    /// A method that will update state of the [`Component`] within the [`Controller`].
    pub update: Box<UpdateComponentFn<Ctx, C>>,
}

impl<Ctx, C> NestedController<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    /// Update the internal state of the [`Component`] with the provided context.
    pub fn update(&self, context: &Ctx, changed: &[usize]) {
        (self.update)(
            context,
            changed,
            &mut self.controller.borrow().component.borrow_mut(),
        )
    }
}
