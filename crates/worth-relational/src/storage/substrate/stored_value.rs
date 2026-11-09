//! Closed physical element carriers: scalars inline, independent payloads shared.
use super::{StorageAllocation, StorageAllocationObservation};
pub(super) mod sealed {
    pub trait Sealed {}
}

pub(crate) trait StoredValue: sealed::Sealed + Clone {
    type Value: Clone;
    fn from_value(value: Self::Value) -> Self;
    fn value(&self) -> &Self::Value;
    /// Ordinary image/maintenance access; controlled writes replace prepared carriers.
    fn value_mut(&mut self) -> &mut Self::Value;
    fn into_value(self) -> Self::Value;
    fn same_storage(&self, other: &Self) -> bool;
    fn allocation(&self, ancestor_unique: bool) -> Option<StorageAllocationObservation>;
    fn allocation_bytes() -> u64;
}

impl<T: Clone> sealed::Sealed for StorageAllocation<T> {}
impl<T: Clone> StoredValue for StorageAllocation<T> {
    type Value = T;
    fn from_value(value: T) -> Self {
        Self::new(value)
    }
    fn value(&self) -> &T {
        self
    }
    fn value_mut(&mut self) -> &mut T {
        Self::make_mut(self)
    }
    fn into_value(self) -> T {
        Self::unwrap_or_clone(self)
    }
    fn same_storage(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
    fn allocation(&self, ancestor_unique: bool) -> Option<StorageAllocationObservation> {
        Some(self.observation(ancestor_unique))
    }
    fn allocation_bytes() -> u64 {
        Self::layout_bytes()
    }
}

macro_rules! inline {
    ($($value:ty),* $(,)?) => { $(
        impl sealed::Sealed for $value {}
        impl StoredValue for $value {
            type Value = Self;
            fn from_value(value: Self) -> Self { value }
            fn value(&self) -> &Self { self }
            fn value_mut(&mut self) -> &mut Self { self }
            fn into_value(self) -> Self { self }
            fn same_storage(&self, other: &Self) -> bool { self == other }
            fn allocation(&self, _: bool) -> Option<StorageAllocationObservation> { None }
            fn allocation_bytes() -> u64 { 0 }
        }
    )* };
}
inline!(
    (),
    u8,
    u32,
    u64,
    usize,
    bool,
    crate::identity::data::PartitionId,
    crate::identity::data::VersionId,
    (crate::identity::data::KindId, u64),
    Option<crate::identity::data::KindId>,
    Option<crate::identity::data::VersionId>,
    crate::storage::data::RecordLifecycleState,
    Result<
        crate::storage::data::SymbolPageShape,
        crate::storage::data::RelationalStorageUnavailable,
    >,
);

impl<T: StoredValue> sealed::Sealed for super::SharedColumn<T> {}

/// An inline element inherits parent uniqueness but has no separate allocation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoredValueObservation {
    allocation: Option<StorageAllocationObservation>,
    pub(crate) unique: bool,
}
impl StoredValueObservation {
    pub(super) fn new<T: StoredValue>(value: &T, parent_unique: bool) -> Self {
        let allocation = value.allocation(parent_unique);
        Self {
            unique: allocation.map_or(parent_unique, |allocation| allocation.unique),
            allocation,
        }
    }
    pub(crate) fn visit(self, visitor: &mut dyn super::StorageAllocationVisitor) -> bool {
        self.allocation
            .is_none_or(|allocation| visitor.visit(allocation))
    }
    pub(crate) fn add_owned_bytes(&mut self, bytes: u64) {
        if let Some(allocation) = &mut self.allocation {
            allocation.bytes = allocation.bytes.saturating_add(bytes);
        } else {
            assert_eq!(
                bytes, 0,
                "inline shallow carriers have no separately owned payload heap"
            );
        }
    }
}

impl<K: Ord + Copy, V: StoredValue> sealed::Sealed for super::SharedMap<K, V> {}
