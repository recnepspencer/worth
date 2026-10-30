use std::{cmp::Ordering, sync::Arc};
use worth_execution::ExecutionArray;

/// An immutable typed row borrows its full value from admitted native backing.
/// Metadata may share; no T/ClientKey/String copy occurs during run replacement.
pub(super) struct StoredRow<T> {
    backing: Arc<ExecutionArray<T>>,
    index: usize,
}
impl<T> StoredRow<T> {
    pub(super) fn new(backing: Arc<ExecutionArray<T>>, index: usize) -> Self {
        Self { backing, index }
    }
    pub(super) fn value(&self) -> &T {
        &self.backing[self.index]
    }
}
impl<T> Clone for StoredRow<T> {
    fn clone(&self) -> Self {
        Self {
            backing: Arc::clone(&self.backing),
            index: self.index,
        }
    }
}
impl<T: Ord> PartialEq for StoredRow<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<T: Ord> Eq for StoredRow<T> {}
impl<T: Ord> PartialOrd for StoredRow<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord> Ord for StoredRow<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}
