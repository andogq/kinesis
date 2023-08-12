use super::DomRenderable;
use crate::fragment::Location;
use web_sys::{Document, Text};

/// Helper type for updatable text. The closure will be called with the current context, and must
/// return text to be placed within a text node to be rendered.
pub type GetTextFn<Ctx> = Box<dyn Fn(&Ctx) -> String>;

/// Represents a reactive [`Text`] node within a fragment. Will generate a new value for the
/// provided context, and will update the node appropriately.
pub struct Updatable<Ctx> {
    /// Call back function that generates a new [`String`] for the given state.
    get_text: GetTextFn<Ctx>,

    /// Reference to the [`Text`] node that will be inserted and updated in the DOM.
    text_node: Text,
}

impl<Ctx> Updatable<Ctx> {
    /// Create a new instance with the provided `get_text` callback. Requires a reference to
    /// [`Document`] in order to create the [`Text`] node.
    pub fn new(document: &Document, get_text: GetTextFn<Ctx>) -> Self {
        Self {
            get_text,
            text_node: document.create_text_node(""),
        }
    }
}

impl<Ctx> DomRenderable<Ctx> for Updatable<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &Location) {
        location.mount(&self.text_node);
    }

    fn update(&mut self, context: &Ctx, _changed: &[usize]) {
        self.text_node.set_data(&(self.get_text)(context));
    }

    fn detach(&mut self, top_level: bool) {
        if top_level {
            self.text_node
                .parent_node()
                .expect("node to have parent")
                .remove_child(&self.text_node);
        }
    }
}
