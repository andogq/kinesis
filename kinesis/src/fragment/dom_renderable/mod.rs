mod iterator;

use super::Location;
pub use iterator::*;

/// Used to implement things that can be rendered within the DOM. Must include the required
/// functionality to mount nodes to the provided position, update itself as a result of any state
/// cahnges, and to properly clean up after itself when detached from the DOM.
pub trait DomRenderable<Ctx>
where
    Ctx: 'static,
{
    /// Mount self to the DOM as described by `location`. It is guarenteed that this method will
    /// only be called if not currently mounted, so this does not need to be checked. It also
    /// contains a `register_event` function, which can be used to propagate events back up to the
    /// controller.
    fn mount(&mut self, location: &Location);

    /// Update self due to a state change. Identifiers corresponding to the changed fields will be
    /// included as `changed`, however these should only be used to propagate changes to child
    /// [`super::fragment::Fragment`]s.
    ///
    /// Ideally, `changed` should not be included, and renderables should (somehow) register
    /// sub-fragments to the original controller, to avoid this implemenetation detail.
    fn update(&mut self, context: &Ctx, changed: &[usize]);

    /// Detach self from the DOM. Should result in everying mounted in [`Self::mount()`] being
    /// unmounted. `top_level` indicates that this item is at the top level of the item being
    /// detached, indicating that it must be detached. If something is a child of one of the
    /// mounted nodes within this item, it doesn't need to be detached given that it's parent is.
    ///
    /// Note: Every node that is mounted doesn't need to be directly unmounted. If a node is a
    /// child of another node, then unmounting the parent will result in the child being unmounted.
    fn detach(&mut self, top_level: bool);
}
