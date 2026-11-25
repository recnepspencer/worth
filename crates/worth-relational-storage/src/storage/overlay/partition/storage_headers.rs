//! Inline map values share the already-owned native columns and touched sets.
use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::{StorageAllocationObservation, StoredValue, StoredValueSealed};

macro_rules! inline_header {
    ($header:ty) => {
        impl StoredValueSealed for $header {}
        impl StoredValue for $header {
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
            // A conservative nonmatch never suppresses a real owner replacement.
            fn same_storage(&self, other: &Self) -> bool {
                std::ptr::eq(self, other)
            }
            fn allocation(&self, _: bool) -> Option<StorageAllocationObservation> {
                None
            }
            fn allocation_bytes() -> u64 {
                0
            }
        }
    };
}
inline_header!(PartitionState);
inline_header!(PartitionMutationJournal);
