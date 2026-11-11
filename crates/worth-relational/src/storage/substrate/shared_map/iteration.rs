use super::node::MapNode;

pub(crate) struct SharedMapIter<'a, K: Ord + Copy, V: super::StoredValue> {
    pending: [Option<&'a MapNode<K, V>>; MAXIMUM_ADDRESSABLE_AVL_HEIGHT],
    pending_len: usize,
    remaining: usize,
}

impl<'a, K: Ord + Copy, V: super::StoredValue> SharedMapIter<'a, K, V> {
    pub(super) fn from_key(root: Option<&'a MapNode<K, V>>, first: &K) -> Self {
        let mut iterator = Self {
            pending: [None; MAXIMUM_ADDRESSABLE_AVL_HEIGHT],
            pending_len: 0,
            remaining: root.map_or(0, |node| node.len),
        };
        let mut next = root;
        while let Some(node) = next {
            if node.key < *first {
                iterator.remaining -= 1 + node.left.as_ref().map_or(0, |left| left.len);
                next = node.right.as_deref();
            } else {
                iterator.pending[iterator.pending_len] = Some(node);
                iterator.pending_len += 1;
                next = node.left.as_deref();
            }
        }
        iterator
    }
    pub(super) fn new(root: Option<&'a MapNode<K, V>>) -> Self {
        let mut iterator = Self {
            pending: [None; MAXIMUM_ADDRESSABLE_AVL_HEIGHT],
            pending_len: 0,
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
        prepare(height as u64 + 1, 0)?;
        Ok(Self::new(root))
    }
    pub(crate) fn try_next<Stop>(
        &mut self,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Option<(&'a K, &'a V::Value)>, Stop> {
        let Some(index) = self.pending_len.checked_sub(1) else {
            return Ok(None);
        };
        let next =
            self.pending[index].expect(/* Proof: descend initializes each prefix slot before incrementing pending_len. Only advance takes a slot, after decrementing that same length, so a live prefix slot is always Some. */ "the nonempty pending prefix contains its original node");
        let descent = next.right.as_ref().map_or(0, |right| right.height);
        prepare(descent as u64 + 1, 0)?;
        Ok(self.advance())
    }
    fn advance(&mut self) -> Option<(&'a K, &'a V::Value)> {
        self.pending_len = self.pending_len.checked_sub(1)?;
        let node = self.pending[self.pending_len]
            .take()
            .expect(/* Proof: descend initializes each prefix slot before incrementing pending_len. Only advance takes a slot, after decrementing that same length, so a live prefix slot is always Some. */ "the nonempty pending prefix contains its original node");
        self.descend(node.right.as_deref());
        self.remaining -= 1;
        Some((&node.key, node.value.value()))
    }
    fn descend(&mut self, mut current: Option<&'a MapNode<K, V>>) {
        while let Some(node) = current {
            self.pending[self.pending_len] = Some(node);
            self.pending_len += 1;
            current = node.left.as_deref();
        }
    }
}

impl<'a, K: Ord + Copy, V: super::StoredValue> Iterator for SharedMapIter<'a, K, V> {
    type Item = (&'a K, &'a V::Value);
    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<K: Ord + Copy, V: super::StoredValue> ExactSizeIterator for SharedMapIter<'_, K, V> {}

// Every two AVL levels at least double the minimum node count plus one.
// A tree whose length fits in usize therefore cannot exceed this height.
// The pending prefix contains at most one path through that admitted topology;
// its inline slots need no growth or heap reservation, including for empty maps.
const MAXIMUM_ADDRESSABLE_AVL_HEIGHT: usize = 2 * usize::BITS as usize;

#[cfg(test)]
mod tests;
