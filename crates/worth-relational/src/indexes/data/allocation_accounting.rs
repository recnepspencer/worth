use super::{DerivedIndexEntryMap, DerivedIndexRows};

use crate::identity::data::EntityId;
use crate::storage::data::AuthoritativeFieldComparisonKey;

use super::{
    DerivedIndexArtifacts, DerivedIndexEntries, DerivedIndexGeneration, RelatedEntityOrderingEntry,
    RelationJoinEntry, RelationJoinKey,
};

impl DerivedIndexArtifacts {
    pub(super) fn recursive_owned_allocation_capacity_bytes(&self) -> u64 {
        vector_capacity_bytes::<DerivedIndexGeneration>(&self.generations).saturating_add(
            self.generations
                .iter()
                .map(DerivedIndexGeneration::owned_allocation_capacity_bytes)
                .sum::<u64>(),
        )
    }
}

impl DerivedIndexGeneration {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.source_branch_id.0.capacity() as u64)
            .saturating_add(self.applicability.branch_id.0.capacity() as u64)
            .saturating_add(self.entries.owned_allocation_capacity_bytes())
    }
}

impl DerivedIndexEntries {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::EntityField(entries) => comparison_entries_bytes(entries),
            Self::RelationField(entries) => comparison_entries_bytes(entries),
            Self::RelatedEntityOrdering(entries) => ordering_entries_bytes(entries),
            Self::RelationJoin(entries) => fixed_entries_bytes(entries),
        }
    }
}

fn comparison_entries_bytes<RecordId: Clone>(
    entries: &DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, RecordId>,
) -> u64 {
    map_payload_bytes(entries)
        .saturating_add(
            entries
                .keys()
                .map(AuthoritativeFieldComparisonKey::owned_allocation_capacity_bytes)
                .sum(),
        )
        .saturating_add(entries.values().map(row_payload_bytes::<RecordId>).sum())
}

fn ordering_entries_bytes(
    entries: &DerivedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>,
) -> u64 {
    map_payload_bytes(entries).saturating_add(
        entries
            .values()
            .map(|values| {
                row_payload_bytes::<RelatedEntityOrderingEntry>(values).saturating_add(
                    values
                        .iter()
                        .map(RelatedEntityOrderingEntry::owned_allocation_capacity_bytes)
                        .sum(),
                )
            })
            .sum(),
    )
}

fn fixed_entries_bytes(entries: &DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>) -> u64 {
    map_payload_bytes(entries).saturating_add(
        entries
            .values()
            .map(row_payload_bytes::<RelationJoinEntry>)
            .sum(),
    )
}

fn map_payload_bytes<Key: Ord + Clone, Value: Clone>(
    entries: &DerivedIndexEntryMap<Key, Value>,
) -> u64 {
    // Charge reachable payload for every generation, including shared Arc key
    // headers and map handles. This is an estimate, not unique physical bytes:
    // im tree-node capacity and allocator overhead are not exposed here.
    let per_key = std::mem::size_of::<(std::sync::Arc<Key>, DerivedIndexRows<Value>)>()
        .saturating_add(std::mem::size_of::<Key>())
        .saturating_add(2 * std::mem::size_of::<usize>());
    (entries.len() as u64).saturating_mul(per_key as u64)
}

fn vector_capacity_bytes<Value>(values: &Vec<Value>) -> u64 {
    (values.capacity() as u64).saturating_mul(std::mem::size_of::<Value>() as u64)
}

fn row_payload_bytes<Value>(values: &DerivedIndexRows<Value>) -> u64 {
    // Each row has an Arc allocation plus a vector handle; im node capacity is
    // deliberately excluded from this logical reachable-payload estimate.
    let per_row = std::mem::size_of::<Value>()
        .saturating_add(std::mem::size_of::<std::sync::Arc<Value>>())
        .saturating_add(2 * std::mem::size_of::<usize>());
    (values.len() as u64).saturating_mul(per_row as u64)
}

impl DerivedIndexArtifacts {
    /// Actual allocations made by Clone; persistent map/vector nodes and row
    /// payloads are shared by pinned im 15.1 rather than newly allocated.
    #[cfg(test)]
    pub(crate) fn clone_header_storage_bytes(&self) -> Option<u64> {
        let roster = std::alloc::Layout::array::<DerivedIndexGeneration>(self.generations.len())
            .ok()?
            .size();
        self.generations
            .iter()
            .try_fold(u64::try_from(roster).ok()?, |total, generation| {
                total
                    .checked_add(u64::try_from(generation.source_branch_id.0.len()).ok()?)?
                    .checked_add(u64::try_from(generation.applicability.branch_id.0.len()).ok()?)
            })
    }
}
