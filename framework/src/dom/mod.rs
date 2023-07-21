pub mod dynamic;
mod event;
pub mod renderable;
pub mod text;

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{console, Document};

pub use self::event::EventType;
use self::renderable::{DomNodeBuildResult, DynamicContent, Renderable, RenderedNode};

#[derive(Clone, Copy)]
pub enum DomElementKind {
    Div,
    P,
    Button,
}

impl Default for DomElementKind {
    fn default() -> Self {
        Self::Div
    }
}

impl From<DomElementKind> for &str {
    fn from(dom_node_kind: DomElementKind) -> &'static str {
        use DomElementKind::*;

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

pub enum DomContent {
    Element {
        kind: DomElementKind,
        content: Option<Vec<Box<dyn Renderable>>>,
    },
    Text {
        content: String,
    },
}

pub struct DomNode {
    kind: DomElementKind,
    children: Vec<Box<dyn Renderable>>,
    listeners: Vec<EventType>,
}
impl DomNode {
    pub fn p() -> Self {
        Self {
            kind: DomElementKind::P,
            children: Vec::new(),
            listeners: Vec::new(),
        }
    }

    pub fn button() -> Self {
        Self {
            kind: DomElementKind::Button,
            children: Vec::new(),
            listeners: Vec::new(),
        }
    }

    pub fn div() -> Self {
        Self {
            kind: DomElementKind::Div,
            children: Vec::new(),
            listeners: Vec::new(),
        }
    }

    pub fn child(mut self, child: Box<dyn Renderable>) -> Self {
        self.children.push(child);

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
        element: Option<RenderedNode>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        // If no existing element, create a new one
        let element = element
            // Make sure that the cached node is an Element
            .and_then(|element| {
                if let RenderedNode::Element(element) = element {
                    Some(element)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                console::log_1(&"creating element".into());

                document
                    .create_element(self.kind.into())
                    .expect("to be able to create element")
            });

        let mut dynamic_content = Vec::<DynamicContent>::new();

        // let children = match content {
        //     Some(NodeContent::Text(text_content)) => {
        //         match text_content {
        //             TextContent::Static(content) => {
        //                 element.set_text_content(Some(content.as_str()));
        //             }
        //             TextContent::Dynamic {
        //                 dependencies,
        //                 update_type,
        //             } => dynamic_content.push(DynamicContent {
        //                 dependencies,
        //                 update_type,
        //                 callback: {
        //                     let element = element.clone();
        //                     Rc::new(move |content| {
        //                         element.set_text_content(Some(content.as_str()));
        //                     })
        //                 },
        //             }),
        //         };
        //
        //         None
        //     }
        //     Some(NodeContent::Nodes(children)) => Some(children),
        //     _ => None,
        // };

        // Set up event listeners
        for event in self.listeners {
            element.add_event_listener_with_callback(
                &String::from(event),
                &get_event_closure(event),
            )?;
        }

        Ok(Some(DomNodeBuildResult {
            element: Some(RenderedNode::Element(element)),
            cache_node: true,
            children: Some(self.children),
            dynamic_content,
            in_place: false,
        }))
    }
}
