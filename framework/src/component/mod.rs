use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element};

use crate::{dom::DomNode, scheduler::EventCallbackClosure};

pub struct EventListener {
    pub element_path: VecDeque<usize>,
    pub event_type: String,
}
impl EventListener {
    pub fn new(el_id: usize, event_type: &str) -> Self {
        Self {
            element_path: VecDeque::from_iter([el_id]),
            event_type: event_type.to_string(),
        }
    }

    pub fn nested_in(mut self, el_id: usize) -> Self {
        self.element_path.push_front(el_id);
        self
    }
}

/// Trait that represents a renderable component
pub trait Component {
    /// Handle an incomming event, allowing for mutation of the component's state.
    fn handle_event(&mut self);

    /// Renders the component for a given state. Can optionally not render anything.
    fn render(&self) -> Option<(Vec<DomNode>, Vec<EventListener>)>;
}

pub struct ComponentController<C> {
    component: C,
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
    pub fn new(component: C, document: &Document) -> Self {
        Self {
            component,
            document: document.clone(),
            elements: HashMap::new(),
        }
    }

    pub fn render(&mut self, event_callback_closure: &EventCallbackClosure) -> Result<(), JsValue> {
        if let Some((nodes, listener_map)) = self.component.render() {
            for (i, node) in nodes.into_iter().enumerate() {
                // Convert node to Element
                let el = node.build(&self.document)?;

                // Apply required event listeners
                listener_map
                    .iter()
                    .filter(|listener| listener.element_path[0] == i)
                    .try_for_each(|listener| {
                        el.add_event_listener_with_callback(
                            &listener.event_type,
                            event_callback_closure.as_ref().unchecked_ref(),
                        )
                    })?;

                // Add to the DOM
                if let Some(element) = self.elements.get(&i) {
                    // Update an existing element
                    element.replace_with_with_node_1(&el)?;
                } else {
                    // Create a new element in the DOM
                    let body = self.document.body().expect("body to exist");
                    body.append_child(&el)?;
                }

                // Save the newly created element for future reference
                self.elements.insert(i, el);
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
