mod adjacency;
mod digest_index;
#[cfg(test)]
#[path = "partition_content/tests.rs"]
mod incremental_tests;
mod records;

use super::{PartitionMutationJournal, PartitionState};
use crate::storage::substrate::StorageAllocationVisitor;
use crate::symbols::data::{StringInterner, Symbol};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PartitionContentDigestError {
    UnresolvedContentSymbol(Symbol),
}

/// Derived, destroyable commitment acceleration. Exact storage identities and
/// the sealed publication journal select reuse; digests grant no authority.
#[derive(Debug, Clone, Default)]
pub(crate) struct PartitionContentCommitment {
    entities: records::RecordContentIndex,
    relations: records::RecordContentIndex,
    adjacency: adjacency::AdjacencyContentIndex,
    reverse_adjacency: adjacency::AdjacencyContentIndex,
}

impl PartitionContentCommitment {
    pub(crate) fn capture(
        partition: &PartitionState,
        symbols: &StringInterner,
        previous: Option<(&Self, &PartitionState, &PartitionMutationJournal)>,
    ) -> Result<(Self, u64), PartitionContentDigestError> {
        let mut commitment = previous
            .map(|(cache, _, _)| cache.clone())
            .unwrap_or_default();
        let old = previous.map(|(_, partition, _)| partition);
        let entity_slots = previous.map_or_else(
            || partition.entity_arena.occupied_slots(),
            |(_, _, journal)| journal.entity_slots.iter().copied().collect(),
        );
        let relation_slots = previous.map_or_else(
            || partition.relation_arena.occupied_slots(),
            |(_, _, journal)| journal.relation_slots.iter().copied().collect(),
        );
        let mut hashed = commitment.entities.update(
            &partition.entity_arena,
            old.map(|old| &old.entity_arena),
            entity_slots,
            symbols,
            |extra, symbols| {
                let fingerprint = extra
                    .structural_fingerprint
                    .map(|fingerprint| {
                        symbols
                            .resolve(fingerprint.family)
                            .map(|family| (family, fingerprint.value))
                            .ok_or(PartitionContentDigestError::UnresolvedContentSymbol(
                                fingerprint.family,
                            ))
                    })
                    .transpose()?;
                Ok(hash_value(
                    b"entity-extra",
                    &(
                        fingerprint,
                        &extra.lineage_id,
                        &extra.authoritative_aspect_state,
                    ),
                ))
            },
        )?;
        hashed += commitment.relations.update(
            &partition.relation_arena,
            old.map(|old| &old.relation_arena),
            relation_slots,
            symbols,
            |extra, _| Ok(hash_value(b"relation-extra", extra)),
        )?;
        let adjacency_slots = previous.map_or_else(
            || {
                partition
                    .adjacency
                    .iter()
                    .map(|(&slot, _)| slot)
                    .collect::<Vec<_>>()
            },
            |(_, _, journal)| journal.adjacency_slots.iter().copied().collect(),
        );
        let reverse_slots = previous.map_or_else(
            || {
                partition
                    .reverse_adjacency
                    .iter()
                    .map(|(&slot, _)| slot)
                    .collect::<Vec<_>>()
            },
            |(_, _, journal)| journal.reverse_adjacency_slots.iter().copied().collect(),
        );
        hashed += commitment.adjacency.update(
            &partition.adjacency,
            old.map(|old| &old.adjacency),
            adjacency_slots,
        );
        hashed += commitment.reverse_adjacency.update(
            &partition.reverse_adjacency,
            old.map(|old| &old.reverse_adjacency),
            reverse_slots,
        );
        Ok((commitment, hashed))
    }

    pub(crate) fn digest(&self, partition_id: crate::identity::data::PartitionId) -> [u8; 32] {
        hash_value(
            b"partition-content-v2",
            &(
                partition_id,
                self.entities.digest(),
                self.relations.digest(),
                self.adjacency.digest(),
                self.reverse_adjacency.digest(),
            ),
        )
    }

