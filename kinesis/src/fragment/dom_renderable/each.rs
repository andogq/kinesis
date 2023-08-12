use std::rc::Rc;

use super::DomRenderable;
use crate::fragment::{Fragment, FragmentBuilder, Location, RegisterEventFn};
use web_sys::{Document, Node as WsNode};

/// A function that returns an [`Iterator`] of [`FragmentBuilder`]s, for the given context.
pub type GetItemsFn<Ctx> = Box<dyn Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>>;

/// Use to create multiple fragments depending on the current context. Useful for creating a
/// [`Fragment`] for each item in an [`Iterator`].
pub struct Each<Ctx> {
    /// A reference to [`Document`], which is required in order to create new [`Fragment`]s.
    document: Document,

    /// A function that will return an [`Iterator`] of [`FragmentBuilder`]s for the given context.
    get_items: GetItemsFn<Ctx>,

    /// If there are mounted fragments, their references will be contained here. This is primarily
    /// to allow for proper detaching of the [`Fragment`]s.
    mounted_fragments: Option<Vec<Fragment<Ctx>>>,

    /// A reference to an anchor within the DOM. Items within the iterator will be rendered at this
    /// location.
    anchor: WsNode,

    /// A callback capable of passing events back up to the controller.
    register_event: RegisterEventFn,
}

impl<Ctx> Each<Ctx>
where
    Ctx: 'static,
{
    /// Create a new each block with the provided `get_items` function. Requires a reference to
    /// [`Document`] in order to clone and store it for future use.
    pub fn new(
        document: &Document,
        get_items: GetItemsFn<Ctx>,
        register_event: RegisterEventFn,
    ) -> Self {
        Self {
            document: document.clone(),
            get_items,
            mounted_fragments: None,
            anchor: document.create_text_node("").into(),
            register_event,
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

impl<Ctx> DomRenderable<Ctx> for Each<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &Location) {
        location.mount(&self.anchor);
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        // Detach all current mounted fragments (top level as their parent won't be removed)
        self.detach_fragments(true);

        // Create new fragments
        self.mounted_fragments = Some(
            (self.get_items)(context)
                .map(|builder| {
                    let mut fragment =
                        builder.build(&self.document, Rc::clone(&self.register_event));

                    fragment.mount(&Location::anchor(&self.anchor));
                    fragment.update(context, changed);

                    fragment
                })
                .collect(),
        );
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
