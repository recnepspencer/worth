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
use std::sync::Arc;

pub(crate) struct RegionCaptureBasis<'region> {
    pub(crate) retention: &'region RelationalRegionRetention,
    pub(crate) directory: &'region Arc<super::RegionRecordDirectory>,
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
    pub(crate) directory: Arc<super::RegionRecordDirectory>,
    pub(crate) live_counts: crate::storage::residency::LiveRecordCounts,
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
        // Head custody can borrow a physically complete image already loaded
        // by another observer. Acquire its real Full witness before carrying
        // Full dictionary coverage; this branch never requests omitted history.
        if previous.history_presence.is_complete()
            && basis.retention.admitted_full_lease().is_none()
        {
            let full = basis
                .retention
                .demand_full(execution, 1 << 30)
                .map_err(RegionContentFailure::from)?;
            full.admit_symbols(execution, symbols)
                .map_err(RegionContentFailure::from)?;
        }
        basis
            .retention
            .admit_own_symbols(execution, symbols)
            .map_err(RegionContentFailure::from)?;
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
        let directory = super::RegionRecordDirectory::updated(
            execution,
            basis.directory,
            &previous.partition,
            &partition,
            journal,
            change,
        )
        .map_err(|error| error.into_content_failure(partition.partition_id))?;
        let symbol_changes =
            super::RegionSymbolChanges::derive(execution, &previous.partition, &partition)?;
        symbols
            .demand_symbol_ids(execution, symbol_changes.owned.ids.iter().copied())
            .map_err(RegionContentFailure::from)?;
        partition.clear_runtime_pin_counters();
        let (commitment, content_values_hashed) = PartitionContentCommitment::maintain(
            &partition,
            &symbols.interner_snapshot(),
            (prior_commitment, &previous.partition, journal),
        )
        .map_err(RegionContentFailure::Content)?;
        let inventory = partition
            .allocation_inventory_since(&previous.partition, basis.inventory)
            .map_err(RegionContentFailure::Content)?;
        let new_payload_bytes = partition
            .new_authoritative_allocation_bytes_since(&previous.partition)
            .map_err(RegionContentFailure::Content)?;
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
        let live_counts = super::LiveRecordCounts::from_partition(&partition);
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
                None,
            ))
        };
        retention.inherit_symbols(basis.retention, symbol_changes);
        retention
            .admit_own_symbols(execution, symbols)
            .map_err(RegionContentFailure::from)?;
        Ok(Self {
            partition_id,
            retention,
            inventory,
            content_digest,
            kind_commitment,
            directory,
            live_counts,
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
    slots: &crate::storage::overlay::TouchedSlots,
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

#[cfg(test)]
#[path = "region_publication/full/tests.rs"]
mod full_tests;
