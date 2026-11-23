//! Select exact History operands and the candidate round that can produce a merge draft.
use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::candidate_preparation::CandidateHistoryInputs;
use crate::authority::commit::pipeline::execution_admission::AdmittedCommitState;
use crate::authority::commit::pipeline::history_preparation::HistoryPreparationInputs;
use crate::authority::commit::pipeline::merge_draft::{prepare_merge_draft, AcceptedMergeDraft};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl RelationalPreparationRuntime {
    pub(crate) fn candidate_history_inputs(
        &self,
    ) -> CandidateHistoryInputs<
        '_,
        impl FnOnce(
                &ExecutionRequest<'_, '_>,
                &AdmittedCommitState,
            ) -> Result<AcceptedMergeDraft, TransactionCommitError>
            + '_,
    > {
        CandidateHistoryInputs {
            runtime_identity: self.runtime_instance_id(),
            publication: &self.publication_owner,
            history: &self.history,
            diagnostics: &self.diagnostics,
            prepare: HistoryPreparationInputs {
                admission: self.into(),
                continuity: self.into(),
                descriptor_semantics: self
                    .config
                    .schema
                    .descriptor_semantics_policy
                    .current_write_version(),
                merge_draft: move |execution: &ExecutionRequest<'_, '_>,
                                   admitted: &AdmittedCommitState| {
                    prepare_merge_draft(execution, self, admitted)
                },
            },
        }
    }
}
