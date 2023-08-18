use web_sys::Document;

use super::{NestedController, UpdateComponentFn};
use crate::component::{Component, Controller};
use std::{any::Any, cell::RefCell};

/// Represents a [`Component`] that is nested within another component. Is used whilst building a
/// fragment, but before the component is turned into a [`crate::component::Controller`].
pub struct NestedComponent<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: Component + ?Sized,
{
    /// The component to be nested.
    pub component: RefCell<Box<C>>,

    /// An update function to alter the internal state of the component for the given context.
    pub update: Box<UpdateComponentFn<Ctx, C>>,
}

impl<Ctx, C> NestedComponent<Ctx, C>
where
    Ctx: Any + ?Sized,
    C: 'static + Component + ?Sized,
{
    /// Consume the nested component, and turn it into a [`NestedController`], handling the
    /// wrapping of the [`Component`] in a [`Controller`].
    fn to_controller(self, document: &Document) -> NestedController<Ctx, C> {
        let controller = Controller::<C>::from_ref_cell(document, self.component);

        NestedController {
            controller,
            update: self.update,
        }
    }
}
