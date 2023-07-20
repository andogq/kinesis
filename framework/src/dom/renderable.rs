use std::rc::Rc;

use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, Node};

use super::EventType;

pub type DependencyRegistrationCallback = Rc<dyn Fn(String)>;

/// Generated representation of dynamic content within a component. Contains all of the required
/// information to detect a changed dependency, and trigger a re-render.
pub struct DynamicContent {
    pub dependencies: Vec<usize>,
    pub update_type: usize,
    pub callback: DependencyRegistrationCallback,
}

/// Allows for DOM nodes and elements to be stored together, without having to cast between them as
/// JS does.
#[derive(Clone)]
pub enum RenderedNode {
    Node(Node),
    Element(Element),
}

impl Into<web_sys::Node> for &RenderedNode {
    fn into(self) -> web_sys::Node {
        use RenderedNode::*;

        match self {
            Node(node) => node.clone().into(),
            Element(element) => element.clone().into(),
        }
    }
}

/// Information returned after a DOM node is build. Includes the element that it was rendered in,
/// as well as any children (that will need to be rendered), and a list of dynamic content within
/// the component.
pub struct DomNodeBuildResult {
    /// Element that the node was rendered into.
    pub element: Option<RenderedNode>,

    /// Whether to store the returned node, and pass it back in on the next render or update. This
    /// is useful in situations where the node can be directly editted in place.
    pub cache_node: bool,

    /// A list of children that will need to be rendered within the element.
    pub children: Option<Vec<Box<dyn Renderable>>>,

    /// Any dynamic content that needs to be rendered within the component.
    pub dynamic_content: Vec<DynamicContent>,

    /// Indicates that the following result should be evaluated in place.
    pub in_place: bool,
}

/// Represents anything that could be rednered in th eDOM
pub trait Renderable {
    /// Builds (or updates in place) the current node. Will not build children.
    fn render(
        self: Box<Self>,
        document: &Document,
        element: Option<RenderedNode>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue>;
}

impl<I> Renderable for I
where
    I: IntoIterator<Item = Box<dyn Renderable>>,
{
    fn render(
        self: Box<Self>,
        _document: &Document,
        element: Option<RenderedNode>,
        _get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        Ok(Some(DomNodeBuildResult {
            element,
            cache_node: false,
            children: Some(self.into_iter().collect()),
            dynamic_content: Vec::new(),
            in_place: true,
        }))
    }
}