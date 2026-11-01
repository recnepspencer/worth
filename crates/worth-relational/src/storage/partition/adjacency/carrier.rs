//! The exact adjacency header is shallow; only its map nodes allocate.
use super::{AdjacencyEntries, AdjacencySet};
use crate::storage::substrate::{
    SharedMap, StorageAllocationObservation, StoredValue, StoredValueSealed,
};

impl StoredValueSealed for AdjacencySet {}
impl StoredValue for AdjacencySet {
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
        match (self, other) {
            (Self::Inline(left), Self::Inline(right))
            | (Self::Compressed(left), Self::Compressed(right)) => same_entries(left, right),
            _ => false,
        }
    }
    fn allocation(&self, _: bool) -> Option<StorageAllocationObservation> {
        None
    }
    fn allocation_bytes() -> u64 {
        0
    }
}
fn same_entries(left: &AdjacencyEntries, right: &AdjacencyEntries) -> bool {
    left.current.same_storage(&right.current)
        && same_optional(&left.current_by_kind, &right.current_by_kind)
        && same_optional(&left.historical_by_kind, &right.historical_by_kind)
        && same_optional(
            &left.structural_revision_by_kind,
            &right.structural_revision_by_kind,
        )
}
fn same_optional<K: Ord + Copy, V: StoredValue>(
    left: &Option<SharedMap<K, V>>,
    right: &Option<SharedMap<K, V>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.same_storage(right),
        _ => false,
    }
}
