use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use super::super::rejection::elapsed_micros;
use super::preparation::PreparedPublicationCompletion;
use super::PublicationSettlementInputs;
use contracts::history::data::coordinates::CommitId;
use worth_relational_durability::internal_port::runtime::state::subsystems::durability::commit_log::DurabilityCommitLog;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::commit_log::CommitPhase;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use std::sync::Arc;
use contracts::transactions::data::commit_log::CommitLogOwnerPort as _;

pub mod finalization;

use finalization::{finalize_published_commit, FinalizationInput};

pub(super) struct FinalizedPublicationPhase {
    positioned_commit:
        Arc<branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    changed_records: Vec<contracts::transactions::data::primitives::RecordRef>,
}

impl FinalizedPublicationPhase {
    pub(super) fn into_parts(
        self,
    ) -> (
        Arc<branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
        Vec<contracts::transactions::data::primitives::RecordRef>,
    ) {
        (self.positioned_commit, self.changed_records)
    }
}

pub(super) struct CanonicalLogPhaseInput<'a> {
    pub(super) commit_log: &'a mut CommitLog,
    pub(super) phase_timing: &'a mut CommitPhaseTiming,
    pub(super) positioned_commit:
        &'a branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    pub(super) commit_id: CommitId,
    pub(super) envelope_cell:
        Arc<branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
}

/// Record the settled commit in the runtime's own canonical log. A durable
/// medium already holds the commit, so this step has no failure to report.
fn record_canonical_log_phase(
    durability: &DurabilityCommitLog<'_>,
    input: CanonicalLogPhaseInput<'_>,
) {
    let CanonicalLogPhaseInput {
        commit_log,
        phase_timing,
        positioned_commit,
        commit_id,
        envelope_cell,
    } = input;
    let branch_id = &positioned_commit.envelope().commit.branch_id;
    commit_log.begin_phase(CommitPhase::DurableAppend);
    let phase_started = std::time::Instant::now();
    commit_log.record_durable_append_prepared(
        commit_id,
        &branch_id.0,
        positioned_commit.position(),
    );
    durability.append(positioned_commit, envelope_cell);
    commit_log.complete_phase(CommitPhase::DurableAppend);
    phase_timing.durable_append_micros = elapsed_micros(phase_started);
}

pub(super) fn finalize_publication_phase(
    inputs: &PublicationSettlementInputs<'_>,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    prepared: PreparedPublicationCompletion,
    published_snapshot_basis: branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
    published_snapshot_slot: branch::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    positioned_commit: Arc<
        branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    >,
    envelope_cell: Arc<
        branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency,
    >,
) -> FinalizedPublicationPhase {
    let PreparedPublicationCompletion {
        mirror_publication,
        prepared_history,
        validated_lineage_events,
        lineage_nodes,
        relation_lineages,
        artifacts,
        changed_records,
        version_id,
        previous_branch_head_version,
        commit_id,
        commit_reference,
        branch_id,
        merge_base_commits,
        merge_parent_branches,
        deferred_diagnostic_artifacts,
    } = prepared;
    commit_log.record_publication_position(positioned_commit.position());
    record_canonical_log_phase(
        &inputs.log,
        CanonicalLogPhaseInput {
            commit_log,
            phase_timing,
            positioned_commit: positioned_commit.as_ref(),
            commit_id,
            envelope_cell,
        },
    );
    inputs.lineage.install_published_lineage(
        validated_lineage_events,
        commit_id,
        lineage_nodes,
        relation_lineages,
    );
    commit_log.begin_phase(CommitPhase::Publication);
    let phase_started = std::time::Instant::now();
    let mut publication_phase_timing =
        crate::authority::commit::phases::finalize::PublicationPhaseTiming::default();
    finalize_published_commit(
        inputs,
        FinalizationInput {
            mirror_publication,
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
        inputs.publication.push_diagnostic_artifact(artifact);
    }
    commit_log.record_commit_published(commit_id, &commit_reference.branch_id.0);
    commit_log.complete_phase(CommitPhase::Publication);
    phase_timing.publication_micros = elapsed_micros(phase_started);
    phase_timing.publication_storage_commit_micros = publication_phase_timing.storage_commit_micros;
    phase_timing.publication_index_refresh_micros = publication_phase_timing.index_refresh_micros;
    phase_timing.publication_history_publish_micros =
        publication_phase_timing.history_publish_micros;
    phase_timing.publication_visibility_pin_micros = publication_phase_timing.visibility_pin_micros;
    phase_timing.publication_retention_reconcile_micros =
        publication_phase_timing.retention_reconcile_micros;
    phase_timing.publication_compaction_micros = publication_phase_timing.compaction_micros;
    phase_timing.publication_bundle_publish_micros = publication_phase_timing.bundle_publish_micros;
    phase_timing.publication_retention_pass_micros = publication_phase_timing.retention_pass_micros;
    phase_timing.publication_post_commit_consumer_micros =
        publication_phase_timing.post_commit_consumer_micros;

    FinalizedPublicationPhase {
        positioned_commit,
        changed_records,
    }
}
