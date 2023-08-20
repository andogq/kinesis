use web_sys::Document;

use super::{NestedController, UpdateComponentFn};
use crate::{
    component::{Component, Controller},
    fragment::FragmentBuilder,
};
use std::{cell::RefCell, rc::Rc};

/// Represents a [`Component`] that is nested within another component. Is used whilst building a
/// fragment, but before the component is turned into a [`crate::component::Controller`].
pub struct NestedComponent<C>
where
    C: Component + ?Sized,
{
    /// The component to be nested.
    pub component: Rc<RefCell<C>>,

    pub fragment_builder: FragmentBuilder,

    /// An update function to alter the internal state of the component for the given context.
    pub update: Box<UpdateComponentFn>,
}

impl<C> NestedComponent<C>
where
    C: 'static + Component + ?Sized,
{
    /// Consume the nested component, and turn it into a [`NestedController`], handling the
    /// wrapping of the [`Component`] in a [`Controller`].
    pub fn into_controller(self, document: &Document) -> NestedController<C> {
        let controller = Controller::<C>::new(document, (self.component, self.fragment_builder));

        NestedController {
            controller,
            update: self.update,
        }
    }
}
