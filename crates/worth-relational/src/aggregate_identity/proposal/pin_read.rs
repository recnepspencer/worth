//! A candidate pin read prepares the projection only, never a consequence plan.
use super::*;
use crate::transactions::data::TransactionId;

impl AggregateProposal {
    fn pin_projection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &SelectedRelationalBranchRead<'_>,
        after: &WorkingState,
        transaction: TransactionId,
    ) -> Result<Arc<AggregateProjection>, TransactionCommitError> {
        if transaction != self.identity.transaction_id() {
            return Err(binding_error(
                "aggregate proposal business plan affinity differs",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &*state {
            ArtifactState::Ready(artifact) => return Ok(Arc::clone(&artifact.projection)),
            ArtifactState::Consumed(projection) | ArtifactState::Projected(projection) => {
                return Ok(Arc::clone(projection));
            }
            ArtifactState::Failed(error) => return Err(error.clone()),
            ArtifactState::Uninitialized => {}
        }
        let changes = self
            .changes
            .as_ref()
            .ok_or_else(|| binding_error("physical maintenance reads stored logical truth"))?;
        match super::super::preparation::prepare(
            execution,
            None,
            &self.authority,
            selected,
            after,
            super::super::preparation::BusinessPlan::Transaction(transaction),
            changes,
            self.identity.proposed_version_id(),
            self.identity.commit_id(),
        ) {
            Ok(artifact) => {
                let projection = Arc::clone(&artifact.projection);
                *state = ArtifactState::Projected(Arc::clone(&projection));
                Ok(projection)
            }
            Err(error) => {
                *state = ArtifactState::Failed(error.clone());
                Err(error)
            }
        }
    }
}

impl AggregateProposalBorrow<'_> {
    /// The candidate read at a coupled pin. It takes the business plan's
    /// transaction id and nothing else of the commands.
    pub(crate) fn candidate_read(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: TransactionId,
    ) -> Result<super::super::AggregateCandidateRead, TransactionCommitError> {
        let projection =
            self.owner
                .pin_projection(execution, &self.selected, self.after, transaction)?;
        Ok(super::super::AggregateCandidateRead::new(
            projection,
            Arc::clone(self.owner.selected.state().root()),
            self.owner.selected.version_id(),
        ))
    }
}
