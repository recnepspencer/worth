use super::node::Node;
/// Ordered traversal uses a bounded inline stack: an AVL tree representable
/// by usize elements has height below twice the pointer width.
pub struct Iter<'a, K, V> {
    stack: [Option<&'a Node<K, V>>; 2 * usize::BITS as usize],
    used: usize,
    remaining: usize,
}
impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(root: Option<&'a Node<K, V>>, remaining: usize) -> Self {
        let mut result = Self {
            stack: [None; 2 * usize::BITS as usize],
            used: 0,
            remaining,
        };
        result.descend(root);
        result
    }
    fn descend(&mut self, mut next: Option<&'a Node<K, V>>) {
        while let Some(node) = next {
            self.stack[self.used] = Some(node);
            self.used += 1;
            next = node.left.as_deref();
        }
    }
}
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.used == 0 {
            return None;
        }
        self.used -= 1;
        let node = self.stack[self.used]
            .take()
            .expect("populated traversal path");
        self.remaining -= 1;
        self.descend(node.right.as_deref());
        Some((&node.entry.key, &node.entry.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> std::iter::FusedIterator for Iter<'_, K, V> {}