    pub(crate) fn allocation_bytes(&self) -> u64 {
        self.entities.allocation_bytes()
            + self.relations.allocation_bytes()
            + self.adjacency.allocation_bytes()
            + self.reverse_adjacency.allocation_bytes()
    }
    pub(crate) fn visit_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.entities.visit_allocations(visitor);
        self.relations.visit_allocations(visitor);
        self.adjacency.visit_allocations(visitor);
        self.reverse_adjacency.visit_allocations(visitor);
    }
}

impl PartitionState {
    /// Cold reconstruction from authoritative content, never from carried hashes.
    pub(crate) fn authoritative_content_digest(
        &self,
        symbols: &StringInterner,
    ) -> Result<[u8; 32], PartitionContentDigestError> {
        PartitionContentCommitment::capture(self, symbols, None)
            .map(|(commitment, _)| commitment.digest(self.partition_id))
    }
}

fn hash_value(domain: &[u8], value: &impl Serialize) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.partition-value.v2\0");
    hash.update((domain.len() as u64).to_be_bytes());
    hash.update(domain);
    rmp_serde::encode::write(&mut Sha256Writer(&mut hash), value)
        .expect("authoritative in-memory values are serializable");
    hash.finalize().into()
}

struct Sha256Writer<'a>(&'a mut Sha256);

impl std::io::Write for Sha256Writer<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    use super::hash_value;
    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::identity::data::{KindId, PartitionId, RelationId};
    use crate::storage::overlay::PartitionState;
    use crate::storage::partition::AdjacencySet;
    use crate::storage::substrate::{EntityArena, RelationArena};
    use crate::symbols::data::StringInterner;

    #[test]
    fn streamed_content_value_hash_matches_buffered_canonical_encoding() {
        fn buffered<T: Serialize>(domain: &[u8], value: &T) -> [u8; 32] {
            let mut hash = Sha256::new();
            hash.update(b"worth.relational.partition-value.v2\0");
            hash.update((domain.len() as u64).to_be_bytes());
            hash.update(domain);
            hash.update(rmp_serde::to_vec(value).unwrap());
            hash.finalize().into()
        }

        let records = vec![
            (Some(1_u64), vec!["alpha".to_owned(), "beta".to_owned()]),
            (None, vec!["unicode-λ".to_owned()]),
        ];
        let revisions = std::collections::BTreeMap::from([
            ("field-a", (3_u64, false)),
            ("field-b", (8_u64, true)),
        ]);
        assert_eq!(
            hash_value(b"record", &records),
            buffered(b"record", &records)
        );
        assert_eq!(
            hash_value(b"metadata", &revisions),
            buffered(b"metadata", &revisions)
        );
        assert_eq!(hash_value(b"", &0_u64), buffered(b"", &0_u64));
        assert_ne!(
            hash_value(b"record", &records),
            hash_value(b"extra", &records)
        );
    }

    #[test]
    fn derived_adjacency_caches_cannot_change_truth_digest() {
        let policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 0,
        };
        let mut partition = PartitionState {
            partition_id: PartitionId(5),
            adjacency_policy: policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(0),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: vec![AdjacencySet::new(&policy)].into(),
            reverse_adjacency: Default::default(),
        };
        let symbols = StringInterner::default();
        let baseline_digest = partition.authoritative_content_digest(&symbols).unwrap();
        let baseline_inventory = partition.allocation_inventory();

        partition.adjacency[0]
            .index_historical_kind(KindId(9), RelationId::new(PartitionId(5), 4, 1));
        let perturbed_inventory = partition.allocation_inventory();

        assert_eq!(
            partition.authoritative_content_digest(&symbols).unwrap(),
            baseline_digest
        );
        assert_eq!(
            perturbed_inventory.authoritative_bytes,
            baseline_inventory.authoritative_bytes
        );
        assert_eq!(perturbed_inventory.allocator_bookkeeping_bytes, 0);
        assert!(perturbed_inventory.optional_cache_bytes > baseline_inventory.optional_cache_bytes);
    }
}
