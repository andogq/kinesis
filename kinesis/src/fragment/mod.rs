mod builder;
mod dom_renderable;
mod util;

use crate::util::HashMapList;
pub use builder::*;
pub use dom_renderable::DomRenderable;
pub use util::*;
use web_sys::{Document, Node as WsNode};

/// A top level representation of a fragment. Can contain static data, conditional fragments and
/// looped fragments. Is responsible for mounting/updating/detaching itself and all children.
/// Importantly, it will only contain static nodes, meaning that there are no variable nodes
/// conditionally being mounted/unmounted within the fragment. If there are any dynamic nodes, a
/// new fragment must be created.
pub struct Fragment<Ctx> {
    /// A reference to the [`Document`].
    document: Document,

    /// Collection of all renderables (eg [`dom_renderable::Conditional`], [`dom_renderable::Each`], ect).
    renderables: Vec<(Option<usize>, Box<dyn DomRenderable<Ctx>>)>,

    /// Collection of static [`web_sys::Node`]s, and a reference to the static node that it should
    /// be mounted in.
    static_nodes: Vec<(Option<usize>, WsNode)>,

    /// Collection mapping between context properties (key), and the renderables that rely on it
    /// (value).
    dependencies: HashMapList<usize, usize>,

    /// Whether the fragment is currently mounted or not.
    mounted: bool,
}

impl<Ctx> Fragment<Ctx>
where
    Ctx: 'static,
{
    /// Create a new [`FragmentBuilder`].
    pub fn build() -> FragmentBuilder<Ctx> {
        FragmentBuilder::new()
    }

    /// Create a new fragment. Requires a reference to [`Document`] in order to store for future
    /// usage, so that [`web_sys::Node`]s can be created as required.
    pub fn new(document: &Document) -> Self {
        Self {
            document: document.clone(),

            renderables: Vec::new(),

            static_nodes: Vec::new(),

            dependencies: HashMapList::new(),

            mounted: false,
        }
    }

    /// Inserts a static node into the fragment.
    pub fn with_static_node(&mut self, kind: Node, location: Option<usize>) {
        let piece = kind.create_node(&self.document);

        self.static_nodes.push((location, piece));
    }

    /// Inserts a dynamic [`DomRenderable`] into the fragment.
    pub(super) fn with_renderable<P>(
        &mut self,
        part: P,
        dependencies: &[usize],
        location: Option<usize>,
    ) -> usize
    where
        P: 'static + DomRenderable<Ctx>,
    {
        let id = self.renderables.len();

        self.renderables
            .push((location, Box::new(part) as Box<dyn DomRenderable<Ctx>>));
        self.register_dependencies(id, dependencies);

        id
    }

    /// Performs a full update on the fragment.
    ///
    /// This uses the [`DomRenderable::update()`] method, a generated dependency list based off of
    /// the registered dependencies of [`DomRenderable`]s.
    pub fn full_update(&mut self, context: &Ctx) {
        DomRenderable::update(
            self,
            context,
            self.dependencies
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }

    /// Helper function to rgister dependencies.
    fn register_dependencies(&mut self, id: usize, dependencies: &[usize]) {
        for dependency in dependencies {
            self.dependencies.insert(*dependency, id);
        }
    }
}

impl<Ctx> DomRenderable<Ctx> for Fragment<Ctx>
where
    Ctx: 'static,
{
    fn mount(&mut self, location: &Location) {
        self.static_nodes.iter().for_each(|(parent_id, node)| {
            parent_id
                .map(|parent_id| {
                    Location::parent(
                        &self
                            .static_nodes
                            .get(parent_id)
                            .expect("location to exist")
                            .1,
                    )
                })
                .unwrap_or(location.clone())
                .mount(node);
        });

        self.renderables.iter_mut().for_each(|(parent_id, part)| {
            part.mount(
                &parent_id
                    .map(|parent_id| {
                        Location::parent(
                            &self
                                .static_nodes
                                .get(parent_id)
                                .expect("location to exist")
                                .1,
                        )
                    })
                    .unwrap_or(location.clone()),
            )
        });

        self.mounted = true;
    }

    fn update(&mut self, context: &Ctx, changed: &[usize]) {
        if self.mounted {
            self.renderables
                .iter_mut()
                .for_each(|(_, part)| part.update(context, changed));
        }
    }

    fn detach(&mut self, top_level: bool) {
        self.static_nodes.iter().for_each(|(_, node)| {
            node.parent_node()
                .expect("node to have parent")
                .remove_child(node);
        });

        self.renderables
            .iter_mut()
            .for_each(|(_, part)| part.detach(top_level));

        self.mounted = false;
    }
}
