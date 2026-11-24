//! The ordinary apply phase consumes the exact applied candidate once.
use super::{
    completion::CoupledAppliedCandidate, refusals, retained_assembly::BoundAuthoredCoupling,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::{
    RelationalMutationProposalIdentity, RelationalMutationProposalIdentityOwnerPort as _,
};
use worth_relational_storage::internal_port as storage;

impl BoundAuthoredCoupling {
    pub(crate) fn adopt_applied(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        identity: &RelationalMutationProposalIdentity,
        commands: &contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
        symbols: &mut storage::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
    ) -> Result<
        crate::authority::commit::phases::applied_mutation::AppliedCoreCustody,
        worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial,
    >{
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
        commands: worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
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
    pub fn inherit_prepared_symbols(
        &self,
        symbols: &mut storage::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
    ) -> Result<(), worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial>{
        let prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_ref().ok_or_else(|| {
            refusals::reject(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        symbols.adopt(prepared.mutation.candidate_symbols())?;
        Ok(())
    }
    pub fn adopt_prepared_symbols(
        &self,
        identity: &RelationalMutationProposalIdentity,
        symbols: &mut storage::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
    ) -> Result<(), TransactionCommitError> {
        let mut prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_mut().ok_or_else(|| {
            refusals::reject(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        if !prepared.identity.same_proposal_sequence(identity) {
            return Err(refusals::reject(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::ForeignCoupledOwner,
                self.owners.source.subject(),
            ));
        }
        std::mem::swap(symbols, &mut prepared.mutation.identities.symbols);
        Ok(())
    }
}
