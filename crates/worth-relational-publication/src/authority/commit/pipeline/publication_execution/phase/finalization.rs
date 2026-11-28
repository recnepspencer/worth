use worth_relational_contracts::internal_port::execution::monotonic_time::{elapsed_since, monotonic_now};
use super::PublicationSettlementInputs;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub(super) struct FinalizationInput<'a> {
    pub(super) mirror_publication: worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication,
    pub(super) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublicationAccelerators,
    pub(super) changed_records: &'a [contracts::transactions::data::primitives::RecordRef],
    pub(super) version_id: contracts::identity::data::primitives::VersionId,
    pub(super) previous_branch_head_version: Option<contracts::identity::data::primitives::VersionId>,
    pub(super) branch_id: &'a contracts::history::data::coordinates::BranchId,
    pub(super) reserved_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    pub(super) patch_position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    pub(super) phase_timing:
        &'a mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
    pub(super) published_snapshot_basis: branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
    pub(super) published_snapshot_slot:
        branch::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
}

pub(super) fn finalize_published_commit(
    inputs: &PublicationSettlementInputs<'_>,
    input: FinalizationInput<'_>,
) {
    let FinalizationInput {
        mirror_publication,
        prepared_history,
        changed_records,
        version_id,
        previous_branch_head_version,
        branch_id,
        reserved_snapshot,
        patch_position,
        phase_timing,
        published_snapshot_basis,
        published_snapshot_slot,
    } = input;
    publish_storage(
        inputs.retention.partitions,
        inputs.retention.counters,
        mirror_publication,
        phase_timing,
    );
    let started = monotonic_now();
    prepared_history.install(inputs.history, patch_position);
    phase_timing.history_publish_micros = phase_timing
        .history_publish_micros
        .saturating_add(elapsed_since(started).as_micros() as u64);
    advance_visibility(
        inputs,
        branch_id,
        previous_branch_head_version,
        version_id,
        &published_snapshot_basis,
        phase_timing,
    );
    // Only main publishes into the mirror. A sibling's changed identities
    // carry no authority to trim or reconcile the mirror's record versions.
    if branch_id == &inputs.history.main_branch {
        reconcile_published_retention(inputs, changed_records, version_id, phase_timing);
    }
    compact_durability(&inputs.log, inputs.log_policy, phase_timing);
    install_published_snapshot(
        &inputs.publication,
        reserved_snapshot,
        published_snapshot_basis,
        published_snapshot_slot,
        phase_timing,
    );

}

fn publish_storage(
    partitions: &worth_relational_storage::internal_port::runtime::state::subsystems::storage::StorageSubsystem,
    counters: &std::sync::Mutex<
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    >,
    prepared: worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = monotonic_now();
    worth_relational_storage::internal_port::storage::mirror_publication::publish_prepared_mirror(
        partitions, counters, prepared,
    );
    timing.storage_commit_micros = elapsed_since(started).as_micros() as u64;
}

fn advance_visibility(
    inputs: &PublicationSettlementInputs<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    previous_branch_head_version: Option<contracts::identity::data::primitives::VersionId>,
    version_id: contracts::identity::data::primitives::VersionId,
    published_snapshot_basis: &branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = monotonic_now();
    inputs.visibility.move_residency(
        branch_id,
        previous_branch_head_version,
        Some(version_id),
        Some(published_snapshot_basis),
    );
    timing.visibility_pin_micros = elapsed_since(started).as_micros() as u64;
}

fn reconcile_published_retention(
    inputs: &PublicationSettlementInputs<'_>,
    changed_records: &[contracts::transactions::data::primitives::RecordRef],
    version_id: contracts::identity::data::primitives::VersionId,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = monotonic_now();
    let fence = inputs
        .visibility
        .directory
        .retention_fence_version(version_id);
    inputs.retention.reconcile(changed_records, fence);
    timing.retention_reconcile_micros = elapsed_since(started).as_micros() as u64;
}

fn compact_durability(
    log: &worth_relational_durability::internal_port::runtime::state::subsystems::durability::commit_log::DurabilityCommitLog<'_>,
    policy: &contracts::config::data::policies::DurableLogPolicy,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = monotonic_now();
    log.compact_if_needed(policy);
    timing.compaction_micros = elapsed_since(started).as_micros() as u64;
}

fn install_published_snapshot(
    publication: &crate::publication::authority::PublicationAuthority<'_>,
    reserved_snapshot: worth_relational_branch::internal_port::snapshots::data::SnapshotHandle,
    basis: branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
    slot: branch::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = monotonic_now();
    publication.install_published_snapshot(reserved_snapshot, basis, slot);
    timing.bundle_publish_micros = elapsed_since(started).as_micros() as u64;
}
