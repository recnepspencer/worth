use super::node::MapNode;

pub(crate) struct SharedMapIter<'a, K: Ord + Copy, V: Clone> {
    pending: Vec<&'a MapNode<K, V>>,
    remaining: usize,
}

impl<'a, K: Ord + Copy, V: Clone> SharedMapIter<'a, K, V> {
    pub(super) fn new(root: Option<&'a MapNode<K, V>>) -> Self {
        let mut iterator = Self {
            pending: Vec::with_capacity(root.map_or(0, |_| MAXIMUM_ADDRESSABLE_AVL_HEIGHT)),
            remaining: root.map_or(0, |node| node.len),
        };
        iterator.descend(root);
        iterator
    }
    pub(super) fn try_new<Stop>(
        root: Option<&'a MapNode<K, V>>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, Stop> {
        let height = root.map_or(0, |node| node.height);
        let bytes = height.saturating_mul(std::mem::size_of::<&MapNode<K, V>>());
        prepare(
            u64::try_from(height).unwrap_or(u64::MAX).saturating_add(1),
            u64::try_from(bytes).unwrap_or(u64::MAX),
        )?;
        let mut iterator = Self {
            pending: Vec::with_capacity(height),
            remaining: root.map_or(0, |node| node.len),
        };
        iterator.descend(root);
        Ok(iterator)
    }
    pub(crate) fn try_next<Stop>(
        &mut self,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Option<(&'a K, &'a V)>, Stop> {
        let Some(next) = self.pending.last().copied() else {
            return Ok(None);
        };
        let descent = next.right.as_ref().map_or(0, |right| right.height);
        prepare(
            u64::try_from(descent).unwrap_or(u64::MAX).saturating_add(1),
            0,
        )?;
        let node = self
            .pending
            .pop()
            .expect("the admitted next node remains present");
        self.descend(node.right.as_deref());
        self.remaining -= 1;
        Ok(Some((&node.key, &node.value)))
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

// Every two AVL levels at least double the minimum node count plus one.
// A tree whose length fits in usize therefore cannot exceed this height.
// Reserve the addressable bound rather than the current height: unvisited
// fanout must not grow the allocation of an ordinary bounded traversal.
const MAXIMUM_ADDRESSABLE_AVL_HEIGHT: usize = 2 * usize::BITS as usize;
