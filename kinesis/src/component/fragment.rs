use web_sys::{console, Document, Node, Text};

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
impl NodeOrReference {
    pub fn resolve(self, node_collection: &[Node]) -> Result<Node, ()> {
        use NodeOrReference::*;

        match self {
            Node(node) => Ok(node),
            Reference(id) => node_collection.get(id).cloned().ok_or(()),
        }
    }
}

#[derive(Clone)]
pub enum ResolvedLocation {
    Anchor(Node),
    Parent(Node),
    AnchoredParent { parent: Node, anchor: Option<Node> },
}
impl ResolvedLocation {
    pub fn anchor<N>(anchor: N) -> Self
    where
        N: AsRef<Node>,
    {
        Self::Anchor(anchor.as_ref().clone())
    }
    pub fn parent<N>(parent: N) -> Self
    where
        N: AsRef<Node>,
    {
        Self::Parent(parent.as_ref().clone())
    }
    pub fn anchored_parent<P, A>(parent: P, anchor: Option<A>) -> Self
    where
        P: AsRef<Node>,
        A: AsRef<Node>,
    {
        Self::AnchoredParent {
            parent: parent.as_ref().clone(),
            anchor: anchor.as_ref().map(|anchor| anchor.as_ref().clone()),
        }
    }

    pub fn mount<N>(&self, node: &N)
    where
        N: AsRef<Node>,
    {
        use ResolvedLocation::*;

        let (parent, anchor) = match self {
            Anchor(anchor) => (
                anchor.parent_node().expect("anchor to have parent"),
                Some(anchor),
            ),
            Parent(parent) => (parent.clone(), None),
            AnchoredParent { parent, anchor } => (parent.clone(), anchor.as_ref()),
        };

        parent
            .insert_before(node.as_ref(), anchor)
            .expect("node mounted into parent");
    }
}

/// Helper type for updatable text. The closure will be called with the current context, and must
/// return text to be placed within a text node to be rendered.
pub type GetTextFn<Ctx> = Box<dyn Fn(&Ctx) -> String>;

pub struct Updatable<Ctx> {
    get_text: GetTextFn<Ctx>,

    /// Reference to the text node that will be inserted and updated in the DOM
    text_node: Text,
}
impl<Ctx> Updatable<Ctx> {
    pub fn new(document: &Document, get_text: GetTextFn<Ctx>) -> Self {
        Self {
            get_text,
            text_node: document.create_text_node(""),
        }
    }
}

impl<Ctx> Part<Ctx> for Updatable<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &ResolvedLocation) {
        location.mount(&self.text_node);
    }

    fn update(&mut self, context: &Ctx, _changed: &[usize]) {
        self.text_node.set_data(&(self.get_text)(context));
    }

    fn detach(&mut self, top_level: bool) {
        if top_level {
            self.text_node
                .parent_node()
                .expect("node to have parent")
                .remove_child(&self.text_node);
        }
    }
}

pub type CheckConditionFn<Ctx> = Box<dyn Fn(&Ctx) -> bool>;
pub struct Conditional<Ctx> {
    check_condition: CheckConditionFn<Ctx>,
    fragment: Fragment<Ctx>,

    /// Reference to the anchor for this fragment in the DOM
    anchor: Node,

    fragment_mounted: bool,
}
impl<Ctx> Conditional<Ctx> {
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

impl<Ctx> Part<Ctx> for Conditional<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &ResolvedLocation) {
        location.mount(&self.anchor);
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        let should_mount = (self.check_condition)(context);
        if !self.fragment_mounted && should_mount {
            self.fragment.mount(&ResolvedLocation::AnchoredParent {
                parent: self.anchor.parent_node().expect("anchor to have parent"),
                anchor: Some(self.anchor.clone()),
            });

            self.fragment_mounted = true;
        } else if self.fragment_mounted && !should_mount {
            // Top level because parent won't be removed
            self.fragment.detach(true);

            self.fragment_mounted = false;
        }

        if self.fragment_mounted {
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

pub type GetItemsFn<Ctx> = Box<dyn Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>>;
pub struct Each<Ctx> {
    document: Document,
    get_items: GetItemsFn<Ctx>,
    mounted_fragments: Option<Vec<Fragment<Ctx>>>,

    anchor: Node,
}
impl<Ctx> Each<Ctx>
where
    Ctx: 'static,
{
    pub fn new(document: &Document, get_items: GetItemsFn<Ctx>) -> Self {
        Self {
            document: document.clone(),
            get_items,
            mounted_fragments: None,
            anchor: document.create_text_node("").into(),
        }
    }

    fn detach_fragments(&mut self, top_level: bool) {
        self.mounted_fragments
            .take()
            .into_iter()
            .flatten()
            .for_each(|mut fragment| fragment.detach(top_level));
    }
}

impl<Ctx> Part<Ctx> for Each<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &ResolvedLocation) {
        location.mount(&self.anchor);
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        // Detach all current mounted fragments (top level as their parent won't be removed)
        self.detach_fragments(true);

