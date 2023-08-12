use crate::fragment::{Fragment, Location};
use web_sys::{Document, Node as WsNode};

use super::DomRenderable;

/// A function that will return some condition for the given context.
pub type CheckConditionFn<Ctx> = Box<dyn Fn(&Ctx) -> bool>;

/// Use to render a fragment conditionally depending on a reactive contition. Will propagate any
/// context updates to the child [`Fragment`] if it is mounted.
pub struct Conditional<Ctx> {
    /// A function to determine whether the fragment should be mounted, depending on the provided
    /// context.
    check_condition: CheckConditionFn<Ctx>,

    /// The [`Fragment`] to conditionally mount.
    fragment: Fragment<Ctx>,

    /// Reference to the anchor for this fragment in the DOM
    anchor: WsNode,

    /// Used to track whether the fragment is currently mounted or not.
    ///
    /// Could alternatively use [`Fragment::mounted`] instead, however this ensures that there is
    /// no reliance on the internal state of the [`Fragment`].
    fragment_mounted: bool,
}

impl<Ctx> Conditional<Ctx> {
    /// Create a new conditional with the provided `check_condition` function and `fragment`.
    /// Requires a reference to [`Document`] in order to create an anchor.
    pub fn new(
        document: &Document,
        check_condition: CheckConditionFn<Ctx>,
        fragment: Fragment<Ctx>,
    ) -> Self {
        Self {
            check_condition,
            fragment,
            anchor: document.create_text_node("").into(),
            fragment_mounted: false,
        }
    }
}

impl<Ctx> DomRenderable<Ctx> for Conditional<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &Location) {
        location.mount(&self.anchor);
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        let should_mount = (self.check_condition)(context);
        if !self.fragment_mounted && should_mount {
            self.fragment.mount(&Location::anchor(&self.anchor));
            self.fragment.full_update(context);

            self.fragment_mounted = true;
        } else if self.fragment_mounted && !should_mount {
            // Top level because parent won't be removed
            self.fragment.detach(true);

            self.fragment_mounted = false;
        } else if self.fragment_mounted {
            self.fragment.update(context, changed);
        }
    }

    fn detach(&mut self, top_level: bool) {
        self.anchor
            .parent_node()
            .expect("node to have parent")
            .remove_child(&self.anchor);

        if self.fragment_mounted {
            self.fragment.detach(top_level);
            self.fragment_mounted = false;
        }
    }
}
