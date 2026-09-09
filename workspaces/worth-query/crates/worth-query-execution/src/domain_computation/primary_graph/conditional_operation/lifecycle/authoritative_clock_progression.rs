use worth_runtime_bridge::facade::{
    BridgeConditionalSignalBasisBinding, BridgeSealedRuntimeAssembly,
};

use super::{
    ErasedClockObservationOutcome, WorthQueryConditionalClockObservationFailureKind,
    WorthQueryConditionalTruthBasis,
};
use crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake;

pub(super) struct AuthoritativeClockWork<'a, Schema> {
    pub(super) runtime:
        &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    pub(super) bridge: &'a BridgeSealedRuntimeAssembly,
    pub(super) signal_basis: &'a BridgeConditionalSignalBasisBinding,
    pub(super) relational_branch: &'a worth_relational::facade::history::BranchId,
    pub(super) relational_commit_ceiling: Option<worth_relational::facade::history::CommitId>,
    pub(super) cursor: &'a mut u64,
    pub(super) maximum_commits: usize,
    pub(super) watched_records: Vec<worth_relational::facade::transactions::RecordRef>,
    pub(super) include_whole_graph: bool,
    pub(super) bootstrap_identity: Option<&'a str>,
    pub(super) retained_wakes: &'a mut [WorthQueryRetainedConditionalWake],
    pub(super) runtime_binding_identity: &'a str,
    pub(super) runtime_capability_identity: u64,
    pub(super) truth: &'a WorthQueryConditionalTruthBasis,
}

pub(super) struct AuthoritativeClockProgress {
    pub(super) commit_count: usize,
    pub(super) work_remaining: bool,
    pub(super) caught_up_to_latest: bool,
    pub(super) granular_invalidations:
        Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
}

pub(super) fn reconsider_authoritative_clock_work<Schema>(
    work: AuthoritativeClockWork<'_, Schema>,
) -> Result<AuthoritativeClockProgress, ErasedClockObservationOutcome> {
    let commits = super::super::authoritative_reconsideration::relevant_authoritative_commits(
        work.runtime,
        work.relational_branch,
        work.relational_commit_ceiling,
        *work.cursor,
        work.maximum_commits,
        work.watched_records,
        work.include_whole_graph,
        work.bootstrap_identity,
    )
    .map_err(runtime_rejection)?;
    let commit_count = commits.commit_count();
    let delivered = super::super::authoritative_reconsideration::deliver_authoritative_commits(
        work.bridge,
        work.signal_basis,
        work.cursor,
        commits,
        work.retained_wakes,
        work.runtime_binding_identity,
        work.runtime_capability_identity,
        work.truth,
    )
    .map_err(runtime_rejection)?;
    Ok(AuthoritativeClockProgress {
        commit_count,
        work_remaining: delivered.work_remaining,
        caught_up_to_latest: delivered.caught_up_to_latest,
        granular_invalidations: delivered.granular_invalidations,
    })
}

pub(super) fn runtime_rejection(detail: String) -> ErasedClockObservationOutcome {
    ErasedClockObservationOutcome::Failed {
        kind: WorthQueryConditionalClockObservationFailureKind::RuntimeRejected,
        detail,
    }
}
