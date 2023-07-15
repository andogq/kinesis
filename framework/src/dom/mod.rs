use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{console, Document, Element};

use crate::component::EventType;

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

pub enum Content<T> {
    Static(T),
    Dynamic {
        /// List of values that this content relies on to render. Indexes based off of the order
        /// they are defined on the component struct.
        dependencies: Vec<usize>,

        /// A handler that will be passed a list of dependencies, and will render the content
        /// appropriately.
        // TODO: Handler should take referenced to changed values, which will be dynamic
        render: Box<dyn Fn(&isize) -> T>,
    },
}

pub enum NodeContent {
    Text(Content<String>),
    Nodes(Vec<DomNode>),
}

#[derive(Default)]
pub struct DomNode {
    id: usize,
    kind: DomNodeKind,
    content: Option<NodeContent>,
    listeners: Vec<EventType>,
}
impl DomNode {
    pub fn p(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::P,
            ..Default::default()
        }
    }

    pub fn button(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::Button,
            ..Default::default()
        }
    }

    pub fn div(id: usize) -> Self {
        Self {
            id,
            kind: DomNodeKind::Div,
            ..Default::default()
        }
    }

    pub fn text_content(mut self, content: Content<String>) -> Self {
        self.content = Some(NodeContent::Text(content));
        self
    }

    pub fn children(mut self, children: Vec<DomNode>) -> Self {
        self.content = Some(NodeContent::Nodes(children));
        self
    }

    pub fn child(mut self, child: DomNode) -> Self {
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
        counter: isize,
        element: Option<Element>,
        get_closure: &F,
    ) -> Result<(Element, Option<Vec<Self>>), JsValue>
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

        let children = match self.content {
            Some(NodeContent::Text(text_content)) => {
                // TODO: (somehow) only change these things if their dependencies have altered
                let text_content = match text_content {
                    Content::Static(content) => content,
                    Content::Dynamic {
                        dependencies,
                        render,
                    } => {
                        // TODO: Somehow extract actual args
                        render(&counter)
                    }
                };

                console::log_1(&text_content.clone().into());

                element.set_text_content(Some(text_content.as_str()));

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

        Ok((element, children))
    }
}
