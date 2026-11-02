//! Selection enters the same validated-proposal/candidate publication path.
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::mvcc::{
    PreparedRelationalCommitCandidate, RelationalPreparationPort,
    RelationalRetainedEntitySelection, RelationalRetainedEntitySelectionDenial as Denial,
    RelationalRetainedEntitySelectionPreparationError as Error,
};

impl RelationalPreparationPort {
    /// Prepare a new selection occurrence; this performs no publication.
    /// Publish or discard the returned candidate through the original owner.
    pub fn prepare_retained_entity_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        input: RelationalRetainedEntitySelection,
    ) -> Result<PreparedRelationalCommitCandidate, Error> {
        let configuration = self.binding.configuration_binding();
        let epoch = configuration.operation();
        let runtime = self.binding.runtime_snapshot_from(&epoch);
        let runtime = if execution.is_leased() {
            runtime.with_commit_work_budget(None)
        } else {
            runtime
        };
        let _operation = runtime
            .admit_operation()
            .ok_or(Error::Admission(Denial::OwnerUnavailable))?;
        runtime
            .require_retained_selection_profile()
            .map_err(Error::Admission)?;
        runtime
            .validate_retained_selection_sources(&input.transaction.basis, &input.historical)
            .map_err(Error::Admission)?;
        consult_stage(execution, CommitStage::Validation)
            .map_err(|denial| Error::Preparation(denial.into()))?;
        crate::mvcc::validation::require_not_interrupted(
            &input.transaction.control,
            &input.transaction.basis.inner.retention_binding,
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
        )
        .map_err(Error::Preparation)?;
        let retained =
            crate::mvcc::retained_entity_selection::payload::prepare(&runtime, execution, &input)?;
        let proposal = runtime
            .validate_consulted_retained_entity_selection(execution, input.transaction, retained)
            .map_err(Error::Preparation)?;
        consult_stage(execution, CommitStage::Admission)
            .map_err(|denial| Error::Preparation(denial.into()))?;
        self.prepare_validated_proposal_inner(execution, &runtime, proposal)
            .map_err(Error::Preparation)
    }
}
