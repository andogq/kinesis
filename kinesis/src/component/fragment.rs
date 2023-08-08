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

pub struct Piece {
    kind: Kind,
    location: Location,
}
impl Piece {
    pub fn new(kind: Kind, location: Location) -> Self {
        Self { kind, location }
    }

    pub fn create_node(&self, document: &Document) -> Node {
        match &self.kind {
            Kind::Element(element_kind) => document
                .create_element(element_kind.into())
                .expect("to create a new element")
                .into(),
            Kind::Text(text_content) => document.create_text_node(text_content).into(),
        }
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

pub type CheckConditionFn<Ctx> = Box<dyn Fn(&Ctx) -> bool>;
pub struct Conditional<Ctx> {
    check_condition: CheckConditionFn<Ctx>,
    fragment: Fragment<Ctx>,
    location: Location,
}
impl<Ctx> Conditional<Ctx> {
    pub fn new<F>(location: Location, fragment: Fragment<Ctx>, check_condition: F) -> Self
    where
        F: 'static + Fn(&Ctx) -> bool,
    {
        Self {
            check_condition: Box::new(check_condition) as CheckConditionFn<Ctx>,
            fragment,
            location,
        }
    }
}

#[derive(Clone, Copy)]
enum DependencyType {
    Updatable(usize),
    Conditional(usize),
}

pub struct Fragment<Ctx> {
    document: Document,

    mounted: bool,

    pieces: Vec<(Piece, Node)>,
    updatables: Vec<(Updatable<Ctx>, Text)>,
    conditionals: Vec<(Conditional<Ctx>, Node)>,

    dependencies: HashMapList<usize, DependencyType>,
}

impl<Ctx> Fragment<Ctx> {
    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),
            mounted: false,

            pieces: Vec::new(),
            updatables: Vec::new(),
            conditionals: Vec::new(),

            dependencies: HashMapList::new(),
        }
    }

    /// Inserts a [Piece] into the fragment, which can include text or an element.
    pub fn with_piece(mut self, piece: Piece) -> Self {
        // Create the accompanying node
        let node = piece.create_node(&self.document);

        self.pieces.push((piece, node));

        self
    }

    /// Inserts an [Updatable] piece of text into the fragment. The dependencies for the text
    /// should be specified, so that when the dependencies change the text can be updated in the
    /// DOM.
    pub fn with_updatable_text(
        mut self,
        dependencies: &[usize],
        updatable: Updatable<Ctx>,
    ) -> Self {
        // Create the node
        let node = self.document.create_text_node("");

        // Determine the updatable's ID
        let updatable_id = self.updatables.len();

        // Insert into the updatables collection
        self.updatables.push((updatable, node));

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Updatable(updatable_id));

        self
    }

    pub fn with_conditional_fragment(
        mut self,
        dependencies: &[usize],
        conditional: Conditional<Ctx>,
    ) -> Self {
        // Create the anchor node
        let anchor = self.document.create_text_node("");

        // Determine the conditional's ID
        let conditional_id = self.conditionals.len();

        // Insert into the conditionals collection
        self.conditionals.push((conditional, anchor.into()));

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Conditional(conditional_id));

        self
    }

    /// Mount the current fragment to the specified target target.
    pub fn mount(&mut self, context: &Ctx, target: &Node, anchor: Option<&Node>) {
        // Prevent double mounting
        if self.mounted {
            return;
        }

        // Mount all of the parts of the fragment
        for (node, location) in self
            .pieces
            .iter()
            .map(|(piece, node)| (node, &piece.location))
            .chain(self.updatables.iter().map(|(updatable, text_node)| {
                // Update text content for updatable content
                let text_content = updatable.get_text.as_ref()(context);
                text_node.set_data(&text_content);

                (text_node.as_ref(), &updatable.location)
            }))
            .chain(self.conditionals.iter().map(|(conditional, anchor)| {
                // Pass through the anchor nodes for the conditional fragments
                (anchor, &conditional.location)
            }))
        {
            self.mount_node(node, location, target, anchor);
        }

        // Now that nodes have been mounted, attempt to mount conditionals
        for (conditional, anchor) in self.conditionals.iter_mut() {
            if (conditional.check_condition)(context) {
                conditional.fragment.mount(
                    context,
                    &anchor.parent_node().expect("anchor to have parent"),
                    Some(anchor),
                );
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

        for (node, _) in self
            .pieces
            .iter()
            .map(|(piece, node)| (node, &piece.location))
            .chain(
                self.updatables
                    .iter()
                    .map(|(updatable, node)| (node.as_ref(), &updatable.location)),
            )
            .chain(
                self.conditionals
                    .iter()
                    .map(|(conditional, node)| (node, &conditional.location)),
            )
            // Select only top level nodes
            .filter(|(_, location)| matches!(location, Location::Target))
        {
            node.parent_node()
                .expect("node to have parent before removal")
                .remove_child(node)
                .expect("to remove node from parent");
        }

        // Trigger unmount for conditional fragments
        for (conditional, _) in &mut self.conditionals {
            conditional.fragment.detach();
        }

        self.mounted = false;
    }

    /// Update relevant parts of fragment in response to the state changing
    pub fn update(&mut self, changed: &[usize], context: &Ctx) {
        for dependency in changed
            .iter()
            .flat_map(|dependency| self.dependencies.get(dependency))
            .flatten()
        {
            match dependency {
                DependencyType::Updatable(updatable_id) => {
                    let (updatable, node) = self
                        .updatables
                        .get(*updatable_id)
                        .expect("valid updatable for given ID");

                    let text_content = updatable.get_text.as_ref()(context);
                    node.set_data(&text_content);
                }
                DependencyType::Conditional(conditional_id) => {
                    if self.mounted {
                        let (conditional, anchor) = self
                            .conditionals
                            .get_mut(*conditional_id)
                            .expect("valid conditional for given ID");

                        // Acquire current states
                        let desired_state = (conditional.check_condition)(context);
                        let actual_state = conditional.fragment.mounted;

                        if desired_state != actual_state {
                            if desired_state {
                                conditional.fragment.mount(
                                    context,
                                    &anchor.parent_node().expect("anchor to have parent"),
                                    Some(anchor),
                                );
                            } else {
                                conditional.fragment.detach();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Register dependencies.
    fn register_dependencies(&mut self, dependencies: &[usize], dependency_type: DependencyType) {
        for dependency in dependencies {
            self.dependencies.insert(*dependency, dependency_type);
        }
    }

    /// Resolve a node or reference to a node associated with a [Piece].
    fn resolve_to_node<'a>(&'a self, node_or_reference: &'a NodeOrReference) -> Option<&Node> {
        match node_or_reference {
            NodeOrReference::Node(node) => Some(node),
            NodeOrReference::Reference(id) => self.pieces.get(*id).map(|(_, node)| node),
        }
    }

    /// Mounts a [Node] to the specified [Location]. The root target and anchor are included, for
    /// when the node's location is [Location::Target].
    fn mount_node(&self, node: &Node, location: &Location, target: &Node, anchor: Option<&Node>) {
        match location {
            Location::Target => {
                target
                    .insert_before(node, anchor)
                    .expect("to append child to target");
            }
            Location::Append(parent) => {
                self.resolve_to_node(parent)
                    .expect("parent to be an existing node")
                    .append_child(node)
                    .expect("to append child to parent");
            }
            Location::Insert { parent, anchor } => {
                self.resolve_to_node(parent)
                    .expect("parent to be an existing node")
                    .insert_before(
                        node,
                        Some(
                            self.resolve_to_node(anchor)
                                .expect("anchor to be an existing node"),
                        ),
                    )
                    .expect("to insert child before anchor");
            }
        }
    }
}
