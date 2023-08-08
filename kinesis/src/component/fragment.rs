use web_sys::{Document, Node, Text};

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

/// Helper type for updatable text. The closure will be called with the current context, and must
/// return text to be placed within a text node to be rendered.
pub type GetTextFn<Ctx> = Box<dyn Fn(&Ctx) -> String>;

pub struct Updatable<Ctx> {
    get_text: GetTextFn<Ctx>,
    location: Location,
}
impl<Ctx> Updatable<Ctx> {
    pub fn new<F>(location: Location, get_text: F) -> Self
    where
        F: 'static + Fn(&Ctx) -> String,
    {
        Self {
            get_text: Box::new(get_text) as GetTextFn<Ctx>,
            location,
        }
    }
}

pub struct Fragment<Ctx> {
    document: Document,

    mounted: bool,

    pieces: Vec<(Piece, Node)>,
    updatables: Vec<(Updatable<Ctx>, Text)>,

    dependencies: HashMapList<usize, usize>,
}

impl<Ctx> Fragment<Ctx> {
    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),
            mounted: false,
            pieces: Vec::new(),

            updatables: Vec::new(),
            dependencies: HashMapList::new(),
        }
    }

    pub fn with_piece(mut self, piece: Piece) -> Self {
        // Create the accompanying node
        let node = self.make_node(&piece);

        self.pieces.push((piece, node));

        self
    }

    /// Resolve a node or reference to a node with the node cache.
    fn as_node<'a>(&'a self, node_or_reference: &'a NodeOrReference) -> Option<&Node> {
        match node_or_reference {
            NodeOrReference::Node(node) => Some(node),
            NodeOrReference::Reference(id) => self.pieces.get(*id).map(|(_, node)| node),
            _ => None,
        }
    }

    /// Mount the current fragment to the target.
    pub fn mount(&mut self, context: &Ctx, target: &Node, anchor: Option<&Node>) {
        // Prevent double mounting
        if self.mounted {
            return;
        }

        // Mount static pieces, filtering out nodes that don't have a created node
        for (piece, node) in &self.pieces {
            // Mount the node
            self.mount_node(&piece.location, node, target, anchor);
        }

        // Mount updatable pieces
        for (updatable, node) in &self.updatables {
            let text_content = updatable.get_text.as_ref()(context);
            node.set_data(&text_content);

            self.mount_node(&updatable.location, node, target, anchor);
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
            .map(|(_, node)| node);

        // Remove each of them
        for node in top_level {
            node.parent_node()
                .expect("node to have parent before removal")
                .remove_child(node)
                .expect("to remove node from parent");
        }

        self.mounted = false;
    }

    pub fn with_updatable(mut self, dependencies: &[usize], updatable: Updatable<Ctx>) -> Self {
        // Create the node
        let node = self.document.create_text_node("");

        // Determine the updatable's ID
        let updatable_id = self.updatables.len();

        // Insert into the updatables collection
        self.updatables.push((updatable, node));

        for dependency in dependencies {
            self.dependencies.insert(*dependency, updatable_id);
        }

        self
    }

    /// Update relevant parts of fragment in response to the state changing
    pub fn update(&self, changed: &[usize], context: &Ctx) {
        for updatable_id in changed
            .iter()
            .flat_map(|dependency| self.dependencies.get(dependency))
            .flatten()
        {
            let (updatable, node) = self
                .updatables
                .get(*updatable_id)
                .expect("valid updatable for given ID");

            let text_content = updatable.get_text.as_ref()(context);
            node.set_data(&text_content);
        }
    }

    fn mount_node(&self, location: &Location, node: &Node, target: &Node, anchor: Option<&Node>) {
        match location {
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
