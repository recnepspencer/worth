use super::proposed_state::enforced_failure;
use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::branch::selected_read::SelectedRelationalBranchRead;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::data::execution::InvariantFailureEffect;
use crate::validation::engine::InvariantExecutionResult;

pub(crate) struct PreparationInvariantAuthority<'a> {
    runtime: &'a RelationalPreparationRuntime,
}

impl RelationalPreparationRuntime {
    pub(crate) fn invariant_authority(&self) -> PreparationInvariantAuthority<'_> {
        PreparationInvariantAuthority { runtime: self }
    }
}

impl PreparationInvariantAuthority<'_> {
    pub(crate) fn begin_proposed_state(
        &self,
        selected: &SelectedRelationalBranchRead<'_>,
        working: &WorkingState,
        version: crate::identity::data::VersionId,
        plan: RelationalSemanticPlan<'_>,
        proposal: Option<
            &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        >,
        aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
        prepare_sources: &mut dyn FnMut() -> Result<(), TransactionCommitError>,
    ) -> Result<crate::validation::engine::PendingProposedPass, TransactionCommitError> {
        let view = self
            .runtime
            .invariant_validation_view(selected.selected().state());
        let started = super::proposed_state::begin_proposed_state(
            &view,
            selected,
            working,
            version,
            plan,
            proposal,
            aggregate,
            execution,
            control,
            prepare_sources,
        )?;
        match started {
            crate::validation::engine::StartedProposedPass::Pending(pending) => Ok(pending),
            crate::validation::engine::StartedProposedPass::Refused(result) => {
                self.admit_result(result)?;
                unreachable!("the engine only finishes a refused native prefix")
            }
        }
    }

    pub(crate) fn enforce_proposed_state(
        &self,
        selected: &SelectedRelationalBranchRead<'_>,
        working: &WorkingState,
        pending: crate::validation::engine::PendingProposedPass,
        created_entities: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        version: crate::identity::data::VersionId,
        plan: RelationalSemanticPlan<'_>,
        proposal: Option<
            &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        >,
        aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        control.check()?;
        let view = self
            .runtime
            .invariant_validation_view(selected.selected().state());
        let result = super::proposed_state::complete_proposed_state(
            &view,
            selected,
            working,
            pending,
            created_entities,
            version,
            plan,
            proposal,
            aggregate,
            execution,
            control,
        )?;
        self.admit_result(result)
    }

    fn admit_result(
        &self,
        result: InvariantExecutionResult,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        if let Some(failure) = enforced_failure(&result, InvariantFailureEffect::BlockCommit) {
            crate::validation::invariant_authority::preparation_diagnostics::emit_preparation_diagnostics(self.runtime.preparation_diagnostics(), &result);
            if !crate::validation::invariant_authority::failure_diagnostics::emit_collect_all_failure_diagnostics(self.runtime, &result) {
                crate::validation::invariant_authority::failure_diagnostics::emit_conflict_diagnostic(self.runtime, &result, &failure);
            }
            return Err(TransactionCommitError::conflict(
                failure.into_commit_conflict(),
            ));
        }
        match enforced_failure(&result, InvariantFailureEffect::BlockPublication) {
            Some(failure) => {
                crate::validation::invariant_authority::preparation_diagnostics::emit_preparation_diagnostics(
                    self.runtime.preparation_diagnostics(),
                    &result,
                );
                let collect_all = crate::validation::invariant_authority::failure_diagnostics::emit_collect_all_failure_diagnostics(
                    self.runtime,
                    &result,
                );
                if !collect_all {
                    crate::validation::invariant_authority::failure_diagnostics::emit_publication_failure(
                        self.runtime,
                        &result,
                        &failure,
                    );
                }
                Err(TransactionCommitError::publication(
                    failure.into_publication_error(
                        crate::publication::bundle::publication_status::PublicationStage::InvariantCheck,
                    ),
                ))
            }
            None => Ok(result),
        }
    }
}
