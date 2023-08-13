use std::{collections::HashMap, hash::Hash};

#[derive(Default)]
pub struct Tree<K, V>(pub HashMap<K, TreeNode<K, V>>);

#[derive(Default)]
pub struct TreeNode<K, V> {
    /// Value associated with this node in the tree.
    pub value: Option<V>,
    /// Children belonging to this node.
    pub children: Tree<K, V>,
}

impl<K, V> Tree<K, V>
where
    K: Eq + PartialEq + Hash,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Attempt to traverse the tree with a path, and retrieve a reference to the value at that
    /// location.
    pub fn get(&self, path: &[K]) -> Option<&V> {
        if path.is_empty() {
            None
        } else {
            let node = self.0.get(&path[0])?;

            if path.len() == 1 {
                node.value.as_ref()
            } else {
                node.children.get(&path[1..])
            }
        }
    }

    pub fn get_node(&mut self, path: &[K]) -> Option<&mut TreeNode<K, V>> {
        if path.is_empty() {
            None
        } else {
            let node = self.0.get_mut(&path[0])?;

            if path.len() == 1 {
                Some(node)
            } else {
                node.children.get_node(&path[1..])
            }
        }
    }

    pub fn insert<P>(&mut self, path: P, value: V) -> Result<(), ()>
    where
        P: IntoIterator<Item = K>,
    {
        let mut path = path.into_iter();
        let next_key = path.next().ok_or(())?;
        let path = path.collect::<Vec<_>>();

        if !path.is_empty() {
            // Recurse and attempt to insert at the next node
            self.0
                .entry(next_key)
                .or_insert_with(|| TreeNode::empty())
                .children
                .insert(path, value)
        } else {
            // Insert node into this tree (WARN: Can overwrite whole branches of the tree)
            self.0.insert(next_key, TreeNode::new(value));
            Ok(())
        }
    }

    pub fn remove<P>(&mut self, path: P) -> Result<TreeNode<K, V>, ()>
    where
        P: IntoIterator<Item = K>,
    {
        let mut path = path.into_iter();
        let next_key = path.next().ok_or(())?;
        let path = path.collect::<Vec<_>>();

        if !path.is_empty() {
            // Recurse and attempt to remove at the next node
            self.0
                .entry(next_key)
                // TODO: Kinda questionable
                .or_insert_with(|| TreeNode::empty())
                .children
                .remove(path)
        } else {
            // Remove node at this tree
            self.0.remove(&next_key).ok_or(())
        }
    }
}

impl<K, V> TreeNode<K, V>
where
    K: Eq + PartialEq + Hash,
{
    pub fn new(value: V) -> Self {
        Self {
            value: Some(value),
            children: Tree::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: None,
            children: Tree::new(),
        }
    }
}

impl<K, V> IntoIterator for TreeNode<K, V> {
    type Item = V;
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

pub struct IntoIter<K, V> {
    queue: Vec<TreeNode<K, V>>,
}

impl<K, V> IntoIter<K, V> {
    pub fn new(node: TreeNode<K, V>) -> Self {
        Self { queue: vec![node] }
    }
}
impl<K, V> Iterator for IntoIter<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.queue.pop()?;

        // Add children to the queue
        self.queue.extend(next.children.0.into_values());

        // Return the value
        next.value.or_else(|| self.next())
    }
}
