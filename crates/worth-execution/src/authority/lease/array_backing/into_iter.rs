use super::super::fixed_backing::OwnedFixedIntoIter;
use std::iter::FusedIterator;

/// Moves elements in their authored order without allocating or cloning T.
/// Keeps the original physical allocation and ticket until this iterator drops,
/// even after exhaustion. Moved-out values own their separate nested resources.
/// Non-ZST forward/backward traversal uses the pinned owning Vec iterator.
/// Zero-sized values have no distinct stored representation: safe Vec pop serves
/// both directions with the exact remaining count and the original aligned view.
pub struct ExecutionArrayIntoIter<T> {
    backing: OwnedFixedIntoIter<T>,
}

impl<T> ExecutionArrayIntoIter<T> {
    pub(super) fn from_owned(backing: OwnedFixedIntoIter<T>) -> Self {
        Self { backing }
    }
    pub fn elements(&self) -> &[T] {
        self.backing.elements()
    }
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.backing.charged_payload_bytes()
    }
}
impl<T> Iterator for ExecutionArrayIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.backing.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.backing.size_hint()
    }
}
impl<T> DoubleEndedIterator for ExecutionArrayIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.backing.next_back()
    }
}
impl<T> ExactSizeIterator for ExecutionArrayIntoIter<T> {}
impl<T> FusedIterator for ExecutionArrayIntoIter<T> {}
impl<T> std::fmt::Debug for ExecutionArrayIntoIter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionArrayIntoIter")
            .field("remaining_elements", &self.len())
            .finish_non_exhaustive()
    }
}
