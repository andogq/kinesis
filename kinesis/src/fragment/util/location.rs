use web_sys::Node as WsNode;

/// Expresses a location relative to a [`web_sys::Node`] in the DOM. Primarily used for directing a
/// [`super::super::Dynamic`] when mounting it to the DOM.
#[derive(Clone)]
pub struct Location {
    /// The parent to mount the [`web_sys::Node`] within.
    parent: WsNode,

    /// An optional anchor to use when mounting the [`web_sys::Node`]. If provided, the node will
    /// be inserted before the anchor. If not provided, then the node will be appended to the
    /// parent.
    anchor: Option<WsNode>,
}

impl Location {
    /// Create a location from an anchor. Will attempt to retrieve the parent of the anchor, which
    /// will fail if the anchor is not currently mounted.
    pub fn anchor<N>(anchor: &N) -> Self
    where
        N: AsRef<WsNode>,
    {
        let anchor = anchor.as_ref();

        Self {
            parent: anchor.parent_node().expect("anchor to have parent"),
            anchor: Some(anchor.clone()),
        }
    }

    /// Create a location from a parent, without an anchor.
    pub fn parent<N>(parent: &N) -> Self
    where
        N: AsRef<WsNode>,
    {
        Self {
            parent: parent.as_ref().clone(),
            anchor: None,
        }
    }

    /// Create a location with both an anchor and a parent.
    pub fn anchored_parent<P, A>(parent: P, anchor: Option<A>) -> Self
    where
        P: AsRef<WsNode>,
        A: AsRef<WsNode>,
    {
        Self {
            parent: parent.as_ref().clone(),
            anchor: anchor.as_ref().map(|anchor| anchor.as_ref().clone()),
        }
    }

    /// Use the location to mount the provided [`web_sys::Node`]. Assumes that the parent is
    /// mounted.
    pub fn mount<N>(&self, node: &N)
    where
        N: AsRef<WsNode>,
    {
        self.parent
            .insert_before(node.as_ref(), self.anchor.as_ref())
            .expect("node mounted into parent");
    }
}
