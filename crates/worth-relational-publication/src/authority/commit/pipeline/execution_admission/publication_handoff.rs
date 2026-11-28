//! Consume the exact draft operands before handing preparation to publication.
use super::{AdmittedCommitState, DraftValidationInput};

impl<V: DraftValidationInput> AdmittedCommitState<V> {
    pub(in crate::internal_port::authority::commit::pipeline) fn into_publication(
        self,
    ) -> AdmittedCommitState {
        let Self {
            transaction_id,
            validation_input,
            selected_branch_state,
            phase_timing,
            commit_log,
            authority_input,
            prevalidated_invariants,
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
            prevalidated_invariants,
            complexity_before,
            prior_complexity_delta,
        }
    }
}
