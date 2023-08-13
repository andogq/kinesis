use super::RegisterEventFn;
use js_sys::Function;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{console, Event};

/// A registry of [`js_sys::Function`]s, caching created closures for a given event id.
pub struct EventRegistry {
    /// Cached closures.
    closures: HashMap<usize, Function>,

    /// Shared reference to a callback function, which will be called when one of the closures is
    /// called.
    register_event: RegisterEventFn,
}

impl EventRegistry {
    /// Create a new registry, returning a shared reference.
    pub fn new<F>(register_event: F) -> Rc<RefCell<Self>>
    where
        F: 'static + Fn(usize),
    {
        Rc::new(RefCell::new(Self {
            closures: HashMap::new(),
            register_event: Rc::new(register_event),
        }))
    }

    /// Get or create a closure for the provided event id.
    pub fn get(&mut self, event_id: usize) -> &Function {
        self.closures.entry(event_id).or_insert_with(|| {
            let register_event = Rc::clone(&self.register_event);
            console::log_1(&"creating event".into());
            Closure::<dyn Fn(Event)>::new(move |event| {
                register_event(event_id);
            })
            .into_js_value()
            .unchecked_into()
        })
    }
}
