//! The aggregate owner's projection at the current coupled pin.
use super::*;
use crate::aggregate_identity::AggregateCandidateRead;

impl RelationalAuthoredCoupledCandidate<'_> {
    pub(super) fn aggregate_read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<AggregateCandidateRead>, TransactionCommitError> {
        let Some(aggregate) = &self.mutation.aggregate else {
            return Ok(None);
        };
        // The projection comes from the applied effect. Of the commands it
        // depends only on the transaction id, so nothing is copied for it.
        let transaction = self
            .transaction
            .prepared_coupled_commands
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .expect("the ordinary merger carried the authored prefix")
            .transaction_id;
        aggregate
            .borrow_retained(self.mutation.state())
            .candidate_read(execution, transaction)
            .map(Some)
    }
}
