use web_sys::{Document, Node, Text};

use crate::util::HashMapList;

pub enum Kind {
    Text(String),
    Element(ElementKind),
}
impl Kind {
    pub fn create_node(&self, document: &Document) -> Node {
        match &self {
            Kind::Element(element_kind) => document
                .create_element(element_kind.into())
                .expect("to create a new element")
                .into(),
            Kind::Text(text_content) => document.create_text_node(text_content).into(),
        }
    }
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
    /// Append the element to the parent. Corresponds to `appendChild` method.
    Append(NodeOrReference),

    /// Insert the child to the parent before the specified anchor. Corresponds to `insertBefore`
    /// method.
    Insert {
        parent: NodeOrReference,
        anchor: Option<NodeOrReference>,
    },
}

pub struct Piece {
    location: Option<Location>,
    node: Node,
}
impl Piece {}

/// Helper type for updatable text. The closure will be called with the current context, and must
/// return text to be placed within a text node to be rendered.
pub type GetTextFn<Ctx> = Box<dyn Fn(&Ctx) -> String>;

pub struct Updatable<Ctx> {
    get_text: GetTextFn<Ctx>,
    location: Option<Location>,

    /// Reference to the text node that will be inserted and updated in the DOM
    text_node: Text,
}
impl<Ctx> Updatable<Ctx> {
    pub fn mount(
        &mut self,
        document: &Document,
        context: &Ctx,
        target: &Node,
        anchor: Option<&Node>,
    ) {
        self.update(document, context, target, anchor);
    }

    pub fn update(
        &mut self,
        _document: &Document,
        context: &Ctx,
        _target: &Node,
        _anchor: Option<&Node>,
    ) {
        // Update text content for updatable content
        let text_content = (self.get_text)(context);
        self.text_node.set_data(&text_content);
    }

    pub fn detach(&mut self) {
        // No special detach behavior required
    }
}

pub type CheckConditionFn<Ctx> = Box<dyn Fn(&Ctx) -> bool>;
pub struct Conditional<Ctx> {
    check_condition: CheckConditionFn<Ctx>,
    fragment: Fragment<Ctx>,
    location: Option<Location>,

    /// Reference to the anchor for this fragment in the DOM
    anchor: Node,
}
impl<Ctx> Conditional<Ctx> {
    pub fn mount(
        &mut self,
        _document: &Document,
        context: &Ctx,
        target: &Node,
        anchor: Option<&Node>,
    ) {
        if (self.check_condition)(context) {
            self.fragment.mount(context, target, anchor);
        }
    }

    pub fn update(
        &mut self,
        _document: &Document,
        context: &Ctx,
        target: &Node,
        anchor: Option<&Node>,
    ) {
        // Acquire current states
        let desired_state = (self.check_condition)(context);
        let actual_state = self.fragment.mounted;

        if desired_state != actual_state {
            if desired_state {
                self.fragment.mount(context, target, anchor);
            } else {
                self.fragment.detach();
            }
        }
    }

    pub fn detach(&mut self) {
        self.fragment.detach();
    }
}

pub type GetItemsFn<Ctx> = Box<dyn Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>>;
pub struct Each<Ctx> {
    location: Option<Location>,
    get_items: GetItemsFn<Ctx>,
    mounted_fragments: Option<Vec<Fragment<Ctx>>>,

    anchor: Node,
}
impl<Ctx> Each<Ctx> {
    /// Mounts all of the items. Internally calls `update` since the logic is the same.
    pub fn mount(
        &mut self,
        document: &Document,
        context: &Ctx,
        target: &Node,
        anchor: Option<&Node>,
    ) {
        // Mounting logic is the same as update
        self.update(document, context, target, anchor);
    }

    /// Gets a list of fragments from `get_items`, and mounts them all.
    pub fn update(
        &mut self,
        document: &Document,
        context: &Ctx,
        target: &Node,
        anchor: Option<&Node>,
    ) {
        // TODO: Should ideally be something like:
        // For each child
        // Check if child exists
        // If it does, call update and pass context
        // If it doesn't, create and mount it

        // Detach any existing fragments.
        self.detach();

        // Create new fragments
        let fragments = (self.get_items)(context)
            .map(|fragment| {
                let mut fragment = fragment.build(document);

                // Mount each of them
                fragment.mount(context, target, anchor);

                fragment
            })
            .collect();

        // Save fragments
        self.mounted_fragments = Some(fragments);
    }

