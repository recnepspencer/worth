//! The ordinary apply phase consumes the exact applied candidate once.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::AuthoredRetirementRefusal as Cause;

impl BoundAuthoredCoupling {
    pub(crate) fn adopt_applied(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        identity: &RelationalMutationProposalIdentity,
        commands: &crate::transactions::data::MergedCommitPlan,
        symbols: &mut crate::runtime::PendingSymbolMeanings,
    ) -> Result<
        crate::authority::commit::phases::applied_mutation::AppliedCoreCustody,
        crate::authority::mutation::MutationApplyDenial,
    > {
        let candidate = self
            .applied
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .ok_or_else(|| {
                part::reject(Cause::ForeignCoupledOwner, self.owners.source.subject())
            })?;
        let CoupledAppliedCandidate {
            identity: original_identity,
            commands: original_commands,
            mutation,
        } = candidate;
        let mut candidate =
            crate::authority::commit::phases::applied_mutation::AppliedCoreCustody::coupled(
                Arc::clone(self),
                original_identity,
                original_commands,
                mutation,
            );
        execution
            .run_native_with_checkpoint(|context| {
                for intent in commands
                    .merged_intents
                    .iter()
                    .chain(&candidate.commands().merged_intents)
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
        if !candidate.matches(identity, commands) {
            return Err(
                part::reject(Cause::ForeignCoupledOwner, self.owners.source.subject()).into(),
            );
        }
        std::mem::swap(symbols, &mut candidate.identities.symbols);
        Ok(candidate)
    }
}

impl BoundAuthoredCoupling {
    /// Moves the same uncommitted state back; neither reconstructs nor reapplies it.
    pub(crate) fn return_applied(
        &self,
        identity: RelationalMutationProposalIdentity,
        commands: crate::mvcc::OwnedNormalizedPlan,
        mutation: AppliedMutationCore,
    ) {
        let mut slot = self
            .applied
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            slot.is_none(),
            "one original core returns to its existing owner"
        );
        *slot = Some(CoupledAppliedCandidate {
            identity,
            commands,
            mutation,
        });
    }
}
