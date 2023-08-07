use std::rc::Rc;

use web_sys::{Document, Element, Node, Text};

use crate::util::HashMapList;

pub enum Kind {
    Text(String),
    Element(ElementKind),
}

#[derive(Clone, Copy)]
pub enum ElementKind {
    P,
    Div,
}
impl From<&ElementKind> for &str {
    fn from(element_type: &ElementKind) -> Self {
        use ElementKind::*;

        match element_type {
            P => "p",
            Div => "div",
        }
    }
}
impl From<ElementKind> for &str {
    fn from(element_type: ElementKind) -> Self {
        (&element_type).into()
    }
}

/// Allows a [Node] or reference to a cached node to be used interchangeably.
#[derive(Clone)]
pub enum NodeOrReference {
    /// A node in the DOM.
    Node(Node),

    /// A reference to a cached node.
    Reference(usize),
}
impl NodeOrReference {}

pub enum Location {
    /// Insert the element to the target before the specified anchor. Shorthand for
    /// [Location::Insert] with the render target as the parent.
    Target,

    /// Append the element to the parent. Corresponds to `appendChild` method.
    Append(NodeOrReference),

    /// Insert the child to the parent before the specified anchor. Corresponds to `insertBefore`
    /// method.
    Insert {
        parent: NodeOrReference,
        anchor: NodeOrReference,
    },
}
impl Location {
    pub fn append(parent: NodeOrReference) -> Self {
        Self::Append(parent)
    }

    pub fn insert(parent: NodeOrReference, anchor: NodeOrReference) -> Self {
        Self::Insert { parent, anchor }
    }
}

pub struct Piece {
    kind: Kind,
    location: Location,
}
impl Piece {
    pub fn new(kind: Kind, location: Location) -> Self {
        Self { kind, location }
    }
}

/// Helper type for an updatable [Piece]. The closure will be called with the current context, and
/// must return a piece to be rendered.
pub type Updatable<Ctx> = dyn Fn(&Ctx) -> Piece;

pub struct Fragment<C, Ctx> {
    /// A closure, which must return a reference to the context.
    get_context: C,

    document: Document,

    mounted: bool,

    pieces: Vec<(Piece, Option<Node>)>,

    updates: HashMapList<usize, Rc<Updatable<Ctx>>>,
}

impl<'ctx, C, Ctx> Fragment<C, Ctx>
where
    // `'ctx` lifetime defined as the lifetime of the `Ctx` type
    Ctx: 'ctx,
    // Get context closure must return a reference to the closure
    C: Fn() -> &'ctx Ctx,
{
    pub fn new(document: &Document, get_context: C) -> Self {
        Self {
            get_context,

            document: document.clone(),
            mounted: false,
            pieces: Vec::new(),

            updates: HashMapList::new(),
        }
    }

    pub fn with_piece(mut self, piece: Piece) -> Self {
        // Create the accompanying node
        let node = self.make_node(&piece);

        self.pieces.push((piece, Some(node)));

        self
    }

    /// Resolve a node or reference to a node with the node cache.
    fn as_node<'a>(&'a self, node_or_reference: &'a NodeOrReference) -> Option<&Node> {
        match node_or_reference {
            NodeOrReference::Node(node) => Some(node),
            NodeOrReference::Reference(id) => {
                self.pieces.get(*id).and_then(|(_, node)| node.as_ref())
            }
            _ => None,
        }
    }

    /// Mount the current fragment to the target.
    pub fn mount(&mut self, target: &Node, anchor: Option<&Node>) {
        let context = (self.get_context)();

        // Prevent double mounting
        if self.mounted {
            return;
        }

        // Mount static pieces
        for (piece, node) in self
            .pieces
            .iter()
            .filter_map(|(piece, node)| node.as_ref().map(|node| (piece, node)))
        {
            // Mount the node
            self.mount_piece(piece, node, target, anchor);
        }

        // Mount updatable pieces
        for (_, updatable) in &self.updates {
            let piece = updatable(context);
            let node = self.make_node(&piece);

            self.mount_piece(&piece, &node, target, anchor);
        }

        self.mounted = true;
    }

    /// Detach the fragment from the DOM. Only 'top level' pieces need to be unmounted from the
    /// DOM, as they will take any nested pieces with them.
    pub fn detach(&mut self) {
        if !self.mounted {
            // Prevent double detaching
            return;
        }

        // Find top level nodes
        let top_level = self
            .pieces
            .iter()
            .filter(|(piece, _)| matches!(piece.location, Location::Target))
            .filter_map(|(_, node)| node.as_ref());

        // Remove each of them
        for node in top_level {
            node.parent_node()
                .expect("node to have parent before removal")
                .remove_child(node)
                .expect("to remove node from parent");
        }

        self.mounted = false;
    }

    pub fn with_updatable<F>(mut self, dependencies: &[usize], create: F) -> Self
    where
        F: Fn(&Ctx) -> Piece + 'static,
    {
        let create = Rc::new(create) as Rc<dyn Fn(&Ctx) -> Piece>;
        for dependency in dependencies {
            self.updates.insert(*dependency, Rc::clone(&create));
        }

        self
    }

    /// Update relevant parts of fragment in response to the state changing
    pub fn update(&self, state: &C) {
        todo!()
    }

    fn mount_piece(&self, piece: &Piece, node: &Node, target: &Node, anchor: Option<&Node>) {
        match &piece.location {
            Location::Target => {
                target
                    .insert_before(node, anchor)
                    .expect("to append child to target");
            }
            Location::Append(parent) => {
                self.as_node(parent)
                    .expect("parent to be an existing node")
                    .append_child(node)
                    .expect("to append child to parent");
            }
            Location::Insert { parent, anchor } => {
                self.as_node(parent)
                    .expect("parent to be an existing node")
                    .insert_before(
                        node,
                        Some(self.as_node(anchor).expect("anchor to be an existing node")),
                    )
                    .expect("to insert child before anchor");
            }
        }
    }

    fn make_node(&self, piece: &Piece) -> Node {
        match &piece.kind {
            Kind::Element(element_kind) => self
                .document
                .create_element(element_kind.into())
                .expect("to create a new element")
                .into(),
            Kind::Text(text_content) => self.document.create_text_node(text_content).into(),
        }
    }
}
