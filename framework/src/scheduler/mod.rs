use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::Document;

use crate::component::{Component, ComponentControllerRef};

/// Helper type for [Scheduler] for berevity.
pub type SchedulerRef = Rc<RefCell<Scheduler>>;

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
    /// Location of all of the components within the application, providing the scheduler with
    /// unrestricted access to them without mutability or ownership concerns.
    components: Vec<ComponentControllerRef>,
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
            components: Vec::new(),
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

        // Rerender the page
        self.render().unwrap();

        self.running = false;
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        for controller in self.components.iter_mut() {
            controller.render()?;
        }

        Ok(())
    }

    pub fn add_component<C>(&mut self, component: C)
    where
        C: Component + 'static,
    {
        let id = self.components.len();
        self.components
            .push(ComponentControllerRef::new(id, component, &self.document));
    }
}
