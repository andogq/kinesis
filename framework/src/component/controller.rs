use js_sys::Function;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event};

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

        // Take the mounted elements, so they can be passed into the DOM node render
        let first_render = controller.mounted_elements.is_none();
        let mut mounted_elements = controller.mounted_elements.take().into_iter().flatten();

        // Queue of elements to render (children will be pushed in same order)
        let mut element_queue = VecDeque::from_iter(
            controller
                .component
                .borrow()
                .render()
                .into_iter()
                .map(|node| (node, controller.parent.clone())),
        );

        while let Some((node, parent)) = element_queue.pop_front() {
            // Convert node to Element
            let (node_element, children) = node.build(
                &controller.document,
                controller.component.borrow().get_counter_temp(),
                mounted_elements.next(),
                &|id, event_type| {
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
                },
            )?;

            if first_render {
                parent.append_child(&node_element)?;
            }

            if let Some(children) = children {
                // Add the children to the render queue
                element_queue.extend(
                    children
                        .into_iter()
                        .map(|child| (child, node_element.clone())),
                );
            }

            // Save the mounted element for this node
            controller
                .mounted_elements
                .get_or_insert(Vec::new())
                .push(node_element);
        }

        Ok(())
    }
}

impl Clone for ComponentControllerRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
