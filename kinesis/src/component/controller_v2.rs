use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    ops::Deref,
    rc::Rc,
};

use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, Event, Node};

use crate::{
    component::RenderType,
    dom::{
        renderable::{DynamicContent, RenderedNode},
        EventType,
    },
    util::{HashMapList, Tree},
};

use super::{Component, Identifier};

pub struct Controller {
    /// The component that this controller is handling.
    component: Rc<RefCell<dyn Component>>,

    /// A reference to the document object, so that elements can be created.
    document: Document,

    /// Maps an update type and corresponding identifier for the rendered node (value) to it's
    /// dependencies (key).
    dependencies: HashMapList<usize, (usize, Identifier)>,

    element_tree: Tree<usize, RenderedNode>,
    previous_identifiers: HashSet<Identifier>,
}

impl Controller {
    pub fn new<C>(component: C, document: &Document) -> Self
    where
        C: Component + 'static,
    {
        Self {
            component: Rc::new(RefCell::new(component)) as Rc<RefCell<dyn Component>>,
            document: document.clone(),
            dependencies: HashMapList::new(),
            element_tree: Tree::new(),
            previous_identifiers: HashSet::new(),
        }
    }

    /// Generate a closure that cen be attached to an event listener for a DOM element. The
    /// identifier and event types are required, so they can be passed back into the controller in
    /// order for it to appropriately handle the update.
    ///
    /// Requires mutable access to the controller, so that it can use it within the callback.
    fn get_event_callback_closure(
        controller_ref: &ControllerRef,
        identifier: Identifier,
        event_type: EventType,
    ) -> Function {
        let controller_ref = controller_ref.clone();

        Closure::<dyn Fn(Event)>::new(Box::new(move |event: Event| {
            // Gain a mutable reference to the controller
            let mut controller = controller_ref.0.borrow_mut();
            let changed = controller.component.borrow_mut().handle_event(
                identifier.clone(),
                event_type,
                event,
            );

            // Trigger the update in the component
            if let Some(changed) = changed {
                controller.handle_update(&controller_ref, &changed).unwrap();
            }
        }) as Box<dyn Fn(Event)>)
        .into_js_value()
        .unchecked_into()
    }

    /// Will update component in the DOM as needed to reflect the changed dependencies.
    pub fn handle_update(
        &mut self,
        controller_ref: &ControllerRef,
        changed_dependencies: &[usize],
    ) -> Result<(), JsValue> {
        // Determine what renders need to run
        for (render_type, identifier) in changed_dependencies
            .iter()
            .filter_map(|dependency| self.dependencies.get(dependency))
            .flatten()
            .cloned()
            .collect::<HashSet<_>>()
        {
            self.perform_render(
                controller_ref,
                None,
                RenderType::Partial(render_type),
                &identifier,
            )
            .unwrap();
        }

        Ok(())
    }

