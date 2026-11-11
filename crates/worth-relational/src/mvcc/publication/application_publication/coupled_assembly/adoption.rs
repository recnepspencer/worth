//! The ordinary apply phase consumes the exact applied candidate once.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::AuthoredRetirementRefusal as Cause;

impl BoundAuthoredCoupling {
    pub(crate) fn adopt_applied(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: &RelationalMutationProposalIdentity,
        commands: &crate::transactions::data::MergedCommitPlan,
        symbols: &mut crate::runtime::PendingSymbolMeanings,
    ) -> Result<AppliedMutationCore, crate::authority::mutation::MutationApplyDenial> {
        let mut candidate = self
            .applied
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .ok_or_else(|| part::reject(Cause::ForeignCoupledOwner, self.owners.anchor))?;
        execution
            .run_native_with_checkpoint(|context| {
                for intent in commands
                    .merged_intents
                    .iter()
                    .chain(&candidate.commands.plan().merged_intents)
                {
                    checkpoint(
                        context,
                        intent.owned_allocation_capacity_bytes().saturating_add(1),
                    )
                    .map_err(custody::execution_stop)?;
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0?;
        if !candidate.identity.same_proposal_sequence(identity)
            || candidate.commands.plan() != commands
        {
            return Err(part::reject(Cause::ForeignCoupledOwner, self.owners.anchor).into());
        }
        std::mem::swap(symbols, &mut candidate.mutation.identities.symbols);
        *self.completed.lock().unwrap_or_else(|e| e.into_inner()) =
            candidate.mutation.coupled_publication.take();
        Ok(candidate.mutation)
    }
}
