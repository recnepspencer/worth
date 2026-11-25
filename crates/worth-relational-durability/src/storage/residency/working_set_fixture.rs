//! Real cold regions bound to one home's working set, with memory references.
use super::CountedSource;
use crate::durability::native_region_fixture::{populated_partition, HistoryPartition};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root::capture::reconstruction_fixture::PartitionContentOracle as _;
use worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInterner;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::schema_carrier::RecordedBranchSchemaImage;
use crate::storage::residency::region_load::CHECKPOINT_REGION_READMISSION;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::overlay::partition_content::PartitionContentCommitment;
use worth_relational_storage::internal_port::storage::residency::cell::{
    RelationalColdRegion, RelationalRegionResidency,
};
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure;
use worth_relational_storage::internal_port::storage::residency::lease::{Full, RelationalResidencyLease};
use worth_relational_storage::internal_port::storage::residency::live_counts::LiveRecordCounts;
use worth_relational_storage::internal_port::storage::residency::retention::RelationalRegionRetention;
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyBudget, ResidencyPoolKind, WorkingSetAccountant,
};
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;

pub(super) const LIMIT: u64 = 1 << 24;

/// One durable basis: its cold leaf and its independently held memory digest.
#[derive(Clone)]
pub(super) struct Basis {
    pub(super) cold: RelationalColdRegion,
    pub(super) source: Arc<CountedSource>,
    pub(super) reference_digest: [u8; 32],
}

pub(super) struct Bases {
    pub(super) entity: EntityId,
    pub(super) original: Basis,
    pub(super) edited: Basis,
}

/// Encode `partition` through the production checkpoint codec into a cold leaf
/// over a counted archive; the reference digest is computed from memory.
fn encode(
    partition: &PartitionState,
    schema: &Arc<RelationalBranchRootSchemaAuthority>,
    symbols: &StringInterner,
) -> Basis {
    with_reconstruction_fixture_for_test(|lane| {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let (commitment, _) =
            PartitionContentCommitment::capture(lane, execution, partition, symbols).unwrap();
        let schema_digest = RecordedBranchSchemaImage::capture(schema.schema_image_source())
            .unwrap()
            .carrier_digest();
        let codec = &CHECKPOINT_REGION_READMISSION;
        let members = (codec.capture_members)(
            lane,
            execution,
            partition,
            &commitment,
            schema.as_ref(),
            schema_digest,
        )
        .unwrap();
        let source = Arc::new(CountedSource {
            history: members.history.0,
            members: BTreeMap::from([members.head.clone(), members.history.clone()]),
            reads: AtomicUsize::new(0),
            history_reads: AtomicUsize::new(0),
            corrupt: AtomicBool::new(false),
        });
        let authority = RuntimeSymbolTable::default();
        authority.replace(symbols.clone());
        let cold = RelationalColdRegion {
            readmission: codec,
            directory: members.directory.portable(lane, execution).unwrap(),
            history_members: members.history_members,
            kind_authority: members.kind_authority,
            source: Some(source.clone()),
            partition_id: partition.partition_id,
            head: members.head.0,
            history: members.history.0,
            schema: schema.clone(),
            schema_carrier_digest: schema_digest,
            symbols: authority,
            content_digest: commitment.digest(partition.partition_id),
            live_counts: LiveRecordCounts::from_partition(partition),
            entity_slots: partition.entity_arena.slot_count(),
            relation_slots: partition.relation_arena.slot_count(),
            unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                + partition.relation_arena.lifecycle_counts().unavailable,
        };
        Basis {
            cold,
            source,
            reference_digest: partition
                .partition_content_digest_for_test(symbols)
                .unwrap(),
        }
    })
}

/// A committed region with history and a second real basis carrying one more
/// committed version of the same entity, both encoded by the production codec.
pub(super) fn bases() -> Bases {
    let HistoryPartition {
        partition,
        entity,
        schema,
        symbols,
        ..
    } = populated_partition();
    let mut edited = partition.clone();
    let slot = entity.slot_index();
    let extra = edited.entity_arena.get_slot(slot).unwrap().extra().clone();
    edited
        .entity_arena
        .apply_extra_update_for_test(slot, extra, VersionId(1_000))
        .expect("fixture slot is materialized");
    Bases {
        entity,
        original: encode(&partition, &schema, &symbols),
        edited: encode(&edited, &schema, &symbols),
    }
}

pub(super) fn budget(head: u64, historical: u64) -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: head,
        historical_working_set_bytes: historical,
        max_in_flight_fill_bytes: 1 << 30,
        streaming_window_bytes: 1 << 16,
    }
}

pub(super) fn bound_cell(
    basis: &Basis,
    accountant: &Arc<WorkingSetAccountant>,
    pool: ResidencyPoolKind,
) -> Arc<RelationalRegionResidency> {
    let cell = RelationalRegionResidency::cold(basis.cold.clone()).unwrap();
    cell.bind_working_set(accountant, pool).unwrap();
    cell
}

/// A root of `count` independent cold-backed cells over `basis`.
pub(super) fn bound_root(
    basis: &Basis,
    count: usize,
    accountant: &Arc<WorkingSetAccountant>,
) -> Vec<RelationalRegionRetention> {
    (0..count)
        .map(|_| {
            RelationalRegionRetention::new(
                bound_cell(basis, accountant, ResidencyPoolKind::Head),
                None,
            )
        })
        .collect()
}

/// Measured working-set bytes of one Head image and of the additional Full
/// image, charged by the production loader under an unconstrained budget.
pub(super) fn image_bytes(basis: &Basis) -> (u64, u64) {
    let accountant = WorkingSetAccountant::new(budget(u64::MAX, u64::MAX));
    let cell = bound_cell(basis, &accountant, ResidencyPoolKind::Head);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let head = cell.demand_head(execution, LIMIT, None).unwrap();
    let head_bytes = accountant.charged_bytes(ResidencyPoolKind::Head);
    let full = cell.demand_full(execution, LIMIT, None).unwrap();
    let full_bytes = accountant.charged_bytes(ResidencyPoolKind::Head) - head_bytes;
    drop((head, full));
    assert!(head_bytes > 0 && full_bytes > 0, "loads are charged");
    (head_bytes, full_bytes)
}

/// The served bytes match the basis's independent memory reference.
pub(super) fn assert_basis(lease: &RelationalResidencyLease<Full>, basis: &Basis) {
    let symbols = basis.cold.symbols.interner_snapshot();
    let served: &PartitionState = lease;
    assert_eq!(
        served.partition_content_digest_for_test(&symbols).unwrap(),
        basis.reference_digest,
        "a rewarmed image serves exactly its own basis"
    );
}

pub(super) fn memory_exhausted(failure: &RelationalResidencyDemandFailure) -> bool {
    matches!(
        failure,
        RelationalResidencyDemandFailure::StorageUnavailable(unavailable)
            if unavailable.cause == Cause::MemoryExhausted
    )
}
