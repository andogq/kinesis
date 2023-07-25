use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::Document;

use crate::component::Component;

use super::{
    renderable::{DomNodeBuildResult, Renderable, RenderedNode},
    EventType,
};

pub struct Text(String);

impl Text {
    pub fn new<S>(value: S) -> Self
    where
        S: AsRef<str>,
    {
        Self(value.as_ref().to_string())
    }
}

impl Renderable for Text {
    fn render(
        self: Box<Self>,
        document: &Document,
        _component: &dyn Component,
        element: Option<RenderedNode>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        let node = document.create_text_node(&self.0);

        Ok(Some(DomNodeBuildResult {
            element: Some(RenderedNode::Node(node.into())),
            cache_node: false,
            children: None,
            dynamic_content: Vec::new(),
            in_place: false,
        }))
    }
}

impl<S> From<S> for Text
where
    S: AsRef<str>,
{
    fn from(text: S) -> Self {
        Self(text.as_ref().to_string())
    }
}
