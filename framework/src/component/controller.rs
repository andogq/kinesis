use js_sys::Function;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event};

use super::{identifier::Identifier, Component, EventType, RenderType};
use crate::dom::renderable::{DependencyRegistrationCallback, Renderable, RenderedNode};
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

    rendered_elements: Rc<RefCell<HashMap<RenderType, Vec<RenderedNode>>>>,
}

/// Wrapper for component controller, allowing a reference to be passed to closures as required
pub struct ComponentControllerRef(Rc<RefCell<ComponentController>>);

struct RenderQueueEntry {
    renderable: Box<dyn Renderable>,
    parent: RenderedNode,
    identifier: Identifier,
}

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

            rendered_elements: Rc::new(RefCell::new(HashMap::new())),
        })))
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let (first_render, mut mounted_elements, mut element_queue) = {
            let mut controller = self.0.borrow_mut();

            // Take the mounted elements, so they can be passed into the DOM node render
            let first_render = controller.mounted_elements.is_none();
            let mounted_elements = controller.mounted_elements.take().into_iter().flatten();

            // Queue of elements to render (children will be pushed in same order)
            let element_queue = VecDeque::from_iter(
                controller
                    .component
                    .borrow()
                    .render(RenderType::Root)
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(|(index, node)| RenderQueueEntry {
                        renderable: node,
                        parent: RenderedNode::Element(controller.parent.clone()),
                        identifier: controller.identifier.child(index),
                    }),
            );

            (first_render, mounted_elements, element_queue)
        };

        while let Some(RenderQueueEntry {
            renderable,
            parent,
            identifier,
        }) = element_queue.pop_front()
        {
            let children_to_render = self.render_renderable(
                renderable,
                parent,
                identifier,
                mounted_elements.next(),
                first_render,
            )?;

            if let Some(children_to_render) = children_to_render {
                for child in children_to_render {
                    element_queue.push_front(child);
                }
            }
        }

        let mut controller = self.0.borrow();
        console::log_1(&"Running initial dependency render".into());
        let mut dependency_registrations = controller.dependency_registrations.borrow_mut();
        let component = controller.component.borrow();

        // TODO: Perform initial render of dynamic portions

        // for dynamic_content in build_result.dynamic_content {
        // Run action
        // TODO: Work out how to get rendered content into the DOM (same as above)
        // if let Some(content) = component.render(dynamic_content.update_type) {
        // (dynamic_content.callback)(content);
        // }

        // Save action
        // for dependency in dynamic_content.dependencies {
        // dependency_registrations.insert(
        //     dependency,
        //     (
        //         dynamic_content.update_type,
        //         dynamic_content.callback.clone(),
        //     ),
        // );
        //     }
        // }

        Ok(())
    }

    /// Generates a closure that can be attached to a DOM element, in order to respond to an event.
    /// The target element's identifier must be included, so that it can be sent back to the
    /// controller to identify the source of the event, and the event type is required in order to
    /// correctly trigger the relevant callback.
    fn create_event_callback_closure(
        &self,
        identifier: &Identifier,
        event_type: EventType,
    ) -> Box<dyn Fn(Event)> {
        let controller = self.0.borrow();
        let component = Rc::clone(&controller.component);
        let controller_rc = self.clone();

        let identifier = identifier.clone();

        Box::new(move |event: Event| {
            let changed =
                component
                    .borrow_mut()
                    .handle_event(identifier.clone(), event_type, event);

            // Check if anything was actually changed in the event handler
            if let Some(changed) = changed {
                let component = component.borrow();
                let controller = controller_rc.0.borrow();
                let dependency_registrations = controller.dependency_registrations.borrow();

                let mut debug_dep_counter = 0;

                for change in changed {
                    for (update_type, callback) in
                        dependency_registrations.get(&change).into_iter().flatten()
                    {
                        // Perform a partial render of the component
                        let render_type = RenderType::Partial(*update_type);

                        // Fetch the previously rendered content
                        let mut rendered_elements = controller.rendered_elements.borrow_mut();
                        let existing = rendered_elements.get(&render_type);

                        let renderables = component.render(render_type.clone());

                        // Insert the newly rendered content
                        if let Some(renderables) = renderables {
                            // rendered_elements.insert(render_type, renderables);
                            // TODO: Build each of the renderables that are returned
                            // Need a recursive render call???

                            // TODO: Insert new renderables into the DOM
                        } else {
                            rendered_elements.remove(&render_type);
                        }

                        // TODO: Insert the generated content back into the DOM
                        debug_dep_counter += 1;
                    }
                }

                console::log_1(&format!("Updating {debug_dep_counter} deps").into());
            }
        })
    }

    fn render_renderable(
        &self,
        node: Box<dyn Renderable>,
        parent: RenderedNode,
        identifier: Identifier,
        mounted_element: Option<RenderedNode>,
        append_child: bool,
    ) -> Result<Option<Vec<RenderQueueEntry>>, JsValue> {
        // Only allow rendering a child into a parent that is an Element
        let parent = if let RenderedNode::Element(parent) = parent {
            parent
        } else {
            console::log_1(
                &"Skipping render due to child attempting to render in a non element node.".into(),
            );
            return Ok(None);
        };

        console::log_1(&format!("{identifier:?}").into());

        // A callback that can be used to generate a new event closure
        let generate_event_callback_closure = &|event_type| {
            let controller = self.0.borrow();
            let mut callbacks = controller.callbacks.borrow_mut();

            // Prepare event handler closure incase it needs to be re-created
            let event_closure = self.create_event_callback_closure(&identifier, event_type);

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
        let document = self.0.borrow().document.clone();
        if let Some(build_result) =
            node.render(&document, mounted_element, generate_event_callback_closure)?
        {
            let mut controller = self.0.borrow_mut();
            // Only perform render if a build result is returned and there's an element to
            // render
            if let Some(element) = &build_result.element {
                // Save the mounted element for this node
                controller
                    .mounted_elements
                    .get_or_insert(Vec::new())
                    .push(element.clone());

                if append_child {
                    parent.append_child(&element.into())?;
                }
            }

            let children = build_result.children.map(|children| {
                let parent = build_result
                    .element
                    .unwrap_or(RenderedNode::Element(parent));

                let children =
                    children
                        .into_iter()
                        .enumerate()
                        .map(|(index, child)| RenderQueueEntry {
                            renderable: child,
                            parent: parent.clone(),
                            identifier: identifier.child(index),
                        });

                // Add the children to the render queue
                if build_result.in_place {
                    // TODO: Make sure order is correct here
                    children
                        .rev()
                        // .for_each(|entry| element_queue.push_front(entry))
                        .collect()
                } else {
                    children.collect()
                }
            });

            Ok(children)
        } else {
            Ok(None)
        }
    }
}

impl Clone for ComponentControllerRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
