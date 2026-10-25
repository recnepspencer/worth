use crate::storage::residency::{RelationalRegionResidency, RelationalRegionRetention};

use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::root::RelationalBranchRootCaptureDenial;

#[derive(Debug)]
pub(crate) struct RelationalRootRegion {
    pub(super) creation_root_id: u64,
    pub(super) id: u64,
    pub(super) partition_id: PartitionId,
    pub(super) partition: RelationalRegionRetention,
    allocation_inventory: Option<crate::storage::overlay::RelationalPartitionAllocationInventory>,
    pub(super) content_digest: [u8; 32],
    pub(super) entity_slots: usize,
    pub(super) relation_slots: usize,
    pub(super) content_values_hashed: u64,
    materialization_unavailable_records: usize,
}

impl RelationalRootRegion {
    pub(super) fn new(
        creation_root_id: u64,
        id: u64,
        mut partition: PartitionState,
        symbols: &crate::symbols::data::StringInterner,
        previous: Option<(&Self, &crate::storage::overlay::PartitionMutationJournal)>,
    ) -> Result<Self, RelationalBranchRootCaptureDenial> {
        partition.clear_runtime_pin_counters();
        let (content_commitment, content_values_hashed) =
            crate::storage::overlay::PartitionContentCommitment::capture(
                &partition,
                symbols,
                previous.map(|(region, journal)| {
                    (
                        region.partition.full_view().commitment(),
                        region.partition.as_ref(),
                        journal,
                    )
                }),
            )
            .map_err(|error| match error {
                crate::storage::overlay::PartitionContentDigestError::UnresolvedContentSymbol(
                    symbol,
                ) => RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
            })?;
        let content_digest = content_commitment.digest(partition.partition_id);
        let (allocation_inventory, materialization_unavailable_records) =
            if let Some((previous, journal)) = previous {
                let inventory = partition.allocation_inventory_since(
                    &previous.partition,
                    previous.allocation_inventory(),
                );
                let counts = unavailable_delta(
                    &partition.entity_arena,
                    &previous.partition.entity_arena,
                    &journal.entity_slots,
                ) + unavailable_delta(
                    &partition.relation_arena,
                    &previous.partition.relation_arena,
                    &journal.relation_slots,
                );
                let unavailable = previous
                    .materialization_unavailable_records
                    .checked_add_signed(counts)
                    .expect("journal lifecycle delta must fit its previous region");
                (inventory, unavailable)
            } else {
                (
                    partition.allocation_inventory(),
                    partition.entity_arena.lifecycle_counts().unavailable
                        + partition.relation_arena.lifecycle_counts().unavailable,
                )
            };
        Ok(Self {
            creation_root_id,
            id,
            partition_id: partition.partition_id,
            allocation_inventory: Some(allocation_inventory),
            content_digest,
            entity_slots: partition.entity_arena.slot_count(),
            relation_slots: partition.relation_arena.slot_count(),
            partition: RelationalRegionRetention::from_lease(RelationalRegionResidency::resident(
                partition,
                content_commitment,
            )),
            content_values_hashed,
            materialization_unavailable_records,
        })
    }

    pub(super) fn from_checkpoint(
        creation_root_id: u64,
        id: u64,
        checkpoint: super::root_checkpoint::RelationalRootRegionCheckpoint,
    ) -> Self {
        Self {
            creation_root_id,
            id,
            partition_id: checkpoint.partition_id,
            partition: RelationalRegionRetention::new(checkpoint.cell),
            allocation_inventory: None,
            content_digest: checkpoint.content_digest,
            entity_slots: checkpoint.entity_slots,
            relation_slots: checkpoint.relation_slots,
            content_values_hashed: 0,
            materialization_unavailable_records: checkpoint.unavailable_records,
        }
    }

    pub(super) fn allocation_inventory(
        &self,
    ) -> crate::storage::overlay::RelationalPartitionAllocationInventory {
        self.allocation_inventory
            .unwrap_or_else(|| self.partition.cell().resident_inventory())
    }

    pub(super) fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }

    pub(super) fn observation(&self) -> RelationalRootRegionObservation {
        let inventory = self.allocation_inventory();
        RelationalRootRegionObservation {
            creation_root_id: self.creation_root_id,
            region_id: self.id,
            partition_id: self.partition_id,
            root_region_bytes: std::mem::size_of::<Self>() as u64,
            partition_state_bytes: self.partition.cell().residency_allocation_bytes(),
            authoritative_bytes: inventory.authoritative_bytes,
            diagnostic_bytes: inventory.diagnostic_bytes,
            retention_metadata_bytes: inventory.retention_metadata_bytes,
            allocator_bookkeeping_bytes: inventory.allocator_bookkeeping_bytes,
            optional_cache_bytes: inventory.optional_cache_bytes
                + self.partition.cell().resident_commitment_bytes(),
        }
    }

    pub(super) fn reclaimable_unique_authoritative_bytes(&self) -> u64 {
        let mut bytes = std::mem::size_of::<Self>() as u64;
        if self.partition.is_uniquely_owned() {
            bytes = bytes
                .saturating_add(self.partition.residency_allocation_bytes())
                .saturating_add(self.partition.reclaimable_unique_authoritative_bytes());
        }
        bytes
    }
}

fn unavailable_delta<K: crate::storage::substrate::RecordKind>(
    current: &crate::storage::substrate::RecordArena<K>,
    previous: &crate::storage::substrate::RecordArena<K>,
    slots: &std::collections::BTreeSet<usize>,
) -> isize {
    let unavailable = |arena: &crate::storage::substrate::RecordArena<K>, slot| {
        arena.physical_index(slot).is_some_and(|physical| {
            arena.lifecycle[physical]
                == crate::storage::data::RecordLifecycleState::MaterializationUnavailable
        }) as isize
    };
    slots
        .iter()
        .map(|&slot| unavailable(current, slot) - unavailable(previous, slot))
        .sum()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalRootRegionObservation {
    pub(crate) creation_root_id: u64,
    pub(crate) region_id: u64,
    pub(crate) partition_id: PartitionId,
    pub(crate) root_region_bytes: u64,
    pub(crate) partition_state_bytes: u64,
    pub(crate) authoritative_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) retention_metadata_bytes: u64,
    pub(crate) allocator_bookkeeping_bytes: u64,
    pub(crate) optional_cache_bytes: u64,
}
