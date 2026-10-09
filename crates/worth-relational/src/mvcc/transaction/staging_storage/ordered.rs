use super::{author::Author, iter::OrderedIter};
use crate::mvcc::RelationalTransactionStagingDenial as Denial;
use std::sync::Arc;
use worth_execution::{ExecutionAllocationPolicy, ExecutionArray};

pub(super) type Run<T> = Arc<ExecutionArray<super::row::StoredRow<T>>>;
pub(super) type Directory<T> = ExecutionArray<Option<Run<T>>>;

/// Immutable ordered unique rows; Clone shares every physical backing.
/// Full typed T ordering is authoritative. Nested T heaps are not admitted here.
pub(crate) struct OrderedStore<T> {
    pub(super) directory: Option<Arc<Directory<T>>>,
    pub(super) units: usize,
    pub(super) len: usize,
}
impl<T> Default for OrderedStore<T> {
    fn default() -> Self {
        Self {
            directory: None,
            units: 0,
            len: 0,
        }
    }
}
impl<T> Clone for OrderedStore<T> {
    fn clone(&self) -> Self {
        Self {
            directory: self.directory.clone(),
            units: self.units,
            len: self.len,
        }
    }
}
impl<T: Ord> OrderedStore<T> {
    pub(crate) fn iter(&self) -> OrderedIter<'_, T> {
        self.range_by(|_| std::cmp::Ordering::Equal)
    }
    pub(crate) fn range_by(
        &self,
        compare: impl Fn(&T) -> std::cmp::Ordering,
    ) -> OrderedIter<'_, T> {
        OrderedIter::new(self.runs(), compare)
    }
    pub(crate) fn with_inserted(
        &self,
        values: impl IntoIterator<Item = T>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut author = Author::new(self, policy)?;
        for value in values {
            author.insert(value)?;
        }
        author.finish()
    }
    pub(super) fn runs(&self) -> &[Option<Run<T>>] {
        self.directory.as_deref().map_or(&[], |rows| rows.as_ref())
    }
}
impl<T: Ord + std::fmt::Debug> std::fmt::Debug for OrderedStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T: Ord> PartialEq for OrderedStore<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}
impl<T: Ord> Eq for OrderedStore<T> {}
impl<'a, T: Ord> IntoIterator for &'a OrderedStore<T> {
    type Item = &'a T;
    type IntoIter = OrderedIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
