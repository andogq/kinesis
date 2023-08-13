use std::{cell::RefCell, rc::Rc};

use web_sys::Document;

use super::{
    dom_renderable::{GetIterFn, Iterator},
    EventRegistry, Fragment, Node,
};

/// Builder for a [`super::Node`].
pub struct NodeBuilder {
    node: Node,
    location: Option<usize>,
}

/// Builder for a [`Iterator`].
pub struct IteratorBuilder<Ctx> {
    get_items: GetIterFn<Ctx>,
}

impl<Ctx> IteratorBuilder<Ctx>
where
    Ctx: 'static,
{
    pub fn build(
        self,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Iterator<Ctx> {
        Iterator::new(document, self.get_items, event_registry)
    }
}

/// Wrapper types for builders, containing common information between the builders.
pub struct Builder<T> {
    /// A list of dependencies that the built result will rely on.
    dependencies: Vec<usize>,

    /// The ID of the parent to append the built result to. If [`Option::None`], will be appended
    /// to the root of the fragment.
    location: Option<usize>,

    /// The builder with specific fields.
    builder: T,
}

impl<T> Builder<T> {
    /// Create a new builder.
    pub fn new(dependencies: &[usize], location: Option<usize>, builder: T) -> Self {
        Self {
            dependencies: dependencies.to_vec(),
            location,
            builder,
        }
    }
}

/// Used to build and represent a [`Fragment`] that does not yet have access to the [Document].
/// Contains a collection of each of the possible builders.
pub struct FragmentBuilder<Ctx> {
    nodes: Vec<NodeBuilder>,
    iterators: Vec<Builder<IteratorBuilder<Ctx>>>,
}

impl<Ctx> FragmentBuilder<Ctx>
where
    Ctx: 'static,
{
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            iterators: Vec::new(),
        }
    }

    /// Add a [`NodeBuilder`] to the builder.
    pub fn with_node(mut self, node: Node, location: Option<usize>) -> Self {
        self.nodes.push(NodeBuilder { node, location });
        self
    }

    /// Add a [`IteratorBuilder`] to the builder.
    pub fn with_iter<F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
        get_items: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> Box<dyn std::iter::Iterator<Item = FragmentBuilder<Ctx>>>,
    {
        self.iterators.push(Builder::new(
            dependencies,
            location,
            IteratorBuilder {
                get_items: Box::new(get_items) as GetIterFn<Ctx>,
            },
        ));
        self
    }

    /// Use the reference to [`Document`] to build all of the renderables within this fragment
    /// builder. Returns the constructed fragment.
    pub fn build(
        self,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Fragment<Ctx> {
        let mut fragment = Fragment::new(document, event_registry);

        self.nodes
            .into_iter()
            .for_each(|NodeBuilder { node, location }| fragment.with_static_node(node, location));

        self.iterators.into_iter().for_each(
            |Builder {
                 dependencies,
                 location,
                 builder,
             }| {
                fragment.with_renderable(
                    builder.build(document, event_registry),
                    &dependencies,
                    location,
                );
            },
        );

        fragment
    }
}
