use super::{digest_index::DigestIndex, hash_value};
use crate::storage::partition::SparseAdjacencyTable;
use crate::storage::substrate::{SharedMap, StorageAllocationVisitor};

#[derive(Debug, Clone, Default)]
pub(super) struct AdjacencyContentIndex {
    slots: DigestIndex,
    memberships: SharedMap<usize, DigestIndex>,
    membership_bytes: u64,
}

impl AdjacencyContentIndex {
    pub(super) fn update(
        &mut self,
        current: &SparseAdjacencyTable,
        previous: Option<&SparseAdjacencyTable>,
        slots: impl IntoIterator<Item = usize>,
    ) -> u64 {
        let mut hashed = 0;
        let mut cold_slots = previous.is_none().then(Vec::new);
        let mut cold_memberships = previous.is_none().then(Vec::new);
        for slot in slots {
            let old = previous.and_then(|previous| previous.get(slot));
            let mut membership = self.memberships.get(&slot).cloned().unwrap_or_default();
            let old_bytes = membership.allocation_bytes();
            if let Some(current) = current.get(slot) {
                for (id, present) in current.changed_current_memberships(old) {
                    let key = ((id.partition_id.as_u32() as u128) << 96)
                        | ((id.slot_index() as u128) << 32)
                        | id.generation_value() as u128;
                    membership.set(
                        key,
                        present.then(|| hash_value(b"relation-membership", &id)),
                    );
                    hashed += u64::from(present);
                }
                if let Some(entries) = &mut cold_slots {
                    entries.push((slot as u128, membership.digest()));
                } else {
                    self.slots.set(slot as u128, Some(membership.digest()));
                }
                self.membership_bytes = self
                    .membership_bytes
                    .checked_sub(old_bytes)
                    .unwrap()
                    .checked_add(membership.allocation_bytes())
                    .unwrap();
                if let Some(entries) = &mut cold_memberships {
                    entries.push((slot, membership));
                } else {
                    self.memberships.insert(slot, membership);
                }
            } else {
                if cold_slots.is_none() {
                    self.slots.set(slot as u128, None);
                    self.memberships.remove(&slot);
                }
                self.membership_bytes = self.membership_bytes.checked_sub(old_bytes).unwrap();
            }
        }
        if let Some(mut entries) = cold_slots {
            entries.sort_unstable_by_key(|(slot, _)| *slot);
            self.slots = DigestIndex::from_sorted_entries(&entries);
        }
        if let Some(mut entries) = cold_memberships {
            entries.sort_unstable_by_key(|(slot, _)| *slot);
            self.memberships = SharedMap::from_sorted_unique(entries);
        }
        hashed
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        self.slots.digest()
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        self.slots.allocation_bytes() + self.memberships.allocation_bytes() + self.membership_bytes
    }
    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.slots.visit_allocations(visitor);
        self.memberships
            .visit_allocations(false, visitor, &mut |set, allocation, visitor| {
                if visitor.visit(allocation) {
                    set.visit_allocations(visitor);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::identity::data::{KindId, PartitionId, RelationId};

    #[test]
    fn cold_adjacency_bulk_commitment_matches_incremental_construction() {
        let policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let mut table = SparseAdjacencyTable::default();
        for slot in [0, 1, 64, 127] {
            table.ensure(slot, &policy);
        }
        table
            .get_mut(1)
            .unwrap()
            .insert(KindId(7), RelationId::new(PartitionId::main(), 3, 1));
        table
            .get_mut(64)
            .unwrap()
            .insert(KindId(8), RelationId::new(PartitionId::main(), 5, 2));
        let slots = [0, 1, 64, 127];
        let mut cold = AdjacencyContentIndex::default();
        let mut incremental = AdjacencyContentIndex::default();
        assert_eq!(cold.update(&table, None, slots), 2);
        assert_eq!(
            incremental.update(&table, Some(&SparseAdjacencyTable::default()), slots),
            2
        );
        assert_eq!(cold.digest(), incremental.digest());
        assert_eq!(cold.allocation_bytes(), incremental.allocation_bytes());
    }
}
