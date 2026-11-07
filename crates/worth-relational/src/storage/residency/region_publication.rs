//! Assemble immutable root regions without promoting Head contents to Full.
use super::{
    RegionContentFailure, RelationalRegionKindCommitment, RelationalRegionResidency,
    RelationalRegionRetention,
};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::overlay::{
    PartitionContentCommitment, PartitionMutationJournal, PartitionState,
    RelationalPartitionAllocationInventory,
};
use crate::storage::substrate::{RecordArena, RecordKind};
use std::collections::BTreeSet;
use std::sync::Arc;

pub(crate) struct RegionCaptureBasis<'region> {
    pub(crate) retention: &'region RelationalRegionRetention,
    pub(crate) inventory: RelationalPartitionAllocationInventory,
    pub(crate) unavailable_records: usize,
    pub(crate) kind_commitment: RelationalRegionKindCommitment,
}

pub(crate) struct PreparedResidentRegion {
    pub(crate) partition_id: crate::identity::data::PartitionId,
    pub(crate) retention: RelationalRegionRetention,
    pub(crate) inventory: RelationalPartitionAllocationInventory,
    pub(crate) content_digest: [u8; 32],
    pub(crate) kind_commitment: RelationalRegionKindCommitment,
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) content_values_hashed: u64,
    pub(crate) unavailable_records: usize,
    pub(crate) copied_truth_bytes: u64,
    pub(crate) new_payload_bytes: u64,
}

impl PreparedResidentRegion {
    pub(crate) fn incremental(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: RegionCaptureBasis<'_>,
        overlay: &PartitionState,
        journal: &PartitionMutationJournal,
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        symbols: &RuntimeSymbolTable,
        change: Option<&crate::storage::substrate::maintenance::CommitChange>,
    ) -> Result<Self, RegionContentFailure> {
        let absent = || {
            RegionContentFailure::Storage(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(overlay.partition_id),
                cause: RelationalStorageUnavailableCause::SourceUnavailable,
            })
        };
        let previous = basis
            .retention
            .capture_region()
            .map_err(RegionContentFailure::Storage)?
            .ok_or_else(absent)?;
        let prior_commitment = previous.commitment.as_ref().ok_or_else(absent)?;
        let mut partition = previous.partition.clone();
        let copied_truth_bytes = partition.merge_overlay_from_owned(&mut overlay.clone(), journal);
        let prior_kind = previous.kind_indexes.as_ref().ok_or_else(absent)?;
        if prior_kind.commitment() != basis.kind_commitment {
            return Err(RegionContentFailure::Adjacency);
        }
        let kind_indexes = super::kind_admission::update(
            execution,
            &mut partition,
            &previous.partition,
            prior_kind,
            journal,
            change,
        )?;
        let kind_commitment = kind_indexes.commitment();
        partition.clear_runtime_pin_counters();
        let (commitment, content_values_hashed) = PartitionContentCommitment::capture(
            &partition,
            &symbols.interner_snapshot(),
            Some((prior_commitment, &previous.partition, journal)),
        )
        .map_err(RegionContentFailure::Content)?;
        let inventory = partition
            .allocation_inventory_since(&previous.partition, basis.inventory)
            .map_err(RegionContentFailure::Content)?;
        let new_payload_bytes =
            partition.new_authoritative_allocation_bytes_since(&previous.partition);
        let delta = unavailable_delta(
            &partition.entity_arena,
            &previous.partition.entity_arena,
            &journal.entity_slots,
        ) + unavailable_delta(
            &partition.relation_arena,
            &previous.partition.relation_arena,
            &journal.relation_slots,
        );
        let unavailable_records = basis.unavailable_records.checked_add_signed(delta).ok_or(
            RegionContentFailure::Storage(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition.partition_id),
                cause: RelationalStorageUnavailableCause::IntegrityMismatch,
            }),
        )?;
        let entity_slots = partition.entity_arena.slot_count();
        let relation_slots = partition.relation_arena.slot_count();
        let content_digest = commitment.digest(partition.partition_id);
        let history_presence = previous
            .history_presence
            .updated(&previous.partition, &partition);
        let partition_id = partition.partition_id;
        let retention = if !history_presence.is_complete() {
            RelationalRegionRetention::from_head_lease(RelationalRegionResidency::resident_head(
                partition,
                commitment,
                kind_indexes,
                history_presence,
                schema,
                symbols,
            ))
        } else {
            RelationalRegionRetention::from_lease(RelationalRegionResidency::resident(
                partition,
                commitment,
                kind_indexes,
            ))
        };
        Ok(Self {
            partition_id,
            retention,
            inventory,
            content_digest,
            kind_commitment,
            entity_slots,
            relation_slots,
            content_values_hashed,
            unavailable_records,
            copied_truth_bytes,
            new_payload_bytes,
        })
    }
}

fn unavailable_delta<K: RecordKind>(
    current: &RecordArena<K>,
    previous: &RecordArena<K>,
    slots: &BTreeSet<usize>,
) -> isize {
    let unavailable = |arena: &RecordArena<K>, slot| {
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
