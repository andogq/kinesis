use std::collections::VecDeque;
use web_sys::{console, window, Document, Element, MouseEvent, Node};

use crate::counter::Counter;

pub struct Scheduler {
    document: Document,

    running: bool,
    events: VecDeque<usize>,
    components: Vec<(
        Counter,
        Option<Element>,
        Box<dyn Fn(&Document, &Counter, Option<Element>) -> Element>,
    )>,
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
            if let Some((ref mut component, target, render)) = self.components.get_mut(event) {
                component.handle_event();
                *target = Some(render(&self.document, component, target.take()));
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
        render: Box<dyn Fn(&Document, &Counter, Option<Element>) -> Element>,
    ) -> usize {
        let id = self.components.len();
        self.components.push((component, None, render));
        id
    }
}
