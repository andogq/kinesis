use web_sys::{Document, Element, Node, Text};

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

pub struct Fragment {
    document: Document,

    mounted: bool,

    pieces: Vec<(Piece, Option<Node>)>,
}

impl Fragment {
    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),
            mounted: false,
            pieces: Vec::new(),
        }
    }

    pub fn with_piece(mut self, piece: Piece) -> Self {
        // Create the accompanying node
        let node = match &piece.kind {
            Kind::Element(element_kind) => self
                .document
                .create_element(element_kind.into())
                .expect("to create a new element")
                .into(),
            Kind::Text(text_content) => self.document.create_text_node(text_content).into(),
        };

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
        // Prevent double mounting
        if self.mounted {
            return;
        }

        for (piece, node) in self
            .pieces
            .iter()
            .filter_map(|(piece, node)| node.as_ref().map(|node| (piece, node)))
        {
            // Mount the node
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
}

fn render(document: &Document, target: &Node) {
    let mut fragment = Fragment::new(document)
        .with_piece(Piece::new(Kind::Element(ElementKind::P), Location::Target))
        .with_piece(Piece::new(
            Kind::Text("some content".into()),
            Location::Append(NodeOrReference::Reference(0)),
        ));

    fragment.mount(target, None);
}
