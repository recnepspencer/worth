use super::node::MapNode;

pub(crate) struct SharedMapIter<'a, K: Ord + Copy, V: Clone> {
    pending: Vec<&'a MapNode<K, V>>,
    remaining: usize,
}

impl<'a, K: Ord + Copy, V: Clone> SharedMapIter<'a, K, V> {
    pub(super) fn new(root: Option<&'a MapNode<K, V>>) -> Self {
        let mut iterator = Self {
            pending: Vec::new(),
            remaining: root.map_or(0, |node| node.len),
        };
        iterator.descend(root);
        iterator
    }
    fn descend(&mut self, mut current: Option<&'a MapNode<K, V>>) {
        while let Some(node) = current {
            self.pending.push(node);
            current = node.left.as_deref();
        }
    }
}

impl<'a, K: Ord + Copy, V: Clone> Iterator for SharedMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.pending.pop()?;
        self.descend(node.right.as_deref());
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<K: Ord + Copy, V: Clone> ExactSizeIterator for SharedMapIter<'_, K, V> {}
