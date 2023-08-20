use std::{cell::RefCell, iter, rc::Rc};

use web_sys::Document;

use crate::component::{Component, Controller};

use super::{
    dynamic::{GetIterFn, Iterator, UpdateProxy, UpdateProxyFn},
    Dynamic, EventRegistry, Fragment, Node,
};

/// Builder for a [`super::Node`].
pub struct NodeBuilder {
    node: Node,
    location: Option<usize>,
}

/// Wrapper types for builders, containing common information between the builders.
pub struct Builder {
    /// A list of dependencies that the built result will rely on.
    dependencies: Vec<usize>,

    /// The ID of the parent to append the built result to. If [`Option::None`], will be appended
    /// to the root of the fragment.
    location: Option<usize>,

    /// The builder with specific fields.
    builder: Box<dyn DynamicBuilder>,
}

trait DynamicBuilder {
    fn build(
        self: Box<Self>,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Box<dyn Dynamic>;
}

/// Builder for a [`Iterator`].
pub struct IteratorBuilder {
    /// A function that will return an iterator of [`FragmentBuilder`]s.
    get_items: GetIterFn,
}

impl DynamicBuilder for IteratorBuilder {
    fn build(
        self: Box<Self>,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Box<dyn Dynamic> {
        Box::new(Iterator::new(document, self.get_items, event_registry))
    }
}

pub struct ControllerBuilder {
    component: Rc<RefCell<dyn Component>>,
    fragment_builder: FragmentBuilder,
    map_changed: Box<UpdateProxyFn>,
}

impl DynamicBuilder for ControllerBuilder {
    fn build(
        self: Box<Self>,
        document: &Document,
        _event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Box<dyn Dynamic> {
        Box::new(UpdateProxy::new(
            Controller::new(document, (self.component, self.fragment_builder)),
            self.map_changed,
        ))
    }
}

/// Used to build and represent a [`Fragment`] that does not yet have access to the [Document].
/// Contains a collection of each of the possible builders.
pub struct FragmentBuilder {
    /// Static nodes to be rendered within this fragment.
    nodes: Vec<NodeBuilder>,

    dynamic: Vec<Builder>,
}

impl FragmentBuilder {
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            dynamic: Vec::new(),
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
        F: 'static + Fn() -> Box<dyn std::iter::Iterator<Item = FragmentBuilder>>,
    {
        self.dynamic.push(Builder {
            dependencies: dependencies.to_vec(),
            location,
            builder: Box::new(IteratorBuilder {
                get_items: Box::new(get_items) as GetIterFn,
            }),
        });
        self
    }

    pub fn with_component<C, F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
        (component, fragment_builder): (Rc<RefCell<C>>, FragmentBuilder),
        update: F,
    ) -> Self
    where
        C: Component + 'static,
        F: 'static + Fn(&[usize]) -> Vec<usize>,
    {
        self.dynamic.push(Builder {
            dependencies: dependencies.to_vec(),
            location,
            builder: Box::new(ControllerBuilder {
                component: component as Rc<RefCell<dyn Component>>,
                fragment_builder,
                map_changed: Box::new(update),
            }),
        });

        self
    }

    /// Helper function to create an 'updatable' fragment, meaning a fragment that is re-rendered
    /// whenever a dependency changes. This creates an [`iter::Iterator`], as with
    /// [`Self::with_iter()`].
    pub fn with_updatable<F>(
        self,
        dependencies: &[usize],
        location: Option<usize>,
        get_fragment: F,
    ) -> Self
    where
        F: 'static + Fn() -> FragmentBuilder,
    {
        self.with_iter(dependencies, location, move || {
            Box::new(iter::once(get_fragment()))
        })
    }

    /// Helper function to create a 'conditional' fragment, meaning a fragment that may be
    /// re-rendered whenever a dependency changes. Will handle the mounting/unmounting of the
    /// component depending on some condition that is passed in. This utilises [`bool::then()`] to
    /// create an [`Option`] containing the built fragment.
    pub fn with_conditional<F, B>(
        self,
        dependencies: &[usize],
        location: Option<usize>,
        check_condition: F,
        build_fragment: B,
    ) -> Self
    where
        F: 'static + Fn() -> bool,
        B: 'static + Fn() -> FragmentBuilder,
    {
        self.with_iter(dependencies, location, move || {
            Box::new(check_condition().then(&build_fragment).into_iter())
        })
    }

    /// Helper function to add an element [`Node`].
    pub fn with_element(self, kind: impl AsRef<str>, location: Option<usize>) -> Self {
        self.with_node(Node::element(kind), location)
    }

    /// Helper function to add a text [`Node`].
    pub fn with_text(self, content: impl AsRef<str>, location: Option<usize>) -> Self {
        self.with_node(Node::text(content), location)
    }

    /// Use the reference to [`Document`] to build all of the renderables within this fragment
    /// builder. Returns the constructed fragment.
    pub fn build(
        self,
        document: &Document,
        event_registry: &Rc<RefCell<EventRegistry>>,
    ) -> Fragment {
        let mut fragment = Fragment::new(document, event_registry);

        self.nodes
            .into_iter()
            .for_each(|NodeBuilder { node, location }| fragment.with_static_node(node, location));

        self.dynamic.into_iter().for_each(
            |Builder {
                 dependencies,
                 location,
                 builder,
             }| {
                fragment.with_dynamic(
                    builder.build(document, event_registry),
                    &dependencies,
                    location,
                );
            },
        );

        fragment
    }
}
