use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_runtime_bridge::facade::{
    BridgeConditionalSignalBasisBinding, BridgeSealedRuntimeAssembly,
};

use super::{
    ErasedClockObservationOutcome, WorthQueryConditionalClockObservationFailureKind,
    WorthQueryConditionalTruthBasis,
};
use crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake;
use crate::domain_computation::primary_graph::output_lineage::invalidation::CommitTouchesStop;

pub(super) struct AuthoritativeClockWork<'a, Schema> {
    pub(super) runtime:
        &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    pub(super) bridge: &'a BridgeSealedRuntimeAssembly,
    pub(super) signal_basis: &'a BridgeConditionalSignalBasisBinding,
    pub(super) relational_branch: &'a worth_relational::facade::history::BranchId,
    pub(super) relational_commit_ceiling: Option<worth_relational::facade::history::CommitId>,
    pub(super) cursor: &'a mut Option<worth_relational::facade::publication::PatchStreamPosition>,
    pub(super) maximum_commits: usize,
    pub(super) interest:
        crate::domain_computation::primary_graph::output_lineage::invalidation::CommitTouchInterest<
            'a,
        >,
    /// Receives every invalidation consumed from the subscription, so a
    /// later failure or refused reading cannot drop it.
    pub(super) pending: &'a mut super::WorthQueryPendingGranularInvalidations,
    pub(super) retained_wakes: &'a mut [WorthQueryRetainedConditionalWake],
    pub(super) runtime_binding_identity: &'a str,
    pub(super) runtime_capability_identity: u64,
    pub(super) truth: &'a WorthQueryConditionalTruthBasis,
}

pub(super) struct AuthoritativeClockProgress {
    pub(super) commit_count: usize,
    pub(super) work_remaining: bool,
    pub(super) caught_up_to_latest: bool,
}

/// Why authoritative progression stopped before the clock advance.
pub(super) enum AuthoritativeClockStop {
    /// The cursor fell behind the subscription's retained window; truth at the
    /// selected ceiling must be reconstructed, then progress resumes after
    /// `resume_after`.
    Lagging {
        resume_after: Option<worth_relational::facade::publication::PatchStreamPosition>,
    },
    Failed(ErasedClockObservationOutcome),
}

pub(super) fn reconsider_authoritative_clock_work<Schema>(
    phase: &WorthQueryAdvancementPhase<'_>,

    work: AuthoritativeClockWork<'_, Schema>,
) -> Result<AuthoritativeClockProgress, AuthoritativeClockStop> {
    let commits = super::super::authoritative_reconsideration::relevant_authoritative_commits(
        work.runtime,
        work.relational_branch,
        work.relational_commit_ceiling,
        *work.cursor,
        work.maximum_commits,
        work.interest,
    )
    .map_err(|stop| match stop {
        CommitTouchesStop::RequiresReconstruction { resume_after } => {
            AuthoritativeClockStop::Lagging { resume_after }
        }
    })?;
    let commit_count = commits.commit_count();
    let delivered = super::super::authoritative_reconsideration::deliver_authoritative_commits(
        phase,
        work.bridge,
        work.signal_basis,
        work.cursor,
        commits,
        work.retained_wakes,
        work.runtime_binding_identity,
        work.runtime_capability_identity,
        work.truth,
        work.pending,
    )
    .map_err(|cause| {
        AuthoritativeClockStop::Failed(ErasedClockObservationOutcome::Failed {
            detail: format!("authoritative delivery refused: {cause:?}"),
            kind: WorthQueryConditionalClockObservationFailureKind::AuthoritativeDelivery(cause),
        })
    })?;
    Ok(AuthoritativeClockProgress {
        commit_count,
        work_remaining: delivered.work_remaining,
        caught_up_to_latest: delivered.caught_up_to_latest,
    })
}

/// In-observation reconstruction was itself denied.
pub(super) fn reconstruction_failure(
    denial: super::super::installation::WorthQueryConditionalRuntimeInstallationDenial,
) -> ErasedClockObservationOutcome {
    ErasedClockObservationOutcome::Failed {
        kind: WorthQueryConditionalClockObservationFailureKind::ReconstructionDenied(denial.kind()),
        detail: denial.subject().to_string(),
    }
}

pub(super) fn runtime_rejection(detail: String) -> ErasedClockObservationOutcome {
    ErasedClockObservationOutcome::Failed {
        kind: WorthQueryConditionalClockObservationFailureKind::RuntimeRejected,
        detail,
    }
}
