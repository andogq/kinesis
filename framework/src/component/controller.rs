use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event};

use super::Component;

/// Wrapper around a component, used to provide additional functionality and assist with rendering
/// the component to the DOM.
pub struct ComponentController {
    /// The ID of this component (unique within its siblings)
    component_id: usize,
    /// The component to be rendered
    component: Box<dyn Component>,

    /// A reference to the document in order to create elements
    document: Document,
    /// References to already rendered elements on the page, useful for updating in place, rather
    /// than completely re-rendering.
    elements: HashMap<usize, Element>,

    callbacks: Vec<Closure<dyn Fn(Event)>>,
}

pub struct ComponentControllerRef(Rc<RefCell<ComponentController>>);

impl ComponentControllerRef {
    pub fn new<C>(component_id: usize, component: C, document: &Document) -> Self
    where
        C: Component + 'static,
    {
        Self(Rc::new(RefCell::new(ComponentController {
            component_id,
            component: Box::new(component),
            document: document.clone(),
            elements: HashMap::new(),
            callbacks: Vec::new(),
        })))
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let mut controller = self.0.borrow_mut();

        if let Some((nodes, listener_map)) = {
            // Perform render within scope to ensure that mutable reference to component is dropped
            controller.component.render()
        } {
            for (id, node) in nodes.into_iter().enumerate() {
                // Convert node to Element
                let el = node.build(&controller.document)?;

                // Apply required event listeners
                listener_map
                    .iter()
                    .filter(|listener| listener.element_path[0] == id)
                    .try_for_each(|listener| -> Result<(), JsValue> {
                        // TODO: If individual closures are to be created for each component, an
                        // Rc<RefCell> will be  required to lock access to the components                        el.add_event_listener_with_callback(
                        // Create a closure to bind with JS for this specific handler
                        let controller_rc = self.clone();
                        let callback = Closure::<dyn Fn(_)>::new(move |_event: Event| {
                            // TODO: Pass correct event type to handler
                            controller_rc.0.borrow_mut().component.handle_event();

                            // Trigger re-render of component
                            controller_rc.render().expect("render to succeed");
                        });

                        // Bind callback to element
                        el.add_event_listener_with_callback(
                            &String::from(listener.event_type),
                            callback.as_ref().unchecked_ref(),
                        )?;

                        // Save callback so that closure doesn't get freed
                        controller.callbacks.push(callback);

                        Ok(())
                    })?;

                // Add to the DOM
                if let Some(element) = controller.elements.get(&id) {
                    // Update an existing element
                    element.replace_with_with_node_1(&el)?;
                } else {
                    // Create a new element in the DOM
                    let body = controller.document.body().expect("body to exist");
                    body.append_child(&el)?;
                }

                // Save the newly created element for future reference
                controller.elements.insert(id, el);
            }
        }

        Ok(())
    }
}

impl Clone for ComponentControllerRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
