use crate::{
    component::Component,
    fragment::{Fragment, FragmentBuilder, Node},
};
use web_sys::Event;

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

    fn handle_event(&mut self, event_id: usize, _event: Event) -> Option<Vec<usize>> {
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
        Fragment::<Self>::build()
            .with_element("p", None)
            .with_text("some content: ", Some(0))
            .with_updatable(&[0], Some(0), |ctx| {
                Fragment::build().with_text(ctx.count.to_string(), None)
            })
            .with_node(Node::element("button").with_event("click", 0), None)
            .with_text("decrement", Some(2))
            .with_node(Node::element("button").with_event("click", 1), None)
            .with_text("increment", Some(4))
            .with_conditional(
                &[0],
                None,
                |ctx| ctx.count % 2 == 0,
                |_ctx| {
                    Fragment::build()
                        .with_element("p", None)
                        .with_text("showing!", Some(0))
                },
            )
            .with_iter(&[0], None, |ctx| {
                Box::new((0..ctx.count).map(|val| {
                    Fragment::build()
                        .with_element("p", None)
                        .with_text(format!("counting {val}"), Some(0))
                })) as Box<dyn Iterator<Item = FragmentBuilder<Self>>>
            })
    }

    fn get_context(&self) -> &Self::Ctx {
        self
    }
}
