use wasm_bindgen::JsValue;
use web_sys::Element;

use super::renderable::RenderedNode;

/// Possible ways that a [super::renderable::Renderable] may be rendered.
#[derive(Clone)]
pub enum RenderPosition {
    /// Render as an appended child of the provided element.
    Append(Element),

    /// Render as a prepended child of the provided element.
    Prepend(Element),

    /// Insert before the provided element.
    Before(Element),

    /// Insert after the provided element.
    After(Element),

    /// Replace the provided element.
    Replace(Element),

    /// Do not render.
    None,
}

/// Matches up each of the [RenderPosition] variants with the relevant methods to all against the
/// method passed to them.
macro_rules! generate_match {
    ($self:ident, $element:ident {
        $none:path,
        $($variant:path: $method:ident),*
    }) => {
        match $self {
            $(
                $variant(target) => {
                    target.$method($element)?;
                    Ok(())
                }
             )*
            $none => Ok(()),
        }
    };
}

impl RenderPosition {
    pub fn render(&self, element: &RenderedNode) -> Result<(), JsValue> {
        use RenderPosition::*;

        let element = &element.into();

        generate_match!(
            self,
            element {
                None,
                Append: append_child,
                Prepend: prepend_with_node_1,
                Before: before_with_node_1,
                After: after_with_node_1,
                Replace: replace_with_with_node_1
            }
        )
    }

    pub fn get_element(&self) -> Option<Element> {
        use RenderPosition::*;

        match self {
            Append(element) | Prepend(element) | Before(element) | After(element)
            | Replace(element) => Some(element.clone()),
            None => Option::None,
        }
    }
}