        // Create new fragments
        self.mounted_fragments = Some(
            (self.get_items)(context)
                .map(|builder| {
                    let mut fragment = builder.build(&self.document);

                    fragment.mount(&ResolvedLocation::AnchoredParent {
                        parent: self.anchor.parent_node().expect("anchor to have parent"),
                        anchor: Some(self.anchor.clone()),
                    });
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
            .remove_child(&self.anchor);
    }
}

trait Part<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &ResolvedLocation);
    fn update(&mut self, context: &Ctx, changed: &[usize]);
    fn detach(&mut self, top_level: bool);
}

/// Used to build and represent a fragment that does not yet have access to the [Document].
pub struct FragmentBuilder<Ctx> {
    pieces: Vec<(Kind, Option<usize>)>,
    updatables: Vec<(Vec<usize>, Option<usize>, GetTextFn<Ctx>)>,
    conditionals: Vec<(
        Vec<usize>,
        Option<usize>,
        FragmentBuilder<Ctx>,
        CheckConditionFn<Ctx>,
    )>,
    each_blocks: Vec<(Vec<usize>, Option<usize>, GetItemsFn<Ctx>)>,
}
impl<Ctx> FragmentBuilder<Ctx>
where
    Ctx: 'static,
{
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            updatables: Vec::new(),
            conditionals: Vec::new(),
            each_blocks: Vec::new(),
        }
    }

    pub fn with_piece(mut self, kind: Kind, location: Option<usize>) -> Self {
        self.pieces.push((kind, location));
        self
    }

    pub fn with_updatable<F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
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
        location: Option<usize>,
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
        location: Option<usize>,
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

        self.pieces
            .into_iter()
            .for_each(|(kind, location)| fragment.with_piece(kind, location));

        self.updatables
            .into_iter()
            .for_each(|(dependencies, location, get_text)| {
                fragment.with_part(Updatable::new(document, get_text), &dependencies, location);
            });

        self.conditionals.into_iter().for_each(
            |(dependencies, location, fragment_builder, check_condition)| {
                fragment.with_part(
                    Conditional::new(document, check_condition, fragment_builder.build(document)),
                    &dependencies,
                    location,
                );
            },
        );

        self.each_blocks
            .into_iter()
            .for_each(|(dependencies, location, get_items)| {
                fragment.with_part(Each::new(document, get_items), &dependencies, location);
            });

        fragment
    }
}

pub struct Fragment<Ctx> {
    document: Document,

    parts: Vec<(Option<usize>, Box<dyn Part<Ctx>>)>,

    pieces: Vec<(Option<usize>, Node)>,

    dependencies: HashMapList<usize, usize>,

    mounted: bool,
}

impl<Ctx> Fragment<Ctx>
where
    Ctx: 'static,
{
    pub fn build() -> FragmentBuilder<Ctx> {
        FragmentBuilder::new()
    }

    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),

            parts: Vec::new(),

            pieces: Vec::new(),

            dependencies: HashMapList::new(),

            mounted: false,
        }
    }

    /// Inserts a [Piece] into the fragment, which can include text or an element.
    pub fn with_piece(&mut self, kind: Kind, location: Option<usize>) {
        let piece = kind.create_node(&self.document);

        self.pieces.push((location, piece));
    }

    fn with_part<P>(&mut self, part: P, dependencies: &[usize], location: Option<usize>) -> usize
    where
        P: 'static + Part<Ctx>,
    {
        let id = self.parts.len();

        self.parts
            .push((location, Box::new(part) as Box<dyn Part<Ctx>>));
        self.register_dependencies(id, dependencies);

        id
    }

    // Re-export trait methods to make easier for user
    pub fn mount(&mut self, location: &ResolvedLocation) {
        Part::mount(self, location);
    }
    pub fn update(&mut self, context: &Ctx, changed: &[usize]) {
        Part::update(self, context, changed);
    }
    pub fn detach(&mut self, top_level: bool) {
        Part::detach(self, top_level);
    }

    pub fn full_update(&mut self, context: &Ctx) {
        Part::update(
            self,
            context,
            self.dependencies
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }

    /// Register dependencies.
    fn register_dependencies(&mut self, id: usize, dependencies: &[usize]) {
        for dependency in dependencies {
            self.dependencies.insert(*dependency, id);
        }
    }
}

impl<Ctx> Part<Ctx> for Fragment<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &ResolvedLocation) {
        self.pieces.iter().for_each(|(parent_id, node)| {
            parent_id
                .map(|parent_id| {
                    ResolvedLocation::parent(
                        &self.pieces.get(parent_id).expect("location to exist").1,
                    )
                })
                .unwrap_or(location.clone())
                .mount(node);
        });

        self.parts.iter_mut().for_each(|(parent_id, part)| {
            part.mount(
                &parent_id
                    .map(|parent_id| {
                        ResolvedLocation::parent(
                            &self.pieces.get(parent_id).expect("location to exist").1,
                        )
                    })
                    .unwrap_or(location.clone()),
            )
        });

        self.mounted = true;
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        if self.mounted {
            self.parts
                .iter_mut()
                .for_each(|(_, part)| part.update(context, changed));
        }
    }

    fn detach(&mut self, top_level: bool) {
        self.pieces.iter().for_each(|(_, node)| {
            node.parent_node()
                .expect("node to have parent")
                .remove_child(node);
        });

        self.parts
            .iter_mut()
            .for_each(|(_, part)| part.detach(top_level));

        self.mounted = false;
    }
}
