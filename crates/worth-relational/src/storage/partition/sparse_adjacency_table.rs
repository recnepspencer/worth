use crate::storage::substrate::SharedMap;
use std::ops::{Index, IndexMut};

use crate::config::data::AdjacencyPolicy;

use super::AdjacencySet;

#[derive(Debug, Clone, Default)]
pub(crate) struct SparseAdjacencyTable {
    entries: SharedMap<usize, AdjacencySet>,
}

impl SparseAdjacencyTable {
    pub(crate) fn merge_slots_from(
        &mut self,
        overlay: &Self,
        slots: &std::collections::BTreeSet<usize>,
    ) -> u64 {
        slots.iter().fold(0_u64, |bytes, &slot| {
            bytes.saturating_add(self.entries.copy_value_from(slot, &overlay.entries))
        })
    }

    pub(crate) fn visit_new_cache_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        // Outer map owners belong to truth accounting; only changed values'
        // separately owned optional indexes enter this lane.
        self.entries.visit_new_allocations(
            &previous.entries,
            &mut WalkOnly,
            &mut |set, previous, _, _| {
                if let Some(previous) = previous {
                    set.visit_new_cache_allocations(previous, visitor);
                } else {
                    set.visit_cache_allocations(visitor);
                }
            },
        );
    }

    pub(crate) fn visit_new_authoritative_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.entries.visit_new_allocations(
            &previous.entries,
            visitor,
            &mut |set, previous, allocation, visitor| {
                if visitor.visit(allocation) {
                    if let Some(previous) = previous {
                        set.visit_new_authoritative_allocations(previous, visitor);
                    } else {
                        set.visit_authoritative_allocations(false, visitor);
                    }
                }
            },
        );
    }
    pub(crate) fn get(&self, slot: usize) -> Option<&AdjacencySet> {
        self.entries.get(&slot)
    }

    pub(crate) fn get_mut(&mut self, slot: usize) -> Option<&mut AdjacencySet> {
        self.entries.get_mut(&slot)
    }

    pub(crate) fn ensure(&mut self, slot: usize, policy: &AdjacencyPolicy) -> &mut AdjacencySet {
        self.entries
            .entry(slot)
            .or_insert_with(|| AdjacencySet::new(policy))
    }

    pub(crate) fn clear_slot(&mut self, slot: usize, policy: &AdjacencyPolicy) {
        self.entries.insert(slot, AdjacencySet::new(policy));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&usize, &AdjacencySet)> {
        self.entries.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (&usize, &mut AdjacencySet)> {
        self.entries.iter_mut()
    }

    pub(crate) fn into_entries(self) -> impl Iterator<Item = (usize, AdjacencySet)> {
        self.entries.into_iter()
    }

    pub(crate) fn from_entries(entries: impl IntoIterator<Item = (usize, AdjacencySet)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub(crate) fn allocation_bytes(&self) -> u64 {
        self.entries.allocation_bytes()
    }

    pub(crate) fn visit_authoritative_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.entries
            .visit_allocations(unique, visitor, &mut |set, allocation, visitor| {
                if visitor.visit(allocation) {
                    set.visit_authoritative_allocations(allocation.unique, visitor);
                }
            });
    }

    pub(crate) fn visit_cache_allocations(
        &self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        for (_, set) in self.iter() {
            set.visit_cache_allocations(visitor);
        }
    }
}

struct WalkOnly;
impl crate::storage::substrate::StorageAllocationVisitor for WalkOnly {
    fn visit(&mut self, _: crate::storage::substrate::StorageAllocationObservation) -> bool {
        true
    }
}

impl From<Vec<AdjacencySet>> for SparseAdjacencyTable {
    fn from(entries: Vec<AdjacencySet>) -> Self {
        Self {
            entries: entries.into_iter().enumerate().collect(),
        }
    }
}

impl Index<usize> for SparseAdjacencyTable {
    type Output = AdjacencySet;

    fn index(&self, slot: usize) -> &Self::Output {
        &self.entries[&slot]
    }
}

impl IndexMut<usize> for SparseAdjacencyTable {
    fn index_mut(&mut self, slot: usize) -> &mut Self::Output {
        self.entries
            .get_mut(&slot)
            .expect("adjacency slot must be materialized")
    }
}
