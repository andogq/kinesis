use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element};

use crate::{
    component::RenderType,
    dom::{renderable::RenderedNode, EventType},
};

use super::{Component, Identifier};

pub struct Controller {
    /// The component that this controller is handling.
    component: Box<dyn Component>,

    /// A reference to the document object, so that elements can be created.
    document: Document,
}

impl Controller {
    pub fn new<C>(component: C, document: &Document) -> Self
    where
        C: Component + 'static,
    {
        Self {
            component: Box::new(component) as Box<dyn Component>,
            document: document.clone(),
        }
    }

    fn get_event_callback_closure(
        &mut self,
        identifier: Identifier,
        event_type: EventType,
    ) -> Function {
        todo!()
    }
}

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
        let controller = self.0.borrow();
        let root_identifier = Identifier::new();

        // Perform the initial render of the component, to retrieve all of the renderables
        if let Some(renderables) = controller.component.render(RenderType::Root) {
            // Build a queue of renderables to render, it's identifier, and the parent element it
            // will be rendered into.
            let mut queue = renderables
                .into_iter()
                .enumerate()
                .map(|(i, renderable)| (root_identifier.child(i), renderable, parent.clone()))
                .collect::<VecDeque<_>>();

            while let Some((identifier, renderable, parent)) = queue.pop_front() {
                // Render each of the child renderables
                if let Some(result) =
                    renderable.render(&controller.document, None, &|event_type| {
                        self.0
                            .clone()
                            .borrow_mut()
                            .get_event_callback_closure(identifier.clone(), event_type)
                    })?
                {
                    if let Some(RenderedNode::Element(element)) = &result.element {
                        if let Some(children) = result.children {
                            // Add resulting children to the queue to be rendered
                            queue.extend(children.into_iter().enumerate().map(
                                |(i, renderable)| {
                                    (identifier.child(i), renderable, element.clone())
                                },
                            ));
                        }
                    } else if result.element.is_some() && result.children.is_some() {
                        todo!("determine a better way to pass a rendered container element around");
                    }

                    if let Some(element) = &result.element {
                        parent.append_with_node_1(&element.into())?;
                    }
                }
            }
        }

        Ok(None)
    }
}
