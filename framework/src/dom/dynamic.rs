use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::Document;

use crate::component::{Component, RenderType};

use super::{
    renderable::{DomNodeBuildResult, DynamicContent, Renderable, RenderedNode},
    EventType,
};

pub struct Dynamic {
    dependencies: Vec<usize>,
    render_type: usize,
}

/// Renders the update type directly in place. Will automatically re-render when any of the
/// attached dependencies changes.
impl Dynamic {
    pub fn new(render_type: usize) -> Self {
        Self {
            dependencies: Vec::new(),
            render_type,
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
        component: &dyn Component,
        element: Option<RenderedNode>,
        get_event_closure: &dyn Fn(EventType) -> Function,
    ) -> Result<Option<DomNodeBuildResult>, JsValue> {
        // Immediately render the children
        let children = component.render(RenderType::Partial(self.render_type));

        Ok(Some(DomNodeBuildResult {
            element: None,
            cache_node: false,
            children,
            dynamic_content: vec![DynamicContent {
                dependencies: self.dependencies,
                update_type: self.render_type,
            }],
            in_place: false,
        }))
    }
}
