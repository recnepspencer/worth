//! Physical owner traversal. Visitors may prune already-seen or externally retained owners.
#[cfg(test)]
mod tests;

use worth_foundational::facade::{
    AuthoritativeRecordAspectState, FoundationalAllocationKind, FoundationalAllocationObservation,
    FoundationalAllocationVisitor,
};

/// Typed overflow when an allocation inventory, fork-materialized total, or
/// reclamation byte total cannot be represented in `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageAllocationOverflow;

impl std::fmt::Display for StorageAllocationOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("storage allocation accounting total exceeds u64")
    }
}

impl std::error::Error for StorageAllocationOverflow {}

/// Whether an observation's `bytes` is a representable checked extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageAllocationExtent {
    /// The owner's exact layout plus any checked owned extent.
    Checked,
    /// A bound or layout sum did not fit `u64`. Every inventory, publication
    /// delta, inspection, fork-materialized total, and reclamation counter
    /// refuses this typed instead of saturating.
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageAllocationObservation {
    pub id: u64,
    pub bytes: u64,
    pub unique: bool,
    pub strong_count: usize,
    pub birth_seq: Option<u64>,
    pub extent: StorageAllocationExtent,
}

impl StorageAllocationObservation {
    /// Adds a checked nested extent to this owner's observation. `None` or
    /// `u64` overflow marks the observation as `Overflow` so callers refuse it.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn add_checked_nested_bound(&mut self, bound: Option<u64>) {
        match bound.and_then(|bound| self.bytes.checked_add(bound)) {
            Some(bytes) => self.bytes = bytes,
            None => {
                self.bytes = u64::MAX;
                self.extent = StorageAllocationExtent::Overflow;
            }
        }
    }
}

pub trait StorageAllocationVisitor {
    /// Return true to visit allocations owned by this allocation as well.
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool;
}

/// Bridges `worth-foundational`'s physical allocation visitor into `worth-relational`,
/// admitting `ValueBacking` allocations and skipping `SchemaContract` allocations
/// (which are already owned and accounted by `RelationalBranchRootSchemaAuthority`).
struct FoundationalValueBackingAdapter<'a> {
    visitor: &'a mut dyn StorageAllocationVisitor,
}

impl FoundationalAllocationVisitor for FoundationalValueBackingAdapter<'_> {
    fn visit(&mut self, allocation: FoundationalAllocationObservation) -> bool {
        if allocation.kind != FoundationalAllocationKind::ValueBacking {
            return false;
        }
        self.visitor.visit(StorageAllocationObservation {
            id: allocation.id,
            bytes: allocation.bytes,
            unique: allocation.unique,
            strong_count: allocation.strong_count,
            birth_seq: allocation.birth_seq,
            extent: StorageAllocationExtent::Checked,
        })
    }
}

pub fn visit_foundational_value_backing(
    state: &AuthoritativeRecordAspectState,
    ancestor_unique: bool,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    let mut adapter = FoundationalValueBackingAdapter { visitor };
    state.visit_allocations(ancestor_unique, &mut adapter);
}

pub fn visit_new_foundational_value_backing(
    state: &AuthoritativeRecordAspectState,
    previous: Option<&AuthoritativeRecordAspectState>,
    ancestor_unique: bool,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    let mut adapter = FoundationalValueBackingAdapter { visitor };
    state.visit_new_allocations(previous, ancestor_unique, &mut adapter);
}

pub fn visit_inline_value<T>(
    _value: &T,
    allocation: super::StoredValueObservation,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    allocation.visit(visitor);
}

pub fn visit_new_inline_value<T>(
    value: &T,
    _previous: Option<&T>,
    allocation: super::StoredValueObservation,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    visit_inline_value(value, allocation, visitor);
}

impl<T> super::StorageAllocation<T> {
    pub fn observation(&self, ancestor_unique: bool) -> StorageAllocationObservation {
        let strong_count = self.strong_count();
        StorageAllocationObservation {
            id: self.id(),
            bytes: Self::layout_bytes(),
            unique: ancestor_unique && strong_count == 1,
            strong_count,
            birth_seq: None,
            extent: StorageAllocationExtent::Checked,
        }
    }
}
