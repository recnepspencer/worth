//! Retain the existing aggregate owner's projection of exactly the applied parts.
use super::*;
use crate::{aggregate_identity::AggregateCandidateRead, transactions::data::MergedCommitPlan};

impl RelationalAuthoredCoupledCandidate<'_> {
    pub(super) fn aggregate_read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<AggregateCandidateRead>, TransactionCommitError> {
        let Some(aggregate) = &self.mutation.aggregate else {
            return Ok(None);
        };
        let prefix = self
            .transaction
            .prepared_coupled_commands
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prefix = prefix
            .as_ref()
            .expect("the ordinary merger carried the authored prefix");
        let plans = std::iter::once(prefix).chain(self.parts.iter().map(|part| &part.commands));
        execution.run_native_with_checkpoint(|context| {
            let mut count = 0usize;
            let mut bytes = 0u64;
            for plan in plans.clone() {
                for intent in &plan.merged_intents {
                    context.checkpoint(1).map_err(custody::execution_stop)?;
                    count = count.checked_add(1).ok_or_else(|| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
                    bytes = bytes.checked_add(intent.owned_allocation_capacity_bytes().saturating_add(std::mem::size_of_val(intent) as u64)).ok_or_else(|| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
                }
            }
            let _memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(crate::authority::commit::pipeline::request_lease(execution), bytes).map_err(|cause| custody::execution_stop(crate::execution::denial::lease_denial(cause)))?;
            let mut commands = MergedCommitPlan { transaction_id: prefix.transaction_id, merged_intents: Vec::new() };
            commands.merged_intents.try_reserve_exact(count).map_err(|_| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::AllocationFailed))?;
            for plan in plans {
                for intent in &plan.merged_intents {
                    context.checkpoint(intent.owned_allocation_capacity_bytes().saturating_add(1)).map_err(custody::execution_stop)?;
                    commands.merged_intents.push(intent.clone());
                }
            }
            aggregate.borrow_retained(self.mutation.state()).candidate_read(execution, &commands).map(Some)
        }).map_err(custody::execution_stop)?.0
    }
}
