use wasm_bindgen::JsValue;
use web_sys::{Document, Element};

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

#[derive(Default)]
pub struct DomNode {
    kind: DomNodeKind,
    text_content: Option<String>,
}
impl DomNode {
    pub fn p() -> Self {
        Self {
            kind: DomNodeKind::P,
            ..Default::default()
        }
    }

    pub fn button() -> Self {
        Self {
            kind: DomNodeKind::Button,
            ..Default::default()
        }
    }

    pub fn text_content(mut self, content: &str) -> Self {
        self.text_content = Some(content.to_string());
        self
    }

    pub fn build(self, document: &Document) -> Result<Element, JsValue> {
        let el = document.create_element(self.kind.into())?;
        el.set_text_content(self.text_content.as_deref());
        Ok(el)
    }
}