    pub fn perform_render(
        &mut self,
        controller_ref: &ControllerRef,
        // Optional parent, as if it is not supplied then it is assumed the node is being renderd
        // in place
        parent: Option<RenderedNode>,
        render_type: RenderType,
        root_identifier: &Identifier,
    ) -> Result<(), JsValue> {
        let root_identifier = root_identifier.clone();
        let document = self.document.clone();
        let component = self.component.borrow();

        // Perform the initial render of the component, to retrieve all of the renderables
        if let Some(renderables) = component.render(render_type.clone()) {
            // Build a queue of renderables to render, it's identifier, and the parent element it
            // will be rendered into.
            let mut queue = renderables
                .into_iter()
                .enumerate()
                .map(|(i, renderable)| (root_identifier.child(i), renderable, parent.clone()))
                .collect::<VecDeque<_>>();

            let mut used_element_identifiers = HashSet::new();

            while let Some((identifier, renderable, parent)) = queue.pop_front() {
                // Attempt to find element in the element cache
                let element = self.element_tree.get(identifier.as_ref()).cloned();

                console::log_1(
                    &format!(
                        "cached element found for {identifier}? {}",
                        element.is_some()
                    )
                    .into(),
                );

                // Render each of the child renderables
                if let Some(result) = renderable.render(
                    // Share a reference to the document so that it can create new elements
                    &document,
                    // Share a reference to the component so that it can perform partial renders
                    component.deref(),
                    element.clone(),
                    // Share a callback that can be used to create an event closure
                    &mut |event_type| {
                        Controller::get_event_callback_closure(
                            controller_ref,
                            identifier.clone(),
                            event_type,
                        )
                    },
                )? {
                    if let Some(children) = result.children {
                        queue.extend(children.into_iter().enumerate().map(|(i, renderable)| {
                            (
                                identifier.child(i),
                                renderable,
                                if result.in_place {
                                    parent.clone()
                                } else {
                                    result.element.clone()
                                },
                            )
                        }));
                    }

                    // Save the dependencies
                    for (dependency, update_type) in result.dynamic_content.into_iter().flat_map(
                        |DynamicContent {
                             dependencies,
                             update_type,
                         }| {
                            dependencies
                                .into_iter()
                                .map(move |dependency| (dependency, update_type))
                        },
                    ) {
                        // TODO: Work out way to de-dupe these before insertion
                        self.dependencies
                            .insert(dependency, (update_type, identifier.clone()));
                    }

                    if let Some(element) = result.element {
                        // Save the element for future use
                        self.element_tree
                            .insert(identifier.clone(), element.clone())
                            .expect("to be able to insert item into tree");
                        used_element_identifiers.insert(identifier);

                        // Add the element to the parent
                        if let Some(parent) = parent {
                            Node::from(&parent).append_child(&(&element).into())?;
                        }
                    }
                }
            }

            // Get rid of unused elements from the render
            // Somehow need to associate certain element instances with a render type. Once that
            // render type completes, then remove any elements that were not used previously.

            // TODO: Need to somehow seperate keys for each of the previous render types, but in a
            // way that they can be related to each other (eg a Root render type that uses other
            // partial render types).

            if let Some(node) = self.element_tree.get_node(root_identifier.as_ref()) {
                let mut to_remove = vec![];
                let mut queue = vec![(root_identifier, node)];

                // Traverse the tree from here and work out what wasn't altered
                while let Some((identifier, node)) = queue.pop() {
                    // Add children to queue
                    queue.extend(node.children.0.iter_mut().map(|(id, node)| {
                        let identifier = identifier.child(*id);
                        (identifier, node)
                    }));

                    // See if element was rendered, and remove it if not
                    if let (false, Some(node)) = (
                        used_element_identifiers.contains(&identifier),
                        node.value.as_ref(),
                    ) {
                        // Remove it from the DOM
                        let node = Node::from(node);
                        if let Some(parent) = node.parent_node() {
                            // Possible that parent could've already been removed
                            parent.remove_child(&node);
                        }

                        // Remove it from the tree
                        to_remove.push(identifier);
                    }
                }

                for identifier in to_remove {
                    self.element_tree.remove(identifier);
                }
            }

            // Update the previous identifiers for the next render
            self.previous_identifiers = used_element_identifiers;
        } else {
            console::log_1(&"nothing to render".into());
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ControllerRef(Rc<RefCell<Controller>>);

impl ControllerRef {
    pub fn new<C>(component: C, document: &Document) -> Self
    where
        C: Component + 'static,
    {
        Self(Rc::new(RefCell::new(Controller::new(component, document))))
    }

    /// Render the component, returning the created elements.
    ///
    /// Render must (?) be created on the wrapped struct, as it requires passing the [Rc] to the
    /// [Controller] to closures, such as for the `get_event_closure` callback. This could be
    /// changed if another way of building event closures is used.
    pub fn render(&self, parent: &Element) -> Result<Option<Vec<Element>>, JsValue> {
        self.0.borrow_mut().perform_render(
            self,
            Some(RenderedNode::Element(parent.clone())),
            RenderType::Root,
            &Identifier::new(),
        )?;

        Ok(None)
    }
}
