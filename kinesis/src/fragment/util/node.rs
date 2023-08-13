use std::{cell::RefCell, rc::Rc};

use web_sys::{Document, Node as WsNode};

use crate::fragment::EventRegistry;

/// Information required to build a [`web_sys::Node`]. Offers a friendly interface for creating new
/// [`web_sys::Node`]s, and allows for programatic access to certain attributes before creation
/// (namely whether the node is a text node or an element).
pub enum NodeType {
    /// A [`web_sys::Text`] node. Containing [`String`] refers to the content of the generated text
    /// node, which will be passed to [`Document::create_text_node()`].
    Text(String),

    /// A [`web_sys::Element`] node. Containing [`String`] refers to the element type (eg `p`,
    /// `div`), which will be passed to [`Document::create_element()`].
    Element(String),
}

pub struct Node {
    node_type: NodeType,
    events: Vec<(String, usize)>,
}

impl Node {
    /// Create a new [`web_sys::Text`] node with the provided content, casting it to a
    /// [`web_sys::Node`].
    pub fn text<S>(content: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            node_type: NodeType::Text(content.as_ref().to_string()),
            events: Vec::new(),
        }
    }

    /// Create a new [`web_sys::Element`] node of the provided type, casting it to a
    /// [`web_sys::Node`].
    pub fn element<S>(kind: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            node_type: NodeType::Element(kind.as_ref().to_string()),
            events: Vec::new(),
        }
    }

    pub fn with_event<S>(mut self, event_type: S, event_id: usize) -> Self
    where
        S: AsRef<str>,
    {
        self.events
            .push((event_type.as_ref().to_string(), event_id));
        self
    }

    /// Build a [`web_sys::Node`] based off of the current node representation. Requires a
    /// reference to [`Document`] in order to call the relevant node creation method on it.
    pub fn create_node(
        &self,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> WsNode {
        let node: WsNode = match &self.node_type {
            NodeType::Element(element_kind) => document
                .create_element(element_kind)
                .expect("to create a new element")
                .into(),
            NodeType::Text(text_content) => document.create_text_node(text_content).into(),
        };

        self.events.iter().for_each(|(event_type, event_id)| {
            node.add_event_listener_with_callback(
                event_type,
                event_registry.borrow_mut().get(*event_id),
            )
            .expect("to bind listener");
        });

        node
    }
}
