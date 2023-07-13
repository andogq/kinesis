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

    parent: Element,
    mounted_elements: Option<Vec<Element>>,

    /// A reference to the document in order to create elements
    document: Document,

    callbacks: Rc<RefCell<HashMap<(usize, EventType), Function>>>,
}

/// Wrapper for component controller, allowing a reference to be passed to closures as required
pub struct ComponentControllerRef(Rc<RefCell<ComponentController>>);

impl ComponentControllerRef {
    pub fn new<C>(component: C, document: &Document, parent: Element) -> Self
    where
        C: Component + 'static,
    {
        Self(Rc::new(RefCell::new(ComponentController {
            component: Rc::new(RefCell::new(component)),
            parent,
            mounted_elements: None,
            document: document.clone(),
            callbacks: Rc::new(RefCell::new(HashMap::new())),
        })))
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let mut controller = self.0.borrow_mut();

        // Build elements
        let elements = {
            let component = controller.component.borrow();
            component
                .render()
                .into_iter()
                .map(|node| {
                    // Convert node to Element
                    node.build(&controller.document, &|id, event_type| {
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
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        // Work out how to mount elements
        if let Some(mounted_elements) = controller.mounted_elements.as_ref() {
            // Replace with existin elements
            for (el, target) in elements.iter().zip(mounted_elements) {
                target.replace_with_with_node_1(el)?;
            }
        } else {
            for el in elements.iter() {
                controller.parent.append_child(el)?;
            }
        }

        controller.mounted_elements = Some(elements);

        Ok(())
    }
}

impl Clone for ComponentControllerRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
