use std::{cell::RefCell, rc::Rc};

use super::{DomRenderable, DomUpdatable};
use crate::fragment::{EventRegistry, Fragment, FragmentBuilder, Location};
use web_sys::{Document, Node as WsNode};

/// A function that returns an [`Iterator`] of [`FragmentBuilder`]s, for the given context.
pub type GetIterFn<Ctx> =
    Box<dyn Fn(&Ctx) -> Box<dyn std::iter::Iterator<Item = FragmentBuilder<Ctx>>>>;

/// Generates [`Fragment`]s for each item of an [`std::iter::Iterator`] dynamically.
pub struct Iterator<Ctx> {
    /// A reference to [`Document`], which is required in order to create new [`Fragment`]s.
    document: Document,

    /// A function that will return an [`std::iter::Iterator`] of [`FragmentBuilder`]s for the
    /// given context.
    get_iter: GetIterFn<Ctx>,

    /// If there are mounted fragments, their references will be contained here. This is primarily
    /// to allow for proper detaching of the [`Fragment`]s.
    mounted_fragments: Option<Vec<Fragment<Ctx>>>,

    /// A reference to an anchor within the DOM. Items within the iterator will be rendered at this
    /// location.
    anchor: WsNode,

    event_registry: Rc<RefCell<EventRegistry>>,
}

impl<Ctx> Iterator<Ctx> {
    /// Create a new iterator with the provided `get_iter` function. Requires a reference to
    /// [`Document`] in order to clone and store it for future use.
    pub fn new(
        document: &Document,
        get_iter: GetIterFn<Ctx>,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Self {
        Self {
            document: document.clone(),
            get_iter,
            mounted_fragments: None,
            anchor: document.create_text_node("").into(),
            event_registry: Rc::clone(event_registry),
        }
    }

    /// Helper function to trigger each of the mounted [`Fragment`]s to detach, propagating
    /// `top_level` through.
    fn detach_fragments(&mut self, top_level: bool) {
        self.mounted_fragments
            .take()
            .into_iter()
            .flatten()
            .for_each(|mut fragment| fragment.detach(top_level));
    }
}

impl<Ctx> DomRenderable for Iterator<Ctx> {
    fn mount(&mut self, location: &Location) {
        location.mount(&self.anchor);
    }

    fn detach(&mut self, top_level: bool) {
        self.detach_fragments(top_level);

        self.anchor
            .parent_node()
            .expect("node to have parent")
            .remove_child(&self.anchor)
            .expect("to remove child");
    }
}

impl<Ctx> DomUpdatable<Ctx> for Iterator<Ctx>
where
    Ctx: 'static,
{
    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        // Detach all current mounted fragments (top level as their parent won't be removed)
        self.detach_fragments(true);

        // Create new fragments
        self.mounted_fragments = Some(
            (self.get_iter)(context)
                .map(|builder| {
                    let mut fragment = builder.build(&self.document, &self.event_registry);

                    fragment.mount(&Location::anchor(&self.anchor));
                    fragment.update(context, changed);

                    fragment
                })
                .collect(),
        );
    }
}
