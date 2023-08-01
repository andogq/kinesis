use wasm_bindgen::JsValue;
use web_sys::Element;

use super::renderable::RenderedNode;

/// Possible ways that a [super::renderable::Renderable] may be rendered.
#[derive(Clone)]
pub enum RenderPosition {
    /// Render as an appended child of the provided element.
    Append(RenderedNode),

    /// Render as a prepended child of the provided element.
    Prepend(RenderedNode),

    /// Insert before the provided element.
    Before(RenderedNode),

    /// Insert after the provided element.
    After(RenderedNode),

    /// Replace the provided element.
    Replace(RenderedNode),

    /// Do not render.
    None,
}

/// Matches up each of the [RenderPosition] variants with the relevant methods to all against the
/// method passed to them.
macro_rules! generate_match {
    ($self:ident, $element:ident {
        $none:path,
        $($variant:path: ($method:ident)),*
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
        use RenderedNode::*;

        let element = &element.into();

        match self {
            Append(Element(parent)) => {
                parent.append_child(element);
            }
            Append(Node(parent)) => {
                parent.append_child(element);
            }
            Prepend(Element(parent)) => {
                parent.prepend_with_node_1(element);
            }
            Prepend(Node(parent)) => {
                if let Some(sibling) = parent.first_child() {
                    parent.insert_before(&sibling, Some(element));
                } else {
                    // No children in parent, so just append it
                    parent.append_child(element);
                }
            }
            Before(Element(sibling)) => {
                sibling.before_with_node_1(element);
            }
            Before(Node(sibling)) => {
                sibling
                    .parent_element()
                    .expect("to be able to get a parent element")
                    .insert_before(sibling, Some(element));
            }
            After(Element(parent)) => {
                parent.after_with_node_1(element);
            }
            After(Node(sibling)) => {
                let parent = sibling
                    .parent_element()
                    .expect("to be able to get a parent element");

                if let Some(sibling) = sibling.next_sibling() {
                    parent.insert_before(&sibling, Some(element));
                } else {
                    // Can assume there's no more siblings, so append to parent
                    parent.append_child(element);
                }
            }
            Replace(Element(old_element)) => {
                old_element.replace_with_with_node_1(element);
            }
            Replace(Node(old_element)) => {
                old_element
                    .parent_node()
                    .expect("to be able to get a parent node")
                    .replace_child(old_element, element);
            }
            None => (),
        }

        Ok(())
    }

    pub fn get_element(&self) -> Option<RenderedNode> {
        use RenderPosition::*;

        match self {
            Append(element) | Prepend(element) | Before(element) | After(element)
            | Replace(element) => Some(element.clone()),
            None => Option::None,
        }
    }
}
