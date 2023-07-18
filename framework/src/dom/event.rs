#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
}

impl From<EventType> for String {
    /// Convert to an event name for use in JS listeners.
    fn from(event: EventType) -> Self {
        use EventType::*;
        match event {
            Click => "click",
        }
        .to_string()
    }
}
