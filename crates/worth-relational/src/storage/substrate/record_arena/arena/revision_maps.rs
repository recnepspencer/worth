//! Concrete shallow revision headers; only their persistent map nodes allocate.
use crate::storage::data::RelationalFieldRevision;
use crate::storage::substrate::{
    SharedMap, StorageAllocationObservation, StoredValue, StoredValueSealed,
};
use crate::symbols::data::Symbol;

pub(crate) type AspectVersions = SharedMap<Symbol, u64>;
pub(crate) type FieldRevisions = SharedMap<(Symbol, Symbol), RelationalFieldRevision>;

impl StoredValueSealed for RelationalFieldRevision {}
impl StoredValue for RelationalFieldRevision {
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
        self == other
    }
    fn allocation(&self, _: bool) -> Option<StorageAllocationObservation> {
        None
    }
    fn allocation_bytes() -> u64 {
        0
    }
}

// Absence means unavailable revision evidence, distinct from a present empty map.
// Copying either header shares its exact child nodes and never clones payload heaps.
impl StoredValueSealed for Option<FieldRevisions> {}
impl StoredValue for Option<FieldRevisions> {
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
            (None, None) => true,
            (Some(left), Some(right)) => left.same_storage(right),
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
