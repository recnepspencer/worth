//! Exact digest header: only two original immutable child-owner handles.
use super::DigestIndex;
use crate::storage::substrate::{
    StorageAllocation, StorageAllocationObservation, StoredValue, StoredValueSealed,
};
impl StoredValueSealed for DigestIndex {}
impl StoredValue for DigestIndex {
    type Value = Self;
    fn from_value(value: Self) -> Self {
        value
    }
    fn value(&self) -> &Self {
        self
    }
    fn value_mut(&mut self) -> &mut Self {
        self
    }
    fn into_value(self) -> Self {
        self
    }
    fn same_storage(&self, other: &Self) -> bool {
        self.root.as_ref().map(StorageAllocation::id)
            == other.root.as_ref().map(StorageAllocation::id)
            && self.history_last_value.as_ref().map(StorageAllocation::id)
                == other.history_last_value.as_ref().map(StorageAllocation::id)
    }
    fn allocation(&self, _: bool) -> Option<StorageAllocationObservation> {
        None
    }
    fn allocation_bytes() -> u64 {
        0
    }
}
