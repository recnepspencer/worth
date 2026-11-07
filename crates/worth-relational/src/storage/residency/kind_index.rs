//! Destroyable canonical relation-kind-incidence commitment acceleration.
use super::kind_projection::ProjectionCheckpoint;
use crate::identity::data::{KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::RegionRelationProjection;
use crate::storage::substrate::{DigestIndex, SharedMap, StorageAllocationVisitor};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Default)]
pub(super) struct KindIndex {
    relations: SharedMap<RelationId, KindIncidence>,
    digest: DigestIndex,
    incidence_bytes: u64,
}

#[derive(Debug, Clone)]
struct KindIncidence {
    kind: KindId,
    outgoing: DigestIndex,
    incoming: DigestIndex,
}

impl KindIndex {
    /// Cold reconstruction uses sorted bulk construction, independently of the
    /// ordinary membership update. Ordering and tree shape depend only on keys.
    pub(super) fn rebuild(
        projection: &[RegionRelationProjection],
        checkpoint: &mut ProjectionCheckpoint<'_>,
    ) -> Result<Self, Cause> {
        let mut relations = Vec::new();
        let mut digests = Vec::new();
        let mut previous = None;
        let mut incidence_bytes = 0_u64;
        for entry in projection {
            checkpoint(1, 4096)?;
            if previous.is_some_and(|id| id >= entry.relation)
                || entry.outgoing.is_empty() && entry.incoming.is_empty()
            {
                return Err(Cause::IntegrityMismatch);
            }
            previous = Some(entry.relation);
            let mut slots = |values: &std::collections::BTreeSet<usize>| {
                let mut leaves = Vec::new();
                for &slot in values {
                    checkpoint(1, 512)?;
                    leaves.push((slot as u128, membership_digest(slot)));
                }
                Ok::<_, Cause>(DigestIndex::from_sorted_entries(&leaves))
            };
            let value = KindIncidence {
                kind: entry.kind,
                outgoing: slots(&entry.outgoing)?,
                incoming: slots(&entry.incoming)?,
            };
            incidence_bytes += value.allocation_bytes();
            digests.push((relation_key(entry.relation), value.digest()));
            relations.push((entry.relation, value));
        }
        // RelationId's ordering and its packed native key have the same order.
        Ok(Self {
            relations: SharedMap::from_sorted_unique(relations),
            digest: DigestIndex::from_sorted_entries(&digests),
            incidence_bytes,
        })
    }

    /// O(log relations + log incidence) persistent paths for one changed
    /// membership. No unrelated relation, slot or historical version is read.
    pub(super) fn update(
        &mut self,
        relation: RelationId,
        kind: KindId,
        incoming: bool,
        slot: usize,
        present: bool,
        checkpoint: &mut ProjectionCheckpoint<'_>,
    ) -> Result<(), Cause> {
        checkpoint(self.relations.navigation_work_bound() as u64, 0)?;
        let mut incidence = self
            .relations
            .get(&relation)
            .cloned()
            .unwrap_or(KindIncidence {
                kind,
                outgoing: DigestIndex::default(),
                incoming: DigestIndex::default(),
            });
        if incidence.kind != kind {
            return Err(Cause::IntegrityMismatch);
        }
        let old_bytes = incidence.allocation_bytes();
        let selected = if incoming {
            &mut incidence.incoming
        } else {
            &mut incidence.outgoing
        };
        if selected
            .contains_key(slot as u128)
            .map_err(|_| Cause::IntegrityMismatch)?
            == present
        {
            return Ok(());
        }
        let membership_quote = selected.edit_quote(slot as u128);
        let relation_quote = self.digest.edit_quote(relation_key(relation));
        let map_bytes = self
            .relations
            .adoption_allocation_bound()
            .ok_or(Cause::MemoryExhausted)?;
        checkpoint(
            membership_quote.0 + relation_quote.0,
            membership_quote
                .1
                .checked_add(relation_quote.1)
                .and_then(|bytes| bytes.checked_add(map_bytes))
                .ok_or(Cause::MemoryExhausted)?,
        )?;
        selected
            .set(slot as u128, present.then(|| membership_digest(slot)))
            .map_err(|_| Cause::IntegrityMismatch)?;
        let empty = incidence.outgoing.is_empty() && incidence.incoming.is_empty();
        self.incidence_bytes = self
            .incidence_bytes
            .checked_sub(old_bytes)
            .and_then(|bytes| {
                bytes.checked_add(if empty {
                    0
                } else {
                    incidence.allocation_bytes()
                })
            })
            .ok_or(Cause::MemoryExhausted)?;
        self.digest
            .set(relation_key(relation), (!empty).then(|| incidence.digest()))
            .map_err(|_| Cause::IntegrityMismatch)?;
        if empty {
            self.relations.remove(&relation);
        } else {
            self.relations.insert(relation, incidence);
        }
        Ok(())
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"worth.relational.region-kind-authority.v3\0");
        hash.update(self.digest.digest());
        hash.finalize().into()
    }

    pub(super) fn allocation_bytes(&self) -> u64 {
        self.relations.allocation_bytes() + self.digest.allocation_bytes() + self.incidence_bytes
    }

    pub(super) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.digest.visit_allocations(visitor);
        self.relations
            .visit_allocations(false, visitor, &mut |incidence, allocation, visitor| {
                if visitor.visit(allocation) {
                    incidence.outgoing.visit_allocations(visitor);
                    incidence.incoming.visit_allocations(visitor);
                }
            });
    }
}

impl KindIncidence {
    fn allocation_bytes(&self) -> u64 {
        self.outgoing.allocation_bytes() + self.incoming.allocation_bytes()
    }
    fn digest(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"worth.relational.region-kind-incidence.v1\0");
        hash.update(self.kind.0.to_be_bytes());
        hash.update(self.outgoing.digest());
        hash.update(self.incoming.digest());
        hash.finalize().into()
    }
}

fn membership_digest(slot: usize) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-slot.v1\0");
    hash.update((slot as u64).to_be_bytes());
    hash.finalize().into()
}

fn relation_key(id: RelationId) -> u128 {
    ((id.partition_id.as_u32() as u128) << 96)
        | ((id.slot_index() as u128) << 32)
        | id.generation_value() as u128
}
