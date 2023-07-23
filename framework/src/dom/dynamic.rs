use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::Document;

use super::{
    renderable::{DomNodeBuildResult, DynamicContent, Renderable, RenderedNode},
    EventType,
};

pub struct Dynamic {
    dependencies: Vec<usize>,
    update_type: usize,
}

impl Dynamic {
    pub fn new(update_type: usize) -> Self {
        Self {
            dependencies: Vec::new(),
            update_type,
        }
    }

    pub fn depends_on(mut self, dependency: usize) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

impl Renderable for Dynamic {
    fn render(
        self: Box<Self>,
        document: &Document,
        element: Option<RenderedNode>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        // TODO: Maybe immediately render children?

        Ok(Some(DomNodeBuildResult {
            element: None,
            cache_node: false,
            children: None,
            dynamic_content: vec![DynamicContent {
                dependencies: self.dependencies,
                update_type: self.update_type,
            }],
            in_place: false,
        }))
    }
}
