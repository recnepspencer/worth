//! Borrowed canonical membership over persistent, independently funded cells.
use super::{ExecutionOrderedMap, PartitionItemId};

/// Read-only membership. Cloning shares cells and their original grants;
/// iteration does not materialize a set or acquire successful-result authority.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct PartitionMembers(pub(super) ExecutionOrderedMap<PartitionItemId, ()>);
impl PartitionMembers {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn contains(&self, item: &PartitionItemId) -> bool {
        self.0.contains_key(item)
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &PartitionItemId> {
        self.0.keys()
    }
}
