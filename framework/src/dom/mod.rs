mod event;
pub mod renderable;

use std::rc::Rc;

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{console, Document, Element};

pub use self::event::EventType;
use self::renderable::{DomNodeBuildResult, DynamicContent, Renderable};

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
pub enum TextContent {
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
        update_type: usize,
    },
}

/// Possible node content types within a DOM node.
pub enum NodeContent {
    /// Text content, eg within `p` and `h1` elements.
    Text(TextContent),

    /// Other nested nodes.
    Nodes(Vec<Box<dyn Renderable>>),
}

pub struct DomNode {
    kind: DomNodeKind,
    content: Option<NodeContent>,
    listeners: Vec<EventType>,
}
impl DomNode {
    pub fn p() -> Self {
        Self {
            kind: DomNodeKind::P,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn button() -> Self {
        Self {
            kind: DomNodeKind::Button,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn div() -> Self {
        Self {
            kind: DomNodeKind::Div,
            content: None,
            listeners: Vec::new(),
        }
    }

    pub fn text_content(mut self, content: TextContent) -> Self {
        self.content = Some(NodeContent::Text(content));
        self
    }

    pub fn child(mut self, child: Box<dyn Renderable>) -> Self {
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
}

impl Renderable for DomNode {
    fn render(
        self: Box<Self>,
        document: &Document,
        element: Option<Element>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        // If no existing element, create a new one
        let element = element.unwrap_or_else(|| {
            console::log_1(&"creating element".into());
            document
                .create_element(self.kind.into())
                .expect("to be able to create element")
        });

        let mut dynamic_content = Vec::<DynamicContent>::new();

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
                &get_event_closure(event),
            )?;
        }

        Ok(Some(DomNodeBuildResult {
            element: Some(element),
            children,
            dynamic_content,
            in_place: false,
        }))
    }
}
