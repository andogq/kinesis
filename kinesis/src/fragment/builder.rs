use web_sys::Document;

use super::{
    dom_renderable::{CheckConditionFn, Conditional, Each, GetItemsFn, GetTextFn, Updatable},
    Fragment, Node,
};

/// Builder for a [`super::Piece`].
pub struct PieceBuilder {
    kind: Node,
    location: Option<usize>,
}

/// Builder for a [`super::dom_renderable::Updatable`].
pub struct UpdatableBuilder<Ctx> {
    get_text: GetTextFn<Ctx>,
}

impl<Ctx> UpdatableBuilder<Ctx> {
    pub fn build(self, document: &Document) -> Updatable<Ctx> {
        Updatable::new(document, self.get_text)
    }
}

/// Builder for a [`super::dom_renderable::Conditional`].
pub struct ConditionalBuilder<Ctx> {
    fragment_builder: FragmentBuilder<Ctx>,
    check_condition: CheckConditionFn<Ctx>,
}

impl<Ctx> ConditionalBuilder<Ctx>
where
    Ctx: 'static,
{
    pub fn build(self, document: &Document) -> Conditional<Ctx> {
        Conditional::new(
            document,
            self.check_condition,
            self.fragment_builder.build(document),
        )
    }
}

/// Builder for a [`super::dom_renderable::Each`].
pub struct EachBuilder<Ctx> {
    get_items: GetItemsFn<Ctx>,
}

impl<Ctx> EachBuilder<Ctx>
where
    Ctx: 'static,
{
    pub fn build(self, document: &Document) -> Each<Ctx> {
        Each::new(document, self.get_items)
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
    conditionals: Vec<Builder<ConditionalBuilder<Ctx>>>,
    each_blocks: Vec<Builder<EachBuilder<Ctx>>>,
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
            conditionals: Vec::new(),
            each_blocks: Vec::new(),
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

    /// Add a [`ConditionalBuilder`] to the builder.
    pub fn with_conditional<F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
        fragment_builder: FragmentBuilder<Ctx>,
        check_condition: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> bool,
    {
        self.conditionals.push(Builder::new(
            dependencies,
            location,
            ConditionalBuilder {
                fragment_builder,
                check_condition: Box::new(check_condition) as CheckConditionFn<Ctx>,
            },
        ));
        self
    }

    /// Add a [`EachBuilder`] to the builder.
    pub fn with_each<F>(
        mut self,
        dependencies: &[usize],
        location: Option<usize>,
        get_items: F,
    ) -> Self
    where
        F: 'static + Fn(&Ctx) -> Box<dyn Iterator<Item = FragmentBuilder<Ctx>>>,
    {
        self.each_blocks.push(Builder::new(
            dependencies,
            location,
            EachBuilder {
                get_items: Box::new(get_items) as GetItemsFn<Ctx>,
            },
        ));
        self
    }

    /// Use the reference to [`Document`] to build all of the renderables within this fragment
    /// builder. Returns the constructed fragment.
    pub fn build(self, document: &Document) -> Fragment<Ctx> {
        let mut fragment = Fragment::new(document);

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

        self.conditionals.into_iter().for_each(
            |Builder {
                 dependencies,
                 location,
                 builder,
             }| {
                fragment.with_renderable(builder.build(document), &dependencies, location);
            },
        );

        self.each_blocks.into_iter().for_each(
            |Builder {
                 dependencies,
                 location,
                 builder,
             }| {
                fragment.with_renderable(builder.build(document), &dependencies, location);
            },
        );

        fragment
    }
}
