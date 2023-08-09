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

pub type GetItemsFn<Ctx> = Box<dyn Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>>;
pub struct Each<Ctx> {
    location: Location,
    get_items: GetItemsFn<Ctx>,
}

#[derive(Clone, Copy)]
enum DependencyType {
    Updatable(usize),
    Conditional(usize),
}

pub struct FragmentBuilder<Ctx> {
    pieces: Vec<Piece>,
    updatables: Vec<(Vec<usize>, Location, GetTextFn<Ctx>)>,
    conditionals: Vec<(
        Vec<usize>,
        Location,
        FragmentBuilder<Ctx>,
        CheckConditionFn<Ctx>,
    )>,
    each_blocks: Vec<(Vec<usize>, Location, GetItemsFn<Ctx>)>,
}
impl<Ctx> FragmentBuilder<Ctx> {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            updatables: Vec::new(),
            conditionals: Vec::new(),
            each_blocks: Vec::new(),
        }
    }

    pub fn with_piece(mut self, piece: Piece) -> Self {
        self.pieces.push(piece);
        self
    }

    pub fn with_updatable<F>(
        mut self,
        dependencies: &[usize],
        location: Location,
        get_text: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> String,
    {
        self.updatables.push((
            dependencies.to_vec(),
            location,
            Box::new(get_text) as GetTextFn<Ctx>,
        ));
        self
    }

    pub fn with_conditional<F>(
        mut self,
        dependencies: &[usize],
        location: Location,
        fragment: FragmentBuilder<Ctx>,
        check_condition: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> bool,
    {
        self.conditionals.push((
            dependencies.to_vec(),
            location,
            fragment,
            Box::new(check_condition) as CheckConditionFn<Ctx>,
        ));
        self
    }

    pub fn with_each<F>(mut self, dependencies: &[usize], location: Location, get_items: F) -> Self
    where
        F: 'static + Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>,
    {
        self.each_blocks.push((
            dependencies.to_vec(),
            location,
            Box::new(get_items) as GetItemsFn<Ctx>,
        ));
        self
    }

    pub fn build(self, document: &Document) -> Fragment<Ctx> {
        let mut fragment = Fragment::new(document);

        for piece in self.pieces {
            fragment.with_piece(piece);
        }

        for (dependencies, location, get_text) in self.updatables {
            fragment.with_updatable(&dependencies, location, get_text);
        }

        for (dependencies, location, fragment_builder, check_condition) in self.conditionals {
            fragment.with_conditional(
                &dependencies,
                location,
                fragment_builder.build(document),
                check_condition,
            );
        }

        fragment
    }
}

pub struct Fragment<Ctx> {
    document: Document,

    mounted: bool,

    pieces: Vec<(Piece, Node)>,
    updatables: Vec<(Updatable<Ctx>, Text)>,
    conditionals: Vec<(Conditional<Ctx>, Node)>,
    each_blocks: Vec<Each<Ctx>>,

    dependencies: HashMapList<usize, DependencyType>,
}

impl<Ctx> Fragment<Ctx> {
    pub fn build() -> FragmentBuilder<Ctx> {
        FragmentBuilder::new()
    }

    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),
            mounted: false,

            pieces: Vec::new(),
            updatables: Vec::new(),
            conditionals: Vec::new(),
            each_blocks: Vec::new(),

            dependencies: HashMapList::new(),
        }
    }

    /// Inserts a [Piece] into the fragment, which can include text or an element.
    pub fn with_piece(&mut self, piece: Piece) {
        // Create the accompanying node
        let node = piece.create_node(&self.document);

        self.pieces.push((piece, node));
    }

    /// Inserts an [Updatable] piece of text into the fragment. The dependencies for the text
    /// should be specified, so that when the dependencies change the text can be updated in the
    /// DOM.
    pub fn with_updatable(
        &mut self,
        dependencies: &[usize],
        location: Location,
        get_text: GetTextFn<Ctx>,
    ) {
        // Build the updatable
        let updatable = Updatable { get_text, location };

        // Create the node
        let node = self.document.create_text_node("");

        // Determine the updatable's ID
        let updatable_id = self.updatables.len();

        // Insert into the updatables collection
        self.updatables.push((updatable, node));

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Updatable(updatable_id));
    }

    pub fn with_conditional(
        &mut self,
        dependencies: &[usize],
        location: Location,
        fragment: Fragment<Ctx>,
        check_condition: CheckConditionFn<Ctx>,
    ) {
        let conditional = Conditional {
            check_condition,
            fragment,
            location,
        };

        // Create the anchor node
        let anchor = self.anchor();

        // Determine the conditional's ID
        let conditional_id = self.conditionals.len();

        // Insert into the conditionals collection
        self.conditionals.push((conditional, anchor));

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Conditional(conditional_id));
    }

    pub fn with_each(
        &mut self,
        dependencies: &[usize],
        location: Location,
        get_items: GetItemsFn<Ctx>,
    ) {
        // Create an anchor for the each block
        let anchor = self.anchor();

        // Determine the ID of the each block
        let each_block_id = self.each_blocks.len();

        // Insert the each block into the collection
        self.each_blocks.push(Each {
            location,
            get_items,
        });

        // Register dependencies for the each block
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

    /// Create an anchor node. This is just a text node with no content.
    fn anchor(&self) -> Node {
        self.document.create_text_node("").into()
    }
}
