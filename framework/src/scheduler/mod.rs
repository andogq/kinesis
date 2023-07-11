use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Element, MouseEvent};

use crate::component::Component;

struct ComponentEntry {
    component: Box<dyn Component>,
    element: Option<Element>,
}

/// Helper type for [Scheduler] for berevity.
pub type SchedulerRef = Rc<RefCell<Scheduler>>;

/// Main scheduler of the framework. Queues and executes events, and is
/// the owner of any [components](Component) used within the framework.
/// Depending on events that are generated, it will trigger handlers on
/// the components and cause them to be re-rendered.
pub struct Scheduler {
    /// Optional self reference to the same instance of the scheduler.
    /// This allow for easy sharing of the scheduler to components as
    /// they are created.
    ///
    /// TODO: Need to work out if this is even required.
    self_ref: Option<SchedulerRef>,

    /// Ownership to [Document] for the current page, providing the
    /// scheduler with full control to create and manipulate elements.
    document: Document,

    /// Indicates whether the scheduler is running or not. Events are
    /// executed in a blocking fasion until they are exhausted, causing
    /// the scheduler to stop once again. This allows for easy tracking
    /// of whether the scheduler should be started again.
    running: bool,
    /// Queue of events to execute. [VecDeque] allows for events to be
    /// executed in the order they are pushed onto the queue (by
    /// popping them off).
    events: VecDeque<usize>,
    /// Location of all of the components within the application,
    /// providing the scheduler with unrestricted access to them without
    /// mutability or ownership concerns.
    components: Vec<ComponentEntry>,

    /// Used to cache a universal callback for DOM events
    cached_callback: Option<Closure<dyn Fn(MouseEvent)>>,
}

impl Scheduler {
    /// Provides a new [Scheduler] **without** a reference to itself.
    pub fn new(document: Document) -> Self {
        Self {
            self_ref: None,
            document,
            running: false,
            events: VecDeque::new(),
            components: Vec::new(),
            cached_callback: None,
        }
    }

    /// Provides a new [Scheduler] with a reference to itself.
    pub fn new_with_ref(document: Document) -> SchedulerRef {
        let scheduler = Self::new(document);
        let rc = Rc::new(RefCell::new(scheduler));

        let moveable_rc = Rc::clone(&rc);
        rc.borrow_mut().self_ref = Some(moveable_rc);

        rc
    }

    pub fn run(&mut self) {
        // Early return to eliminate any chance of infinite loops.
        if self.running {
            return;
        }

        while let Some(event) = self.events.pop_front() {
            console::log_1(&format!("running event {event}").into());

            // Find element matching id
            if let Some(ComponentEntry { component, .. }) = self.components.get_mut(event) {
                component.handle_event();
            }
        }

        // Rerender the page
        self.render().unwrap();

        self.running = false;
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let callback = self.cached_callback.get_or_insert({
            // Create the callback, save it, and use it
            // WARN: This will only work for MouseEvents, need to extend to work for any kind of
            // event
            let scheduler = Rc::clone(self.self_ref.as_ref().expect("to have self ref"));

            Closure::<dyn Fn(_)>::new(move |_event: MouseEvent| {
                // Trigger event within scheduler
                // TODO: Work out an event object
                scheduler.borrow_mut().add_event(0);
            })
        });

        for ComponentEntry { component, element } in self.components.iter_mut() {
            if let Some((node, listener_map)) = component.render() {
                // Convert node to Element
                let el = node.build(&self.document)?;

                // Apply required event listeners
                for listener in listener_map {
                    el.add_event_listener_with_callback(
                        &listener.event_type,
                        callback.as_ref().unchecked_ref(),
                    )?;
                }

                // Add to the DOM
                if let Some(element) = element.take() {
                    // Update an existing element
                    element.replace_with_with_node_1(&el)?;
                } else {
                    // Create a new element in the DOM
                    let body = self.document.body().expect("body to exist");
                    body.append_child(&el)?;
                }

                // Save the newly created element for future reference
                *element = Some(el);
            }
        }

        Ok(())
    }

    pub fn add_event(&mut self, event: usize) {
        self.events.push_back(event);

        self.run();
    }

    pub fn add_component(&mut self, component: Box<dyn Component>) -> usize {
        let id = self.components.len();
        self.components.push(ComponentEntry {
            component,
            element: None,
        });
        id
    }
}
