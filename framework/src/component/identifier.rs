#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Identifier(Vec<usize>);

impl Identifier {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn sibling(&self) -> Self {
        let mut sibling = self.clone();

        if let Some(id) = sibling.0.last_mut() {
            *id += 1;
        }

        sibling
    }

    pub fn child(&self, index: usize) -> Self {
        let mut child = self.clone();
        child.0.push(index);
        child
    }
}

impl<A> From<A> for Identifier
where
    A: AsRef<[usize]>,
    Vec<usize>: From<A>,
{
    fn from(value: A) -> Self {
        Self(value.into())
    }
}

impl AsRef<[usize]> for Identifier {
    fn as_ref(&self) -> &[usize] {
        self.0.as_ref()
    }
}
