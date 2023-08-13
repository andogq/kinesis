use std::{cell::RefCell, rc::Rc};

use web_sys::Document;

use super::{
    dom_renderable::{GetIterFn, GetTextFn, Iterator, Updatable},
    EventRegistry, Fragment, Node,
};

/// Builder for a [`super::Piece`].
pub struct PieceBuilder {
    kind: Node,
    location: Option<usize>,
}

/// Builder for a [`Updatable`].
pub struct UpdatableBuilder<Ctx> {
    get_text: GetTextFn<Ctx>,
}

impl<Ctx> UpdatableBuilder<Ctx> {
    pub fn build(self, document: &Document) -> Updatable<Ctx> {
        Updatable::new(document, self.get_text)
    }
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
    pieces: Vec<PieceBuilder>,
    updatables: Vec<Builder<UpdatableBuilder<Ctx>>>,
    iterators: Vec<Builder<IteratorBuilder<Ctx>>>,
}

impl<Ctx> FragmentBuilder<Ctx>
where
    Ctx: 'static,
{
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            updatables: Vec::new(),
            iterators: Vec::new(),
        }
    }

    /// Add a [`PieceBuilder`] to the builder.
    pub fn with_piece(mut self, kind: Node, location: Option<usize>) -> Self {
        self.pieces.push(PieceBuilder { kind, location });
        self
    }

    /// Add a [`UpdatableBuilder`] to the builder.
    pub fn with_updatable<F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
        get_text: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> String,
    {
        self.updatables.push(Builder::new(
            dependencies,
            location,
            UpdatableBuilder {
                get_text: Box::new(get_text) as GetTextFn<Ctx>,
            },
        ));
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

        self.pieces
            .into_iter()
            .for_each(|PieceBuilder { kind, location }| fragment.with_static_node(kind, location));

        self.updatables.into_iter().for_each(
            |Builder {
                 dependencies,
                 location,
                 builder,
             }| {
                fragment.with_renderable(builder.build(document), &dependencies, location);
            },
        );

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