    /// Detaches all mounted fragments
    pub fn detach(&mut self) {
        self.mounted_fragments
            .take()
            .into_iter()
            .flatten()
            .for_each(|mut fragment| fragment.detach());
    }
}

#[derive(Clone, Copy)]
enum DependencyType {
    Updatable(usize),
    Conditional(usize),
    EachBlock(usize),
}

/// Used to build and represent a fragment that does not yet have access to the [Document].
pub struct FragmentBuilder<Ctx> {
    pieces: Vec<(Kind, Option<Location>)>,
    updatables: Vec<(Vec<usize>, Option<Location>, GetTextFn<Ctx>)>,
    conditionals: Vec<(
        Vec<usize>,
        Option<Location>,
        FragmentBuilder<Ctx>,
        CheckConditionFn<Ctx>,
    )>,
    each_blocks: Vec<(Vec<usize>, Option<Location>, GetItemsFn<Ctx>)>,
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

    pub fn with_piece(mut self, kind: Kind, location: Option<Location>) -> Self {
        self.pieces.push((kind, location));
        self
    }

    pub fn with_updatable<F>(
        mut self,
        dependencies: &[usize],
        location: Option<Location>,
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
        location: Option<Location>,
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

    pub fn with_each<F>(
        mut self,
        dependencies: &[usize],
        location: Option<Location>,
        get_items: F,
    ) -> Self
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

        for (kind, location) in self.pieces {
            fragment.with_piece(kind, location);
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

        for (dependencies, location, get_items) in self.each_blocks {
            fragment.with_each(&dependencies, location, get_items);
        }

        fragment
    }
}

pub struct Fragment<Ctx> {
    document: Document,

    mounted: bool,

    pieces: Vec<Piece>,
    updatables: Vec<Updatable<Ctx>>,
    conditionals: Vec<Conditional<Ctx>>,
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
    pub fn with_piece(&mut self, kind: Kind, location: Option<Location>) {
        let piece = Piece {
            location,
            node: kind.create_node(&self.document),
        };

        self.pieces.push(piece);
    }

    /// Inserts an [Updatable] piece of text into the fragment. The dependencies for the text
    /// should be specified, so that when the dependencies change the text can be updated in the
    /// DOM.
    pub fn with_updatable(
        &mut self,
        dependencies: &[usize],
        location: Option<Location>,
        get_text: GetTextFn<Ctx>,
    ) {
        // Build the updatable
        let updatable = Updatable {
            get_text,
            location,
            text_node: self.document.create_text_node(""),
        };

        // Determine the updatable's ID
        let updatable_id = self.updatables.len();

        // Insert into the updatables collection
        self.updatables.push(updatable);

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Updatable(updatable_id));
    }

    pub fn with_conditional(
        &mut self,
        dependencies: &[usize],
        location: Option<Location>,
        fragment: Fragment<Ctx>,
        check_condition: CheckConditionFn<Ctx>,
    ) {
        let conditional = Conditional {
            check_condition,
            fragment,
            location,
            anchor: self.anchor(),
        };

        // Determine the conditional's ID
        let conditional_id = self.conditionals.len();

        // Insert into the conditionals collection
        self.conditionals.push(conditional);

        // Register the dependencies
        self.register_dependencies(dependencies, DependencyType::Conditional(conditional_id));
    }

    pub fn with_each(
        &mut self,
        dependencies: &[usize],
        location: Option<Location>,
        get_items: GetItemsFn<Ctx>,
    ) {
        let each = Each {
            location,
            get_items,
            mounted_fragments: None,
            anchor: self.anchor(),
        };

        // Determine the ID of the each block
        let each_block_id = self.each_blocks.len();

        // Insert the each block into the collection
        self.each_blocks.push(each);

        // Register dependencies for the each block
        self.register_dependencies(dependencies, DependencyType::EachBlock(each_block_id))
    }

