use crate::history::data::RelationalCommitReceipt;
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::{LineageCommitPostureInput, PreparedLineageFinalization};
use crate::runtime::WorkingState;
use crate::transactions::data::{MutationIntent, RecordRef, TransactionCommitError};

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn ensure_lineage_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        self.ensure_lineage_for_commit_with_postures(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            &LineageCommitPostureInput::default(),
        )
    }
}
