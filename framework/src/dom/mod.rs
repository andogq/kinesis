use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element};

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

#[derive(Default)]
pub struct DomNode {
    id: usize,
    kind: DomNodeKind,
    text_content: Option<String>,
    children: Vec<DomNode>,
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

    pub fn text_content(mut self, content: &str) -> Self {
        self.text_content = Some(content.to_string());
        self
    }

    pub fn children(mut self, children: Vec<DomNode>) -> Self {
        self.children = children;
        self
    }

    pub fn child(mut self, child: DomNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn listen(mut self, event: EventType) -> Self {
        self.listeners.push(event);
        self
    }

    pub fn build<F>(self, document: &Document, get_closure: &F) -> Result<Element, JsValue>
    where
        F: Fn(usize, EventType) -> Function,
    {
        let el = document.create_element(self.kind.into())?;

        el.set_text_content(self.text_content.as_deref());

        // Set up event listeners
        for event in self.listeners {
            el.add_event_listener_with_callback(
                &String::from(event),
                &get_closure(self.id, event),
            )?;
        }

        // Add nested children
        for child in self.children {
            el.append_child(&child.build(document, get_closure)?.into())?;
        }

        Ok(el)
    }
}
