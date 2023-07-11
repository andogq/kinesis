use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element};

use super::Component;
use crate::scheduler::EventCallbackClosure;

/// Wrapper around a component, used to provide additional functionality and assist with rendering
/// the component to the DOM.
pub struct ComponentController<C> {
    /// The ID of this component (unique within its siblings)
    component_id: usize,
    /// The component to be rendered
    component: C,

    /// A reference to the document in order to create elements
    document: Document,
    /// References to already rendered elements on the page, useful for updating in place, rather
    /// than completely re-rendering.
    elements: HashMap<usize, Element>,
}

impl<C> ComponentController<C>
where
    // Really not sure about this :(
    C: DerefMut,
    C::Target: Component,
{
    pub fn new(component_id: usize, component: C, document: &Document) -> Self {
        Self {
            component_id,
            component,
            document: document.clone(),
            elements: HashMap::new(),
        }
    }

    pub fn render(&mut self, event_callback_closure: &EventCallbackClosure) -> Result<(), JsValue> {
        if let Some((nodes, listener_map)) = self.component.render() {
            for (id, node) in nodes.into_iter().enumerate() {
                // Convert node to Element
                let el = node.build(&self.document)?;

                // Save the ID on the element
                let full_id = [self.component_id, id]
                    .into_iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                el.set_attribute("data-element-id", &full_id)?;

                // Apply required event listeners
                listener_map
                    .iter()
                    .filter(|listener| listener.element_path[0] == id)
                    .try_for_each(|listener| {
                        el.add_event_listener_with_callback(
                            &listener.event_type,
                            event_callback_closure.as_ref().unchecked_ref(),
                        )
                    })?;

                // Add to the DOM
                if let Some(element) = self.elements.get(&id) {
                    // Update an existing element
                    element.replace_with_with_node_1(&el)?;
                } else {
                    // Create a new element in the DOM
                    let body = self.document.body().expect("body to exist");
                    body.append_child(&el)?;
                }

                // Save the newly created element for future reference
                self.elements.insert(id, el);
            }
        }

        Ok(())
    }
}

impl<C> Deref for ComponentController<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<C> DerefMut for ComponentController<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
