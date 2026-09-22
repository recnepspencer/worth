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
                self.slots.set(slot as u128, Some(membership.digest()));
                self.membership_bytes = self
                    .membership_bytes
                    .checked_sub(old_bytes)
                    .unwrap()
                    .checked_add(membership.allocation_bytes())
                    .unwrap();
                self.memberships.insert(slot, membership);
            } else {
                self.slots.set(slot as u128, None);
                self.memberships.remove(&slot);
                self.membership_bytes = self.membership_bytes.checked_sub(old_bytes).unwrap();
            }
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
