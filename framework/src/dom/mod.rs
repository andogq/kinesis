use std::rc::Rc;

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{console, Document, Element};

use crate::component::{Component, DependencyRegistrationCallback, EventType};

#[derive(Clone, Copy)]
pub enum DomNodeKind {
    Div,
    P,
    Button,
}

impl Default for DomNodeKind {
    fn default() -> Self {
        Self::Div
    }
}

impl From<DomNodeKind> for &str {
    fn from(dom_node_kind: DomNodeKind) -> &'static str {
        use DomNodeKind::*;

        match dom_node_kind {
            Div => "div",
            P => "p",
            Button => "button",
        }
    }
}

pub enum Content<UpdateType> {
    Static(String),
    Dynamic {
        /// List of values that this content relies on to render. Indexes based off of the order
        /// they are defined on the component struct.
        dependencies: Vec<usize>,

        /// A handler that will be passed a list of dependencies, and will render the content
        /// appropriately.
        // TODO: Handler should take referenced to changed values, which will be dynamic
        update_type: UpdateType,
    },
}

pub enum NodeContent<UpdateType> {
    Text(Content<UpdateType>),
    Nodes(Vec<DomNode<UpdateType>>),
}

pub struct DomNode<UpdateType> {
    id: usize,
    kind: DomNodeKind,
    content: Option<NodeContent<UpdateType>>,
    listeners: Vec<EventType>,
}
impl<UpdateType> DomNode<UpdateType>
where
    UpdateType: Clone,
{
    pub fn p(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::P,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn button(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::Button,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn div(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::Div,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn text_content(mut self, content: Content<UpdateType>) -> Self {
        self.content = Some(NodeContent::Text(content));
        self
    }

    pub fn children(mut self, children: Vec<DomNode<UpdateType>>) -> Self {
        self.content = Some(NodeContent::Nodes(children));
        self
    }

    pub fn child(mut self, child: DomNode<UpdateType>) -> Self {
        // Make sure that self.content is Some(NodeContent::Nodes(_))
        let children = if let NodeContent::Nodes(children) =
            self.content.get_or_insert(NodeContent::Nodes(Vec::new()))
        {
            children
        } else {
            todo!("properly handle setting children when in a text node");
        };

        children.push(child);

        self
    }

    pub fn listen(mut self, event: EventType) -> Self {
        self.listeners.push(event);
        self
    }

    /// Builds (or updates in place) the current node. Will not build children.
    pub fn build<F, UpdateClosure>(
        self,
        document: &Document,
        element: Option<Element>,
        get_closure: &F,
        request_update: &UpdateClosure,
    ) -> Result<
        (
            Element,
            Option<Vec<Self>>,
            Vec<(Vec<usize>, (UpdateType, DependencyRegistrationCallback))>,
        ),
        JsValue,
    >
    where
        F: Fn(usize, EventType) -> Function,
        UpdateClosure: Fn(UpdateType) -> Option<String>,
    {
        // If no existing element, create a new one
        let element = element.unwrap_or_else(|| {
            console::log_1(&"creating element".into());
            document
                .create_element(self.kind.into())
                .expect("to be able to create element")
        });

        let mut dynamic_content =
            Vec::<(Vec<usize>, (UpdateType, DependencyRegistrationCallback))>::new();

        let children = match self.content {
            Some(NodeContent::Text(text_content)) => {
                // TODO: (somehow) only change these things if their dependencies have altered
                match text_content {
                    Content::Static(content) => {
                        element.set_text_content(Some(content.as_str()));
                    }
                    Content::Dynamic {
                        dependencies,
                        update_type,
                    } => {
                        dynamic_content.push((
                            dependencies,
                            (update_type.clone(), {
                                let element = element.clone();
                                Rc::new(move |content| {
                                    console::log_1(&format!("Adding content {content}").into());

                                    element.set_text_content(Some(content.as_str()));
                                })
                            }),
                        ));

                        // TODO: Somehow extract actual args
                        request_update(update_type);
                    }
                };

                None
            }
            Some(NodeContent::Nodes(children)) => Some(children),
            _ => None,
        };

        // Set up event listeners
        for event in self.listeners {
            element.add_event_listener_with_callback(
                &String::from(event),
                &get_closure(self.id, event),
            )?;
        }

        Ok((element, children, dynamic_content))
    }
}
