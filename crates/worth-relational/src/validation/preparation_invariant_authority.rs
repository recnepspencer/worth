use crate::authority::commit::semantic_plan::RelationalSemanticPlan;
use crate::branch::SelectedRelationalBranchRead;
use crate::runtime::{RelationalPreparationRuntime, WorkingState};
use crate::transactions::data::TransactionCommitError;
use crate::validation::data::{InvariantGroupSet, InvariantPlanContract};
use crate::validation::engine::{
    InvariantEngine, InvariantExecutionDisposition, InvariantExecutionMetadata,
    InvariantExecutionRequest, InvariantExecutionResult, InvariantObservation,
    InvariantRequestProfile, InvariantRuntimeView,
};

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
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    ) -> Result<crate::validation::engine::PendingProposedPass, TransactionCommitError> {
        let view = InvariantRuntimeView::from_preparation_for_state(
            self.runtime,
            selected.selected().state(),
        );
        let aggregate = aggregate.map(|source| source.invocation(plan.commands()));
        let observation = InvariantObservation::committed_branch_with_proposed(
            selected,
            working,
            version,
            proposal.cloned(),
        )
        .with_aggregate(aggregate);
        let request = InvariantExecutionRequest::from_semantic_plan(
            execution,
            InvariantRequestProfile::CommitBoundary,
            &view,
            observation,
            selected.version_id(),
            selected.version_id(),
            Some(plan),
            Some(plan.invariant_contract()),
            control,
        )?;
        let started = InvariantEngine::from_view(&view).begin_proposed(execution, request);
        let started = match aggregate {
            Some(source) => source.settle(started, execution, control),
            None => started,
        }?;
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
        created_entities: &crate::transactions::data::CommitCreatedEntityBindings,
        version: crate::identity::data::VersionId,
        plan: RelationalSemanticPlan<'_>,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let result = self.execute(
            InvariantRequestProfile::ProposedState,
            &selected,
            working,
            pending,
            Some(created_entities),
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
        let enforced = |effect| {
            result.results().iter().find_map(|row| {
                if row.failure_effect != effect || !row.blocks_proposed_state() {
                    return None;
                }
                let crate::validation::data::InvariantVerdict::Violation(violation) = &row.verdict
                else {
                    return None;
                };
                Some(crate::validation::engine::InvariantFailure::new(
                    row.execution_point,
                    effect,
                    violation.clone(),
                ))
            })
        };
        if let Some(failure) =
            enforced(crate::validation::data::InvariantFailureEffect::BlockCommit)
        {
            crate::validation::invariant_authority::preparation_diagnostics::emit_preparation_diagnostics(self.runtime, &result);
            if !crate::validation::invariant_authority::failure_diagnostics::emit_collect_all_failure_diagnostics(self.runtime, &result) {
                crate::validation::invariant_authority::failure_diagnostics::emit_conflict_diagnostic(self.runtime, &result, &failure);
            }
            return Err(TransactionCommitError::conflict(
                failure.clone().into_commit_conflict(),
            ));
        }
        match enforced(crate::validation::data::InvariantFailureEffect::BlockPublication) {
            Some(failure) => {
                crate::validation::invariant_authority::preparation_diagnostics::emit_preparation_diagnostics(
                    self.runtime,
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
                    failure.clone().into_publication_error(
                        crate::publication::bundle::PublicationStage::InvariantCheck,
                    ),
                ))
            }
            None => Ok(result),
        }
    }

    fn execute(
        &self,
        profile: InvariantRequestProfile,
        selected: &SelectedRelationalBranchRead<'_>,
        working: &WorkingState,
        pending: crate::validation::engine::PendingProposedPass,
        applied_created_entities: Option<&crate::transactions::data::CommitCreatedEntityBindings>,
        version: crate::identity::data::VersionId,
        plan: RelationalSemanticPlan<'_>,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        control.check()?;
        let execution_version = match profile {
            InvariantRequestProfile::CommitBoundary => selected.version_id(),
            _ => version,
        };
        let observation = match profile {
            InvariantRequestProfile::ProposedState => {
                InvariantObservation::committed_branch_with_proposed(
                    selected,
                    working,
                    version,
                    proposal.cloned(),
                )
                .with_applied_created_entities(
                    applied_created_entities
                        .expect("applied proposal carries its created bindings"),
                )
            }
            InvariantRequestProfile::MutationSensitive
            | InvariantRequestProfile::SnapshotPublication => {
                let overlay = crate::storage::overlay::OverlayStateView::new(selected, working);
                match applied_created_entities {
                    Some(created_entities) => InvariantObservation::applied_with_proposal(
                        overlay,
                        selected.version_id(),
                        proposal.cloned(),
                        created_entities,
                    ),
                    None => InvariantObservation::speculative_with_proposal(
                        overlay,
                        selected.version_id(),
                        proposal.cloned(),
                    ),
                }
            }
            _ => InvariantObservation::committed_branch_with_proposed(
                selected,
                working,
                version,
                proposal.cloned(),
            ),
        };
        // Aggregate logical changes were captured from the actual applied
        // effect. Its command constituent is used only for native consequences.
        let aggregate = aggregate.map(|source| source.invocation(plan.commands()));
        let observation = observation.with_aggregate(aggregate);
        let observation_kind = observation.kind();
        let view = InvariantRuntimeView::from_preparation_for_state(
            self.runtime,
            selected.selected().state(),
        );
        let plan_contract = Some(plan.invariant_contract());
        let consumed_groups = profile.consumed_groups();
        if plan_contract
            .is_some_and(|contract| !contract.intersects_consumed_groups(consumed_groups))
        {
            return Ok(InvariantExecutionResult::skipped(self.metadata(
                profile,
                observation_kind,
                execution_version,
                plan_contract,
                InvariantGroupSet::empty(),
                crate::validation::data::InvariantCostClass::Global,
                InvariantExecutionDisposition::SkippedByPlanContract,
                proposal,
            )));
        }
        let request = InvariantExecutionRequest::from_semantic_plan(
            execution,
            profile,
            &view,
            observation,
            execution_version,
            selected.version_id(),
            Some(plan),
            plan_contract,
            control,
        )?;
        if !request.should_execute_anything() {
            return Ok(InvariantExecutionResult::skipped(self.metadata(
                profile,
                observation_kind,
                execution_version,
                plan_contract,
                request.applicable_groups(),
                request.max_cost(),
                InvariantExecutionDisposition::SkippedByMayBreakMask,
                request.proposal_identity(),
            )));
        }
        let result =
            InvariantEngine::from_view(&view).complete_proposed(execution, request, pending);
        match aggregate {
            Some(source) => source.settle(result, execution, control),
            None => result,
        }
    }

    fn metadata(
        &self,
        profile: InvariantRequestProfile,
        observation: crate::validation::engine::InvariantObservationKind,
        version: crate::identity::data::VersionId,
        contract: Option<InvariantPlanContract>,
        groups: InvariantGroupSet,
        cost: crate::validation::data::InvariantCostClass,
        disposition: InvariantExecutionDisposition,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> InvariantExecutionMetadata {
        InvariantExecutionMetadata::new(
            profile.execution_point(),
            observation,
            version,
            version,
            profile.consumed_groups(),
            groups,
            cost,
            disposition,
            contract,
            true,
            None,
            Vec::new(),
            None,
            proposal.cloned(),
        )
    }
}
