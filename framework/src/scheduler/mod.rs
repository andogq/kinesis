use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, Document, Event, HtmlElement};

use crate::component::{Component, ComponentController};

/// Helper type for [Scheduler] for berevity.
pub type SchedulerRef = Rc<RefCell<Scheduler>>;

/// Closure type that is used to handle DOM events, and trigger relevant events within the
/// scheduler.
pub type EventCallbackClosure = Closure<dyn Fn(Event)>;

/// Main scheduler of the framework. Queues and executes events, and is the owner of any
/// [components](Component) used within the framework. Depending on events that are generated, it
/// will trigger handlers on the components and cause them to be re-rendered.
pub struct Scheduler {
    /// Optional self reference to the same instance of the scheduler. This allow for easy sharing
    /// of the scheduler to components as they are created.
    ///
    /// TODO: Need to work out if this is even required.
    self_ref: Option<SchedulerRef>,

    /// Ownership to [Document] for the current page, providing the scheduler with full control to
    /// create and manipulate elements.
    document: Document,

    /// Indicates whether the scheduler is running or not. Events are executed in a blocking fasion
    /// until they are exhausted, causing the scheduler to stop once again. This allows for easy
    /// tracking of whether the scheduler should be started again.
    running: bool,
    /// Queue of events to execute. [VecDeque] allows for events to be executed in the order they
    /// are pushed onto the queue (by popping them off).
    events: VecDeque<usize>,
    /// Location of all of the components within the application, providing the scheduler with
    /// unrestricted access to them without mutability or ownership concerns.
    components: Vec<ComponentController<Box<dyn Component>>>,

    /// Used to cache a universal callback for DOM events
    cached_callback: Option<EventCallbackClosure>,
}

impl Scheduler {
    /// Provides a new [Scheduler] **without** a reference to itself. Note that this variation
    /// cannot be used to create components, as there is no way for events to propagate back to the
    /// scheduler.
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
            if let Some(controller) = self.components.get_mut(event) {
                controller.handle_event();
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

            Closure::<dyn Fn(_)>::new(move |event: Event| {
                // Trigger event within scheduler. When an event is received, the target is
                // checked, and the ID is extracted from an attribute. This ID contains information
                // about the component that the element belongs to, as well as the specific
                // location of the element. This can be used to a) trigger the event on the
                // relevant component, and b) allow the component controller to trigger the correct
                // handler based off of the event listener in the markup.
                let target = event
                    .target()
                    .expect("event to have target")
                    .dyn_into::<HtmlElement>()
                    .expect("to be an element");
                let id = target
                    .dataset()
                    .get("elementId")
                    .expect("`data-element-id` attribute to exist")
                    .split('.')
                    .map(|n| n.parse::<usize>())
                    .collect::<Result<Vec<_>, _>>()
                    .expect("valid usize id stored in attribute");

                // TODO: Emit rest of ID with event
                scheduler.borrow_mut().add_event(id[0]);
            })
        });

        for controller in self.components.iter_mut() {
            controller.render(callback)?;
        }

        Ok(())
    }

    pub fn add_event(&mut self, event: usize) {
        self.events.push_back(event);

        self.run();
    }

    pub fn add_component(&mut self, component: Box<dyn Component>) {
        let id = self.components.len();
        self.components
            .push(ComponentController::new(id, component, &self.document));
    }
}
