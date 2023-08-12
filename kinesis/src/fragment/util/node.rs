use web_sys::{Document, Node as WsNode};

/// Information required to build a [`web_sys::Node`]. Offers a friendly interface for creating new
/// [`web_sys::Node`]s, and allows for programatic access to certain attributes before creation
/// (namely whether the node is a text node or an element).
pub enum Node {
    /// A [`web_sys::Text`] node. Containing [`String`] refers to the content of the generated text
    /// node, which will be passed to [`Document::create_text_node()`].
    Text(String),

    /// A [`web_sys::Element`] node. Containing [`String`] refers to the element type (eg `p`,
    /// `div`), which will be passed to [`Document::create_element()`].
    Element(String),
}

impl Node {
    /// Create a new [`web_sys::Text`] node with the provided content, casting it to a
    /// [`web_sys::Node`].
    pub fn text<S>(content: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Text(content.as_ref().to_string())
    }

    /// Create a new [`web_sys::Element`] node of the provided type, casting it to a
    /// [`web_sys::Node`].
    pub fn element<S>(kind: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Element(kind.as_ref().to_string())
    }

    /// Build a [`web_sys::Node`] based off of the current node representation. Requires a
    /// reference to [`Document`] in order to call the relevant node creation method on it.
    pub fn create_node(&self, document: &Document) -> WsNode {
        match &self {
            Node::Element(element_kind) => document
                .create_element(element_kind)
                .expect("to create a new element")
                .into(),
            Node::Text(text_content) => document.create_text_node(text_content).into(),
        }
    }
}
