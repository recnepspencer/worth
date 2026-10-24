use super::super::rejection::elapsed_micros;
use super::preparation::PreparedPublicationCompletion;
use crate::authority::commit::phases::publication::append_durable_commit;
use crate::history::data::{BranchId, CommitId};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{
    CommitLog, CommitPhase, CommitPhaseTiming, TransactionCommitError,
};
use std::sync::Arc;

mod finalization;

use finalization::{finalize_published_commit, FinalizationInput};

pub(super) struct FinalizedPublicationPhase {
    positioned_commit: Arc<crate::history::data::PositionedCanonicalCommit>,
    changed_records: Vec<crate::transactions::data::RecordRef>,
    durability_error: Option<TransactionCommitError>,
}

impl FinalizedPublicationPhase {
    pub(super) fn into_parts(
        self,
    ) -> (
        Arc<crate::history::data::PositionedCanonicalCommit>,
        Vec<crate::transactions::data::RecordRef>,
        Option<TransactionCommitError>,
    ) {
        (
            self.positioned_commit,
            self.changed_records,
            self.durability_error,
        )
    }
}

pub(super) struct DurableAppendPhaseInput<'a> {
    pub(super) commit_log: &'a mut CommitLog,
    pub(super) phase_timing: &'a mut CommitPhaseTiming,
    pub(super) positioned_commit: &'a crate::history::data::PositionedCanonicalCommit,
    pub(super) append_authority: crate::durability::authority::DurableAppendAuthority,
    pub(super) commit_id: CommitId,
    pub(super) branch_id: &'a BranchId,
}

pub(super) fn append_durable_commit_phase(
    runtime: &RelationalRuntime,
    input: DurableAppendPhaseInput<'_>,
) -> Result<(), TransactionCommitError> {
    let DurableAppendPhaseInput {
        commit_log,
        phase_timing,
        positioned_commit,
        append_authority,
        commit_id,
        branch_id,
    } = input;
    commit_log.begin_phase(CommitPhase::DurableAppend);
    let phase_started = std::time::Instant::now();
    commit_log.record_durable_append_prepared(
        commit_id,
        &branch_id.0,
        positioned_commit.position(),
    );
    append_durable_commit(runtime, append_authority, positioned_commit)
        .map_err(|error| error.with_commit_log(commit_log.clone()))?;
    commit_log.complete_phase(CommitPhase::DurableAppend);
    phase_timing.durable_append_micros = elapsed_micros(phase_started);
    Ok(())
}

pub(super) fn finalize_publication_phase(
    runtime: &RelationalRuntime,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    prepared: PreparedPublicationCompletion,
    published_snapshot_basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
) -> Result<FinalizedPublicationPhase, TransactionCommitError> {
    let PreparedPublicationCompletion {
        clone_mode,
        committed_partitions,
        prepared_history,
        validated_lineage_events,
        lineage_nodes,
        artifacts,
        changed_records,
        append_authority,
        version_id,
        previous_branch_head_version,
        commit_id,
        commit_reference,
        branch_id,
        merge_base_commits,
        merge_parent_branches,
        deferred_diagnostic_artifacts,
    } = prepared;
    let positioned_commit = runtime
        .history
        .positioned_canonical_commit(commit_id)
        .expect("performed publication must expose its positioned canonical commit");
    commit_log.record_publication_position(positioned_commit.position());
    let durability_error = append_durable_commit_phase(
        runtime,
        DurableAppendPhaseInput {
            commit_log,
            phase_timing,
            positioned_commit: positioned_commit.as_ref(),
            append_authority,
            commit_id,
            branch_id: &branch_id,
        },
    )
    .err();
    runtime.lineage_authority().install_published_lineage(
        validated_lineage_events,
        commit_id,
        lineage_nodes,
    );
    commit_log.begin_phase(CommitPhase::Publication);
    let phase_started = std::time::Instant::now();
    let mut publication_phase_timing =
        crate::authority::commit::phases::finalize::PublicationPhaseTiming::default();
    finalize_published_commit(
        runtime,
        FinalizationInput {
            clone_mode,
            committed_partitions,
            prepared_history,
            changed_records: &changed_records,
            version_id,
            previous_branch_head_version,
            commit_id,
            commit_reference: &commit_reference,
            branch_id: &branch_id,
            merge_base_commits: &merge_base_commits,
            artifacts,
            patch_position: positioned_commit.position(),
            merge_parent_branches: &merge_parent_branches,
            phase_timing: &mut publication_phase_timing,
            published_snapshot_basis,
            published_snapshot_slot,
        },
    );
    for artifact in deferred_diagnostic_artifacts {
        runtime
            .publication_authority()
            .push_diagnostic_artifact(artifact);
    }
    commit_log.record_commit_published(commit_id, &commit_reference.branch_id.0);
    commit_log.complete_phase(CommitPhase::Publication);
    phase_timing.publication_micros = elapsed_micros(phase_started);
    phase_timing.publication_storage_commit_micros = publication_phase_timing.storage_commit_micros;
    phase_timing.publication_index_refresh_micros = publication_phase_timing.index_refresh_micros;
    phase_timing.publication_history_publish_micros =
        publication_phase_timing.history_publish_micros;
    phase_timing.publication_visibility_pin_micros = publication_phase_timing.visibility_pin_micros;
    phase_timing.publication_retention_trim_micros = publication_phase_timing.retention_trim_micros;
    phase_timing.publication_compaction_micros = publication_phase_timing.compaction_micros;
    phase_timing.publication_bundle_publish_micros = publication_phase_timing.bundle_publish_micros;
    phase_timing.publication_retention_pass_micros = publication_phase_timing.retention_pass_micros;
    phase_timing.publication_post_commit_consumer_micros =
        publication_phase_timing.post_commit_consumer_micros;

    Ok(FinalizedPublicationPhase {
        positioned_commit,
        changed_records,
        durability_error,
    })
}
