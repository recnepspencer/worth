//! Consume the exact draft operands before handing preparation to publication.
use super::{AdmittedCommitState, DraftValidationInput};

impl<V: DraftValidationInput> AdmittedCommitState<V> {
    pub(in crate::authority::commit::pipeline) fn into_publication(self) -> AdmittedCommitState {
        let Self {
            transaction_id,
            validation_input,
            selected_branch_state,
            phase_timing,
            commit_log,
            authority_input,
            merge_execution_accounting,
            merge_execution_diagnostics_plan,
            prevalidated_invariants,
            strategy_commit_artifacts,
            diagnostics,
            complexity_before,
            prior_complexity_delta,
        } = self;
        AdmittedCommitState {
            transaction_id,
            validation_input: validation_input.into_publication(),
            selected_branch_state,
            phase_timing,
            commit_log,
            authority_input,
            merge_execution_accounting,
            merge_execution_diagnostics_plan,
            prevalidated_invariants,
            strategy_commit_artifacts,
            diagnostics,
            complexity_before,
            prior_complexity_delta,
        }
    }
}
