use js_sys::Function;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event};

use super::{Component, EventType};
use crate::dom::renderable::DependencyRegistrationCallback;
use crate::util::HashMapList;

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

    dependency_registrations:
        Rc<RefCell<HashMapList<usize, (usize, DependencyRegistrationCallback)>>>,
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
            dependency_registrations: Rc::new(RefCell::new(HashMapList::new())),
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
            let get_callback_closure = &|id, event_type| {
                let mut callbacks = controller.callbacks.borrow_mut();

                // Prepare event handler closure incase it needs to be re-created
                let event_closure = {
                    let component = Rc::clone(&controller.component);
                    let controller = self.clone();

                    move |event: Event| {
                        let changed = component.borrow_mut().handle_event(id, event_type, event);

                        // Check if anything was actually changed in the event handler
                        if let Some(changed) = changed {
                            let component = component.borrow();
                            let controller = controller.0.borrow();
                            let dependency_registrations =
                                controller.dependency_registrations.borrow();

                            let mut debug_dep_counter = 0;

                            for change in changed {
                                for (update_type, callback) in
                                    dependency_registrations.get(&change).into_iter().flatten()
                                {
                                    if let Some(content) = component.handle_update(*update_type) {
                                        debug_dep_counter += 1;
                                        callback(content);
                                    }
                                }
                            }

                            console::log_1(&format!("Updating {debug_dep_counter} deps").into());
                        }
                    }
                };

                // Cache the closures so they can be re-used
                callbacks
                    .entry((id, event_type))
                    .or_insert_with(|| {
                        // Create a closure to bind with JS for this specific handler
                        Closure::<dyn Fn(Event)>::new(event_closure)
                            .into_js_value()
                            .unchecked_into()
                    })
                    .clone()
            };

            // Build or update the element
            if let Some(build_result) = node.render(
                &controller.document,
                mounted_elements.next(),
                get_callback_closure,
            )? {
                // Only perform render if a build result is returned
                if first_render {
                    parent.append_child(&build_result.element)?;
                }

                if let Some(children) = build_result.children {
                    // Add the children to the render queue
                    element_queue.extend(
                        children
                            .into_iter()
                            .map(|child| (child, build_result.element.clone())),
                    );
                }

                // Save the mounted element for this node
                controller
                    .mounted_elements
                    .get_or_insert(Vec::new())
                    .push(build_result.element);

                console::log_1(&"Running initial dependency render".into());
                let mut dependency_registrations = controller.dependency_registrations.borrow_mut();
                let component = controller.component.borrow();

                for dynamic_content in build_result.dynamic_content {
                    // Run action
                    if let Some(content) = component.handle_update(dynamic_content.update_type) {
                        (dynamic_content.callback)(content);
                    }

                    // Save action
                    for dependency in dynamic_content.dependencies {
                        dependency_registrations.insert(
                            dependency,
                            (
                                dynamic_content.update_type,
                                dynamic_content.callback.clone(),
                            ),
                        );
                    }
                }
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
