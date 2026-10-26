pub(super) struct FinalizationInput<'a> {
    pub(super) clone_mode: crate::storage::overlay::PartitionCloneMode,
    pub(super) committed_partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        (
            crate::storage::overlay::PartitionState,
            crate::storage::overlay::PartitionMutationJournal,
        ),
    >,
    pub(super) prepared_history: crate::mvcc::PreparedRelationalPublicationAccelerators,
    pub(super) changed_records: &'a [crate::transactions::data::RecordRef],
    pub(super) version_id: crate::identity::data::VersionId,
    pub(super) previous_branch_head_version: Option<crate::identity::data::VersionId>,
    pub(super) commit_id: crate::history::data::CommitId,
    pub(super) commit_reference: &'a crate::history::data::RelationalCommitReceipt,
    pub(super) branch_id: &'a crate::history::data::BranchId,
    pub(super) merge_base_commits: &'a [crate::history::data::CommitId],
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) patch_position: crate::publication::patch::data::PatchStreamPosition,
    pub(super) merge_parent_branches: &'a [crate::history::data::BranchId],
    pub(super) phase_timing:
        &'a mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
    pub(super) published_snapshot_basis:
        crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(super) published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
}

pub(super) fn finalize_published_commit(
    runtime: &crate::runtime::RelationalRuntime,
    input: FinalizationInput<'_>,
) {
    let FinalizationInput {
        clone_mode,
        committed_partitions,
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
    let index_refresh_basis = prepared_history.index_refresh_basis().clone();
    publish_storage(
        runtime,
        branch_id,
        clone_mode,
        committed_partitions,
        phase_timing,
    );
    refresh_indexes(runtime, changed_records, &index_refresh_basis, phase_timing);
    let started = std::time::Instant::now();
    prepared_history.install(runtime, patch_position);
    phase_timing.history_publish_micros = phase_timing
        .history_publish_micros
        .saturating_add(started.elapsed().as_micros() as u64);
    advance_visibility(
        runtime,
        branch_id,
        previous_branch_head_version,
        version_id,
        &published_snapshot_basis,
        phase_timing,
    );
    // Only main publishes into the mirror. A sibling's changed identities
    // carry no authority to trim or reconcile the mirror's record versions.
    if branch_id == &runtime.history.main_branch {
        run_retention(runtime, changed_records, version_id, phase_timing);
    }
    compact_durability(runtime, phase_timing);
    let snapshot_id = publish_artifacts(
        runtime,
        version_id,
        artifacts,
        patch_position,
        published_snapshot_basis,
        published_snapshot_slot,
        phase_timing,
    );
    consume_post_commit_artifacts(
        runtime,
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
    runtime: &crate::runtime::RelationalRuntime,
    branch_id: &crate::history::data::BranchId,
    clone_mode: crate::storage::overlay::PartitionCloneMode,
    committed_partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        (
            crate::storage::overlay::PartitionState,
            crate::storage::overlay::PartitionMutationJournal,
        ),
    >,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    runtime
        .storage_authority()
        .publish_branch_partition_commits(branch_id, clone_mode, committed_partitions);
    timing.storage_commit_micros = started.elapsed().as_micros() as u64;
}

fn refresh_indexes(
    runtime: &crate::runtime::RelationalRuntime,
    changed_records: &[crate::transactions::data::RecordRef],
    basis: &crate::mvcc::PreparedIndexRefreshBasis,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    if basis.branch_id() == &runtime.history.main_branch {
        runtime
            .index_authority()
            .refresh_unique_entity_aspect_field_index_for_records(changed_records, basis);
    }
    timing.index_refresh_micros = started.elapsed().as_micros() as u64;
}

fn advance_visibility(
    runtime: &crate::runtime::RelationalRuntime,
    branch_id: &crate::history::data::BranchId,
    previous_branch_head_version: Option<crate::identity::data::VersionId>,
    version_id: crate::identity::data::VersionId,
    published_snapshot_basis: &crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    runtime
        .visibility_pins()
        .move_branch_head_visibility_residency(
            branch_id,
            previous_branch_head_version,
            Some(version_id),
            Some(published_snapshot_basis),
        );
    timing.visibility_pin_micros = started.elapsed().as_micros() as u64;
}

fn run_retention(
    runtime: &crate::runtime::RelationalRuntime,
    changed_records: &[crate::transactions::data::RecordRef],
    version_id: crate::identity::data::VersionId,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    let retention = runtime.retention();
    retention.reconcile_changed_record_states(changed_records, version_id);
    retention.trim_live_history_for_records(changed_records, version_id);
    timing.retention_trim_micros = started.elapsed().as_micros() as u64;
}

fn compact_durability(
    runtime: &crate::runtime::RelationalRuntime,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    runtime.durability_authority().compact_log_if_needed();
    timing.compaction_micros = started.elapsed().as_micros() as u64;
}

fn publish_artifacts(
    runtime: &crate::runtime::RelationalRuntime,
    version_id: crate::identity::data::VersionId,
    artifacts: crate::storage::overlay::PublicationArtifacts,
    patch_position: crate::publication::patch::data::PatchStreamPosition,
    basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    slot: crate::runtime::PublishedSnapshotSlotReservation,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) -> crate::snapshots::data::SnapshotId {
    let started = std::time::Instant::now();
    let snapshot_id = runtime.publication_authority().publish_artifacts(
        version_id,
        artifacts,
        patch_position,
        basis,
        slot,
    );
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
    runtime: &crate::runtime::RelationalRuntime,
    input: PostCommitArtifactInput<'_>,
    timing: &mut crate::authority::commit::phases::finalize::PublicationPhaseTiming,
) {
    let started = std::time::Instant::now();
    runtime
        .publication_authority()
        .consume_post_commit_artifacts(
            input.commit_id,
            input.snapshot_id,
            input.branch_id.clone(),
            &input.commit_reference.parents,
            input.merge_parent_branches,
            input.merge_base_commits,
        );
    timing.post_commit_consumer_micros = started.elapsed().as_micros() as u64;
}
