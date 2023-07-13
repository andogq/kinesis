use js_sys::Function;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Document, Element, Event};

use super::{Component, EventType};

/// Wrapper around a component, used to provide additional functionality and assist with rendering
/// the component to the DOM.
pub struct ComponentController {
    /// The component to be rendered
    component: Rc<RefCell<dyn Component>>,

    /// A reference to the document in order to create elements
    document: Document,
    /// References to already rendered elements on the page, useful for updating in place, rather
    /// than completely re-rendering.
    elements: Rc<RefCell<HashMap<usize, Element>>>,

    callbacks: Rc<RefCell<HashMap<(usize, EventType), Function>>>,
}

/// Wrapper for component controller, allowing a reference to be passed to closures as required
pub struct ComponentControllerRef(Rc<RefCell<ComponentController>>);

impl ComponentControllerRef {
    pub fn new<C>(component: C, document: &Document) -> Self
    where
        C: Component + 'static,
    {
        Self(Rc::new(RefCell::new(ComponentController {
            component: Rc::new(RefCell::new(component)),
            document: document.clone(),
            elements: Rc::new(RefCell::new(HashMap::new())),
            callbacks: Rc::new(RefCell::new(HashMap::new())),
        })))
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let controller = self.0.borrow();

        for (id, node) in controller
            .component
            .borrow()
            .render()
            .into_iter()
            .enumerate()
        {
            // Convert node to Element
            let el = node.build(&controller.document, &|id, event_type| {
                let mut callbacks = controller.callbacks.borrow_mut();

                // Cache the closures so they can be re-used
                callbacks
                    .entry((id, event_type))
                    .or_insert_with(|| {
                        // Create a closure to bind with JS for this specific handler
                        Closure::<dyn Fn(Event)>::new({
                            let component = Rc::clone(&controller.component);
                            let controller = self.clone();

                            move |event: Event| {
                                component.borrow_mut().handle_event(id, event_type, event);

                                // Trigger component re-render
                                controller.render().expect("render to succeed");
                            }
                        })
                        .into_js_value()
                        .unchecked_into()
                    })
                    .clone()
            })?;

            let mut elements = controller.elements.borrow_mut();

            // Add to the DOM
            if let Some(element) = elements.get(&id) {
                // Update an existing element
                element.replace_with_with_node_1(&el)?;
            } else {
                // Create a new element in the DOM
                let body = controller.document.body().expect("body to exist");
                body.append_child(&el)?;
            }

            // Save the newly created element for future reference
            elements.insert(id, el);
        }

        Ok(())
    }
}

impl Clone for ComponentControllerRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