    /// Mount the current fragment to the specified target target.
    pub fn mount(&mut self, context: &Ctx, target: &Node, target_anchor: Option<&Node>) {
        // Prevent double mounting
        if self.mounted {
            return;
        }

        let root_location = Location::Insert {
            parent: NodeOrReference::Node(target.clone()),
            anchor: target_anchor.cloned().map(NodeOrReference::Node),
        };

        // Mount all of the parts of the fragment
        for (node, location) in self
            .pieces
            .iter()
            .map(|piece| (&piece.node, &piece.location))
            .chain(
                self.updatables
                    .iter()
                    .map(|updatable| (updatable.text_node.as_ref(), &updatable.location)),
            )
            .chain(self.conditionals.iter().map(|conditional| {
                // Pass through the anchor nodes for the conditional fragments
                (&conditional.anchor, &conditional.location)
            }))
            .chain(
                self.each_blocks
                    .iter()
                    .map(|each_block| (&each_block.anchor, &each_block.location)),
            )
        {
            self.mount_node(node, location.as_ref().unwrap_or(&root_location));
        }

        for updatable in &mut self.updatables {
            // TODO: Don't really need this node as ref???
            updatable.mount(
                &self.document,
                context,
                updatable.text_node.clone().as_ref(),
                None,
            );
        }

        // Now that nodes have been mounted, attempt to mount conditionals
        for conditional in self.conditionals.iter_mut() {
            let anchor = conditional.anchor.clone();
            conditional.mount(
                &self.document,
                context,
                &anchor.parent_node().expect("anchor to have parent"),
                Some(&anchor),
            );
        }

        for each_block in &mut self.each_blocks {
            let anchor = each_block.anchor.clone();
            each_block.mount(
                &self.document,
                context,
                &anchor.parent_node().expect("anchor to have parent"),
                Some(&anchor),
            );
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
            .map(|piece| (&piece.node, &piece.location))
            .chain(
                self.updatables
                    .iter()
                    .map(|updatable| (updatable.text_node.as_ref(), &updatable.location)),
            )
            .chain(
                self.conditionals
                    .iter()
                    .map(|conditional| (&conditional.anchor, &conditional.location)),
            )
            .chain(
                self.each_blocks
                    .iter()
                    .map(|each_block| (&each_block.anchor, &each_block.location)),
            )
            // Select only top level nodes
            .filter(|(_, location)| location.is_none())
        {
            node.parent_node()
                .expect("node to have parent before removal")
                .remove_child(node)
                .expect("to remove node from parent");
        }

        self.updatables
            .iter_mut()
            .for_each(|updatable| updatable.detach());

        // Trigger unmount for conditional fragments
        self.conditionals
            .iter_mut()
            .for_each(|conditional| conditional.detach());

        // Trigger unmount for each blocks
        self.each_blocks
            .iter_mut()
            .for_each(|each_block| each_block.detach());

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
                    let updatable = self
                        .updatables
                        .get_mut(*updatable_id)
                        .expect("valid updatable for given ID");

                    // TODO: Bad
                    updatable.update(
                        &self.document,
                        context,
                        updatable.text_node.clone().as_ref(),
                        None,
                    );
                }
                DependencyType::Conditional(conditional_id) => {
                    if self.mounted {
                        let conditional = self
                            .conditionals
                            .get_mut(*conditional_id)
                            .expect("valid conditional for given ID");

                        let anchor = conditional.anchor.clone();
                        conditional.update(
                            &self.document,
                            context,
                            &anchor.parent_node().expect("anchor to have parent"),
                            Some(&anchor),
                        );
                    }
                }
                DependencyType::EachBlock(each_block_id) => {
                    if self.mounted {
                        let each_block = self
                            .each_blocks
                            .get_mut(*each_block_id)
                            .expect("valid each block for given ID");

                        let anchor = each_block.anchor.clone();
                        each_block.update(
                            &self.document,
                            context,
                            &anchor.parent_node().expect("anchor to have parent"),
                            Some(&anchor),
                        );
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
            NodeOrReference::Reference(id) => self.pieces.get(*id).map(|piece| &piece.node),
        }
    }

    /// Mounts a [Node] to the specified [Location]. The root target and anchor are included, for
    /// when the node's location is [Location::Target].
    fn mount_node(&self, node: &Node, location: &Location) {
        match location {
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
                        anchor.as_ref().map(|anchor| {
                            self.resolve_to_node(anchor)
                                .expect("anchor to be an existing node")
                        }),
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
