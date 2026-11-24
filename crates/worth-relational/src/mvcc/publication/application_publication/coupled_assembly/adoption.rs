//! The ordinary apply phase consumes the exact applied candidate once.
use super::{
    completion::CoupledAppliedCandidate, refusals, retained_assembly::BoundAuthoredCoupling,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;
use crate::execution::request_stage::checkpoint;
use crate::mvcc::publication::application_publication::custody;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;

impl BoundAuthoredCoupling {
    pub(crate) fn adopt_applied(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        identity: &RelationalMutationProposalIdentity,
        commands: &crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
        symbols: &mut crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
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
                refusals::reject(Cause::ForeignCoupledOwner, self.owners.source.subject())
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
                refusals::reject(Cause::ForeignCoupledOwner, self.owners.source.subject()).into(),
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
        commands: crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
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

impl BoundAuthoredCoupling {
    pub(crate) fn inherit_prepared_symbols(
        &self,
        symbols: &mut crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
    ) -> Result<(), crate::authority::mutation::MutationApplyDenial> {
        let prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_ref().ok_or_else(|| {
            refusals::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        symbols.adopt(prepared.mutation.candidate_symbols())?;
        Ok(())
    }
    pub(crate) fn adopt_prepared_symbols(
        &self,
        identity: &RelationalMutationProposalIdentity,
        symbols: &mut crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
    ) -> Result<(), TransactionCommitError> {
        let mut prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_mut().ok_or_else(|| {
            refusals::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        if !prepared.identity.same_proposal_sequence(identity) {
            return Err(refusals::reject(
                crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                self.owners.source.subject(),
            ));
        }
        std::mem::swap(symbols, &mut prepared.mutation.identities.symbols);
        Ok(())
    }
}
