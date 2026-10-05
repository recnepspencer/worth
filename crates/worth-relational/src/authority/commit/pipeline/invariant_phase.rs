use super::rejection::{attach_rejection, elapsed_micros};
use crate::identity::data::VersionId;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitLog, CommitPhase, CommitPhaseTiming, MergedCommitPlan};
use crate::validation::engine::InvariantExecutionResult;

pub(super) fn enforce_commit_boundary_phase(
    runtime: &RelationalPreparationRuntime,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    selected_branch_state: &crate::branch::SelectedRelationalBranchState,
    proposed_working_state: &crate::storage::overlay::WorkingState,
    proposed_version_id: crate::identity::data::VersionId,
    merged_plan: &MergedCommitPlan,
    proposal_identity: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
    prevalidated_commit_boundary: Option<InvariantExecutionResult>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::InvariantPreCheck);
    let phase_started = std::time::Instant::now();
    let pre_commit_invariants = match prevalidated_commit_boundary {
        Some(result) => result,
        None => runtime
            .invariant_authority()
            .enforce_commit_boundary_for_selected_branch(
                selected_branch_state,
                proposed_working_state,
                proposed_version_id,
                merged_plan,
                proposal_identity,
                lease,
            )
            .map_err(|error| attach_rejection(commit_log, CommitPhase::InvariantPreCheck, error))?,
    };
    commit_log.record_invariant_outcomes(&pre_commit_invariants);
    commit_log.complete_phase(CommitPhase::InvariantPreCheck);
    phase_timing.invariant_pre_check_micros = elapsed_micros(phase_started);
    Ok(pre_commit_invariants)
}

pub(super) fn enforce_snapshot_publication_phase(
    runtime: &RelationalPreparationRuntime,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    selected_branch_state: &crate::branch::SelectedRelationalBranchState,
    working_state: &crate::storage::overlay::WorkingState,
    version_id: VersionId,
    merged_plan: &MergedCommitPlan,
    proposal_identity: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
    prevalidated_snapshot_publication: Option<InvariantExecutionResult>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::InvariantPostCheck);
    let phase_started = std::time::Instant::now();
    let post_invariants = match prevalidated_snapshot_publication {
        Some(result) => result,
        None => runtime
            .invariant_authority()
            .enforce_snapshot_publication_for_working_state(
                selected_branch_state,
                working_state,
                version_id,
                merged_plan,
                proposal_identity,
                lease,
            )
            .map_err(|error| {
                attach_rejection(commit_log, CommitPhase::InvariantPostCheck, error)
            })?,
    };
    commit_log.record_invariant_outcomes(&post_invariants);
    commit_log.complete_phase(CommitPhase::InvariantPostCheck);
    phase_timing.invariant_post_check_micros = elapsed_micros(phase_started);
    Ok(post_invariants)
}
