use crate::{
    component::Component,
    fragment::{Fragment, FragmentBuilder, Node},
};

#[derive(Default)]
pub struct Simple {
    count: usize,
}

impl Simple {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Component for Simple {
    type Ctx = Self;

    fn handle_event(&mut self, event_id: usize) -> Option<Vec<usize>> {
        match event_id {
            0 => {
                self.count -= 1;
                Some(vec![0])
            }
            1 => {
                self.count += 1;
                Some(vec![0])
            }
            _ => None,
        }
    }

    fn render(&self) -> FragmentBuilder<Self::Ctx> {
        Fragment::build()
            .with_piece(Node::element("p"), None)
            .with_piece(Node::text("some content: "), Some(0))
            .with_updatable(&[0], Some(0), |ctx: &Simple| ctx.count.to_string())
            .with_piece(Node::element("button").with_event("click", 0), None)
            .with_piece(Node::text("decrement"), Some(2))
            .with_piece(Node::element("button").with_event("click", 1), None)
            .with_piece(Node::text("increment"), Some(4))
            .with_conditional(
                &[0],
                None,
                Fragment::build()
                    .with_piece(Node::element("p"), None)
                    .with_piece(Node::text("showing!"), Some(0)),
                |ctx| ctx.count % 2 == 0,
            )
            .with_each(&[0], None, |ctx| {
                Box::new((0..ctx.count).map(|val| {
                    Fragment::build()
                        .with_piece(Node::element("p"), None)
                        .with_piece(Node::text(format!("counting {val}")), Some(0))
                })) as Box<dyn Iterator<Item = FragmentBuilder<Self::Ctx>>>
            })
    }
}
