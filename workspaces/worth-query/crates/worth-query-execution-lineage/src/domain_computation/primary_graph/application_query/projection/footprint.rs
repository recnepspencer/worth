//! Projected typed row heap-footprint accounting so `#[derive(WorthQueryRow)]`
//! row structs (including nested `Vec<Child>`, `Option<Child>`, and custom
//! scalar domain types) and handwritten projections accurately report dynamic
//! heap bytes under `streaming_window_bytes` with zero silent zero-byte fallback.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    InlineBytes, InternedString,
};
use worth_query_declaration::facade::application_naming::{ErasedWorthQueryName, WorthQueryName};
use worth_query_declaration::facade::application_schema::{
    WorthQueryRecurringGate, WorthQueryRecurringLifecycle,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId, VersionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};

use super::WorthQueryApplicationDisclosed;

/// Budget-aware heap-byte accounting for projected typed rows and fields.
///
/// Every `#[derive(WorthQueryRow)]` struct (both root and nested/child rows)
/// implements this trait by summing `KnownProjectedHeapBytes::known_heap_bytes`
/// across every projected field directly — with NO silent zero-byte fallback.
/// Custom scalar/domain types projected into a derived or handwritten row MUST
/// implement this trait to account for any owned heap allocations.
pub trait KnownProjectedHeapBytes {
    fn known_heap_bytes(&self) -> usize;
}

macro_rules! impl_zero_heap {
    ($($ty:ty),* $(,)?) => {
        $(
            impl KnownProjectedHeapBytes for $ty {
                fn known_heap_bytes(&self) -> usize {
                    0
                }
            }
        )*
    };
}

impl_zero_heap!(
    (),
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    EntityId,
    RelationId,
    KindId,
    PartitionId,
    VersionId,
    CommitId,
    SchemaVersionId,
    ErasedWorthQueryName,
    CanonicalDate,
    CanonicalTime,
    CanonicalTimestamp,
    CanonicalTimestampTz,
    CanonicalF32,
    CanonicalF64,
    ContentRefId,
    WorthQueryRecurringLifecycle,
    WorthQueryRecurringGate,
    WorthQueryPrincipalMappingStatus,
);

impl KnownProjectedHeapBytes for WorthQueryExternalPrincipalIdentity {
    fn known_heap_bytes(&self) -> usize {
        self.issuer().len().saturating_add(self.subject().len())
    }
}

impl<const N: usize> KnownProjectedHeapBytes for [u8; N] {
    fn known_heap_bytes(&self) -> usize {
        0
    }
}

impl<K> KnownProjectedHeapBytes for WorthQueryName<K> {
    fn known_heap_bytes(&self) -> usize {
        0
    }
}

impl KnownProjectedHeapBytes for SchemaId {
    fn known_heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

impl KnownProjectedHeapBytes for String {
    fn known_heap_bytes(&self) -> usize {
        self.capacity()
    }
}

impl KnownProjectedHeapBytes for InternedString {
    fn known_heap_bytes(&self) -> usize {
        self.owned_allocation_capacity_bytes()
    }
}

impl KnownProjectedHeapBytes for InlineBytes {
    fn known_heap_bytes(&self) -> usize {
        self.owned_allocation_capacity_bytes()
    }
}

impl KnownProjectedHeapBytes for CanonicalDecimal {
    fn known_heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

impl KnownProjectedHeapBytes for CanonicalBigInt {
    fn known_heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

impl KnownProjectedHeapBytes for CanonicalRational {
    fn known_heap_bytes(&self) -> usize {
        self.numerator
            .0
            .capacity()
            .saturating_add(self.denominator.0.capacity())
    }
}

impl KnownProjectedHeapBytes for AspectValue {
    fn known_heap_bytes(&self) -> usize {
        self.owned_allocation_capacity_bytes()
    }
}

impl KnownProjectedHeapBytes for Cow<'_, str> {
    fn known_heap_bytes(&self) -> usize {
        match self {
            Cow::Borrowed(_) => 0,
            Cow::Owned(s) => s.capacity(),
        }
    }
}

impl KnownProjectedHeapBytes for Cow<'_, [u8]> {
    fn known_heap_bytes(&self) -> usize {
        match self {
            Cow::Borrowed(_) => 0,
            Cow::Owned(v) => v.capacity(),
        }
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for Option<T> {
    fn known_heap_bytes(&self) -> usize {
        self.as_ref()
            .map_or(0, KnownProjectedHeapBytes::known_heap_bytes)
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for Box<T> {
    fn known_heap_bytes(&self) -> usize {
        std::mem::size_of::<T>().saturating_add((**self).known_heap_bytes())
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for Arc<T> {
    fn known_heap_bytes(&self) -> usize {
        std::mem::size_of::<T>().saturating_add((**self).known_heap_bytes())
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for Vec<T> {
    fn known_heap_bytes(&self) -> usize {
        let elems = self.capacity().saturating_mul(std::mem::size_of::<T>());
        self.iter().fold(elems, |acc, item| {
            acc.saturating_add(item.known_heap_bytes())
        })
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for BTreeSet<T> {
    fn known_heap_bytes(&self) -> usize {
        let elems = self.len().saturating_mul(std::mem::size_of::<T>());
        self.iter().fold(elems, |acc, item| {
            acc.saturating_add(item.known_heap_bytes())
        })
    }
}

impl<K: KnownProjectedHeapBytes, V: KnownProjectedHeapBytes> KnownProjectedHeapBytes
    for BTreeMap<K, V>
{
    fn known_heap_bytes(&self) -> usize {
        let elems = self
            .len()
            .saturating_mul(std::mem::size_of::<K>().saturating_add(std::mem::size_of::<V>()));
        self.iter().fold(elems, |acc, (k, v)| {
            acc.saturating_add(k.known_heap_bytes())
                .saturating_add(v.known_heap_bytes())
        })
    }
}

impl<A: KnownProjectedHeapBytes, B: KnownProjectedHeapBytes> KnownProjectedHeapBytes for (A, B) {
    fn known_heap_bytes(&self) -> usize {
        self.0
            .known_heap_bytes()
            .saturating_add(self.1.known_heap_bytes())
    }
}

impl<A: KnownProjectedHeapBytes, B: KnownProjectedHeapBytes, C: KnownProjectedHeapBytes>
    KnownProjectedHeapBytes for (A, B, C)
{
    fn known_heap_bytes(&self) -> usize {
        self.0
            .known_heap_bytes()
            .saturating_add(self.1.known_heap_bytes())
            .saturating_add(self.2.known_heap_bytes())
    }
}

impl<T: KnownProjectedHeapBytes> KnownProjectedHeapBytes for WorthQueryApplicationDisclosed<T> {
    fn known_heap_bytes(&self) -> usize {
        match self {
            WorthQueryApplicationDisclosed::Disclosed(value) => value.known_heap_bytes(),
            WorthQueryApplicationDisclosed::Omitted(_) => 0,
        }
    }
}
