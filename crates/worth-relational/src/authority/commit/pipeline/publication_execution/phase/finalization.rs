use super::PublicationSettlementInputs;

pub(super) struct FinalizationInput<'a> {
    pub(super) mirror_publication: crate::storage::PreparedMirrorPublication,
    pub(super) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublicationAccelerators,
    pub(super) changed_records: &'a [crate::transactions::data::primitives::RecordRef],
    pub(super) version_id: crate::identity::data::VersionId,
    pub(super) previous_branch_head_version: Option<crate::identity::data::VersionId>,
    pub(super) commit_id: crate::history::data::CommitId,
    pub(super) commit_reference: &'a crate::history::data::RelationalCommitReceipt,
    pub(super) branch_id: &'a crate::history::data::BranchId,
    pub(super) merge_base_commits: &'a [crate::history::data::CommitId],
    pub(super) artifacts: crate::publication::authority::artifacts::PublicationArtifacts,
    pub(super) patch_position: crate::publication::patch::data::PatchStreamPosition,
    pub(super) merge_parent_branches: &'a [crate::history::data::BranchId],
    pub(super) phase_timing:
        &'a mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
    pub(super) published_snapshot_basis: crate::history::retention::VisibilitySnapshotBasis,
    pub(super) published_snapshot_slot:
        crate::runtime::state::subsystems::visibility::PublishedSnapshotSlotReservation,
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
        commit_id,
        commit_reference,
        branch_id,
        merge_base_commits,
        artifacts,
        patch_position,
        merge_parent_branches,
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
    let started = std::time::Instant::now();
    prepared_history.install(inputs.history, patch_position);
    phase_timing.history_publish_micros = phase_timing
        .history_publish_micros
        .saturating_add(started.elapsed().as_micros() as u64);
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
    let snapshot_id = publish_artifacts(
        &inputs.publication,
        version_id,
        artifacts,
        patch_position,
        published_snapshot_basis,
        published_snapshot_slot,
        phase_timing,
    );
    consume_post_commit_artifacts(
        &inputs.publication,
        PostCommitArtifactInput {
            commit_id,
            snapshot_id,
            branch_id,
            commit_reference,
            merge_parent_branches,
            merge_base_commits,
        },
        phase_timing,
    );
}

fn publish_storage(
    partitions: &crate::runtime::state::subsystems::storage::StorageSubsystem,
    counters: &std::sync::Mutex<
        crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    >,
    prepared: crate::storage::mirror_plan::PreparedMirrorPublication,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    crate::storage::mirror_publication::publish_prepared_mirror(partitions, counters, prepared);
    timing.storage_commit_micros = started.elapsed().as_micros() as u64;
}

fn advance_visibility(
    inputs: &PublicationSettlementInputs<'_>,
    branch_id: &crate::history::data::BranchId,
    previous_branch_head_version: Option<crate::identity::data::VersionId>,
    version_id: crate::identity::data::VersionId,
    published_snapshot_basis: &crate::history::retention::VisibilitySnapshotBasis,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    inputs.visibility.move_residency(
        branch_id,
        previous_branch_head_version,
        Some(version_id),
        Some(published_snapshot_basis),
    );
    timing.visibility_pin_micros = started.elapsed().as_micros() as u64;
}

fn reconcile_published_retention(
    inputs: &PublicationSettlementInputs<'_>,
    changed_records: &[crate::transactions::data::primitives::RecordRef],
    version_id: crate::identity::data::VersionId,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    let fence = inputs
        .visibility
        .directory
        .retention_fence_version(version_id);
    inputs.retention.reconcile(changed_records, fence);
    timing.retention_reconcile_micros = started.elapsed().as_micros() as u64;
}

fn compact_durability(
    log: &crate::runtime::state::subsystems::durability::commit_log::DurabilityCommitLog<'_>,
    policy: &crate::config::data::DurableLogPolicy,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    log.compact_if_needed(policy);
    timing.compaction_micros = started.elapsed().as_micros() as u64;
}

fn publish_artifacts(
    publication: &crate::publication::authority::PublicationAuthority<'_>,
    version_id: crate::identity::data::VersionId,
    artifacts: crate::publication::authority::artifacts::PublicationArtifacts,
    patch_position: crate::publication::patch::data::PatchStreamPosition,
    basis: crate::history::retention::VisibilitySnapshotBasis,
    slot: crate::runtime::state::subsystems::visibility::PublishedSnapshotSlotReservation,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) -> crate::snapshots::data::SnapshotId {
    let started = std::time::Instant::now();
    let snapshot_id =
        publication.publish_artifacts(version_id, artifacts, patch_position, basis, slot);
    timing.bundle_publish_micros = started.elapsed().as_micros() as u64;
    snapshot_id
}

struct PostCommitArtifactInput<'a> {
    commit_id: crate::history::data::CommitId,
    snapshot_id: crate::snapshots::data::SnapshotId,
    branch_id: &'a crate::history::data::BranchId,
    commit_reference: &'a crate::history::data::RelationalCommitReceipt,
    merge_parent_branches: &'a [crate::history::data::BranchId],
    merge_base_commits: &'a [crate::history::data::CommitId],
}

fn consume_post_commit_artifacts(
    publication: &crate::publication::authority::PublicationAuthority<'_>,
    input: PostCommitArtifactInput<'_>,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    publication.consume_post_commit_artifacts(
        input.commit_id,
        input.snapshot_id,
        input.branch_id.clone(),
        &input.commit_reference.parents,
        input.merge_parent_branches,
        input.merge_base_commits,
    );
    timing.post_commit_consumer_micros = started.elapsed().as_micros() as u64;
}
