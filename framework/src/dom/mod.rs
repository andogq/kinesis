use std::rc::Rc;

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{console, Document, Element};

use crate::component::{DependencyRegistrationCallback, EventType};

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

/// Represents the possible variations of text content within an element.
pub enum TextContent<UpdateType> {
    /// Static content that will not change. Does not contain any interpolated variables.
    Static(String),

    /// Dynamic content that contains interpolated variables. Will be re-rendered when the
    /// variables inside the content changes.
    Dynamic {
        /// List of values that this content relies on to render. Indexes based off of the order
        /// they are defined on the component struct.
        dependencies: Vec<usize>,

        /// A handler that will be passed a list of dependencies, and will render the content
        /// appropriately.
        update_type: UpdateType,
    },
}

/// Possible node content types within a DOM node.
pub enum NodeContent<UpdateType> {
    /// Text content, eg within `p` and `h1` elements.
    Text(TextContent<UpdateType>),

    /// Other nested nodes.
    Nodes(Vec<DomNode<UpdateType>>),
}

/// Generated representation of dynamic content within a component. Contains all of the required
/// information to detect a changed dependency, and trigger a re-render.
pub struct DynamicContent<UpdateType> {
    pub dependencies: Vec<usize>,
    pub update_type: UpdateType,
    pub callback: DependencyRegistrationCallback,
}

/// Information returned after a DOM node is build. Includes the element that it was rendered in,
/// as well as any children (that will need to be rendered), and a list of dynamic content within
/// the component.
pub struct DomNodeBuildResult<UpdateType> {
    /// Element that the node was rendered into.
    pub element: Element,

    /// A list of children that will need to be rendered within the element.
    pub children: Option<Vec<DomNode<UpdateType>>>,

    /// Any dynamic content that needs to be rendered within the component.
    pub dynamic_content: Vec<DynamicContent<UpdateType>>,
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

    pub fn text_content(mut self, content: TextContent<UpdateType>) -> Self {
        self.content = Some(NodeContent::Text(content));
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
    pub fn build<F>(
        self,
        document: &Document,
        element: Option<Element>,
        get_closure: &F,
    ) -> Result<DomNodeBuildResult<UpdateType>, JsValue>
    where
        F: Fn(usize, EventType) -> Function,
    {
        // If no existing element, create a new one
        let element = element.unwrap_or_else(|| {
            console::log_1(&"creating element".into());
            document
                .create_element(self.kind.into())
                .expect("to be able to create element")
        });

        let mut dynamic_content = Vec::<DynamicContent<UpdateType>>::new();

        let children = match self.content {
            Some(NodeContent::Text(text_content)) => {
                match text_content {
                    TextContent::Static(content) => {
                        element.set_text_content(Some(content.as_str()));
                    }
                    TextContent::Dynamic {
                        dependencies,
                        update_type,
                    } => dynamic_content.push(DynamicContent {
                        dependencies,
                        update_type,
                        callback: {
                            let element = element.clone();
                            Rc::new(move |content| {
                                element.set_text_content(Some(content.as_str()));
                            })
                        },
                    }),
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

        Ok(DomNodeBuildResult {
            element,
            children,
            dynamic_content,
        })
    }
}
