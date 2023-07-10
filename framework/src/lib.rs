use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{console, window, Document, Element, MouseEvent};

enum DomNodeKind {
    Div,
    P,
}
impl Default for DomNodeKind {
    fn default() -> Self {
        Self::Div
    }
}
impl From<DomNodeKind> for &str {
    fn from(dom_node_kind: DomNodeKind) -> &'static str {
        match dom_node_kind {
            DomNodeKind::Div => "div",
            DomNodeKind::P => "p",
        }
    }
}

#[derive(Default)]
struct DomNode {
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

#[derive(Default)]
struct Counter {
    count: usize,
}
impl Counter {
    pub fn handle_event(&mut self) {
        self.count += 1;
    }

    pub fn render(&self) -> Option<DomNode> {
        Some(DomNode::p().text_content(&format!("The current count is {}", self.count)))
    }
}

struct Scheduler {
    document: Document,

    running: bool,
    events: VecDeque<usize>,
    components: Vec<(Counter, Box<dyn Fn(&Document, &Counter)>)>,
}
impl Scheduler {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            running: false,
            events: VecDeque::new(),
            components: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        while let Some(event) = self.events.pop_front() {
            console::log_1(&format!("running event {event}").into());

            // Find element matching id
            if let Some((ref mut component, render)) = self.components.get_mut(event) {
                component.handle_event();
                render(&self.document, component);
            }
        }

        self.running = false;
    }

    pub fn add_event(&mut self, event: usize) {
        self.events.push_back(event);

        if !self.running {
            self.run();
        }
    }

    pub fn add_component(
        &mut self,
        component: Counter,
        render: Box<dyn Fn(&Document, &Counter)>,
    ) -> usize {
        let id = self.components.len();
        self.components.push((component, render));
        id
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have body");

    let scheduler_rc = Rc::new(RefCell::new(Scheduler::new(document)));
    let mut scheduler = scheduler_rc.borrow_mut();

    let counter = Counter::default();
    let id = scheduler.add_component(counter, {
        let scheduler_rc = Rc::clone(&scheduler_rc);

        Box::new(move |document, counter| {
            if let Some(el) = counter.render() {
                let el = el.build(document).unwrap();

                let scheduler = Rc::clone(&scheduler_rc);
                let callback = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
                    // TODO: Work out how to get message out from here to trigger a re-render
                    console::log_1(&"click".into());

                    scheduler.borrow_mut().add_event(0);
                });

                el.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref());
                body.append_child(&el.into());

                callback.forget();
            }
        })
    });

    // Jump start initial render
    scheduler.add_event(0);

    Ok(())
}
