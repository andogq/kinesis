use js_sys::Function;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event};

use super::{identifier::Identifier, Component, EventType, RenderType};
use crate::dom::renderable::{DependencyRegistrationCallback, RenderedNode};
use crate::util::HashMapList;

/// Wrapper around a component, used to provide additional functionality and assist with rendering
/// the component to the DOM.
pub struct ComponentController {
    /// The component to be rendered
    component: Rc<RefCell<dyn Component>>,

    parent: Element,
    mounted_elements: Option<Vec<RenderedNode>>,

    /// A reference to the document in order to create elements
    document: Document,

    callbacks: Rc<RefCell<HashMap<(Identifier, EventType), Function>>>,

    dependency_registrations:
        Rc<RefCell<HashMapList<usize, (usize, DependencyRegistrationCallback)>>>,

    identifier: Identifier,
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
            identifier: Identifier::new(),
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
                .render(RenderType::Root)
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(|(index, node)| {
                    (
                        node,
                        RenderedNode::Element(controller.parent.clone()),
                        controller.identifier.child(index),
                    )
                }),
        );

        while let Some((node, parent, identifier)) = element_queue.pop_front() {
            // Only allow rendering a child into a parent that is an Element
            let parent = if let RenderedNode::Element(parent) = parent {
                parent
            } else {
                console::log_1(
                    &"Skipping render due to child attempting to render in a non element node."
                        .into(),
                );
                continue;
            };

            console::log_1(&format!("{identifier:?}").into());
            let get_callback_closure = &|event_type| {
                let mut callbacks = controller.callbacks.borrow_mut();

                // Prepare event handler closure incase it needs to be re-created
                let event_closure = {
                    let component = Rc::clone(&controller.component);
                    let controller = self.clone();
                    let identifier = identifier.clone();

                    move |event: Event| {
                        let changed = component.borrow_mut().handle_event(
                            identifier.clone(),
                            event_type,
                            event,
                        );

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
                                    // Perform a partial render of the component
                                    component.render(RenderType::Partial(*update_type));

                                    // TODO: Insert the generated content back into the DOM
                                    // debug_dep_counter += 1;
                                    // callback(content);
                                }
                            }

                            console::log_1(&format!("Updating {debug_dep_counter} deps").into());
                        }
                    }
                };

                // Cache the closures so they can be re-used
                callbacks
                    .entry((identifier.clone(), event_type))
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
                // Only perform render if a build result is returned and there's an element to
                // render
                if let Some(element) = &build_result.element {
                    // Save the mounted element for this node
                    controller
                        .mounted_elements
                        .get_or_insert(Vec::new())
                        .push(element.clone());

                    if first_render {
                        parent.append_child(&element.into())?;
                    }
                }

                if let Some(children) = build_result.children {
                    let parent = build_result
                        .element
                        .unwrap_or(RenderedNode::Element(parent));

                    let children = children
                        .into_iter()
                        .enumerate()
                        .map(|(index, child)| (child, parent.clone(), identifier.child(index)));

                    // Add the children to the render queue
                    if build_result.in_place {
                        children
                            .rev()
                            .for_each(|entry| element_queue.push_front(entry));
                    } else {
                        element_queue.extend(children);
                    }
                }

                console::log_1(&"Running initial dependency render".into());
                let mut dependency_registrations = controller.dependency_registrations.borrow_mut();
                let component = controller.component.borrow();

                for dynamic_content in build_result.dynamic_content {
                    // Run action
                    // TODO: Work out how to get rendered content into the DOM (same as above)
                    // if let Some(content) = component.render(dynamic_content.update_type) {
                    // (dynamic_content.callback)(content);
                    // }

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
