use crate::runtime::RelationalRuntime;
use crate::transactions::data::TransactionCommitError;

impl RelationalRuntime {
    /// Convert one owner-validated proposal into the opaque, single-use
    /// candidate accepted by the independently borrowable publication port.
    pub fn prepare_validated_proposal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        proposal: crate::mvcc::ValidatedRelationalProposal,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Admission,
        )?;
        self.preparation_port()
            .prepare_consulted_validated_proposal(execution, proposal)
    }
}
