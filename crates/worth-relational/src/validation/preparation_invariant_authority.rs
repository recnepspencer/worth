use crate::branch::SelectedRelationalBranchState;
use crate::runtime::{RelationalPreparationRuntime, WorkingState};
use crate::transactions::data::{MergedCommitPlan, TransactionCommitError};
use crate::validation::data::{
    InvariantFailureEffect, InvariantGroup, InvariantGroupSet, InvariantPlanContract,
    InvariantVerdict,
};
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
    pub(crate) fn enforce_commit_boundary_for_selected_branch(
        &self,
        selected: &SelectedRelationalBranchState,
        proposed: &WorkingState,
        version: crate::identity::data::VersionId,
        plan: &MergedCommitPlan,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let result = self.execute(
            InvariantRequestProfile::CommitBoundary,
            selected,
            proposed,
            version,
            plan,
            proposal,
            lease,
        )?;
        match result.summary().blocking_failure() {
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
                    crate::validation::invariant_authority::failure_diagnostics::emit_conflict_diagnostic(
                        self.runtime,
                        &result,
                        failure,
                    );
                }
                Err(TransactionCommitError::conflict(
                    failure.clone().into_commit_conflict(),
                ))
            }
            None => Ok(result),
        }
    }

    pub(crate) fn enforce_mutation_sensitive_for_working_state(
        &self,
        selected: &SelectedRelationalBranchState,
        working: &WorkingState,
        version: crate::identity::data::VersionId,
        plan: &MergedCommitPlan,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let result = self.execute(
            InvariantRequestProfile::MutationSensitive,
            selected,
            working,
            version,
            plan,
            proposal,
            lease,
        )?;
        match result.summary().blocking_failure() {
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
                    crate::validation::invariant_authority::failure_diagnostics::emit_conflict_diagnostic(
                        self.runtime,
                        &result,
                        failure,
                    );
                }
                Err(TransactionCommitError::conflict(
                    failure.clone().into_commit_conflict(),
                ))
            }
            None => Ok(result),
        }
    }

    pub(crate) fn enforce_snapshot_publication_for_working_state(
        &self,
        selected: &SelectedRelationalBranchState,
        working: &WorkingState,
        version: crate::identity::data::VersionId,
        plan: &MergedCommitPlan,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let result = self.execute(
            InvariantRequestProfile::SnapshotPublication,
            selected,
            working,
            version,
            plan,
            proposal,
            lease,
        )?;
        match result.summary().publication_failure() {
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
                        failure,
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
        selected: &SelectedRelationalBranchState,
        working: &WorkingState,
        version: crate::identity::data::VersionId,
        plan: &MergedCommitPlan,
        proposal: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        let execution_version = match profile {
            InvariantRequestProfile::CommitBoundary => selected.version_id(),
            _ => version,
        };
        let observation = match profile {
            InvariantRequestProfile::MutationSensitive
            | InvariantRequestProfile::SnapshotPublication => {
                InvariantObservation::speculative_with_proposal(
                    crate::storage::overlay::OverlayStateView::new(selected.state(), working),
                    selected.version_id(),
                    proposal.cloned(),
                )
            }
            _ => InvariantObservation::committed_branch_with_proposed(
                selected.state(),
                working,
                version,
                proposal.cloned(),
            ),
        };
        let observation_kind = observation.kind();
        let view = InvariantRuntimeView::from_preparation_for_state(self.runtime, selected.state());
        let plan_contract = Some(InvariantPlanContract::from_merged_plan(plan));
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
        let request = InvariantExecutionRequest::from_profile_with_contract_at_current_version(
            profile,
            &view,
            observation,
            execution_version,
            selected.version_id(),
            Some(plan),
            plan_contract,
        );
        if let Some(violation) = request.preparation_violation().cloned() {
            return Ok(InvariantExecutionResult::executed(
                self.metadata(
                    profile,
                    observation_kind,
                    execution_version,
                    plan_contract,
                    request.applicable_groups(),
                    request.max_cost(),
                    InvariantExecutionDisposition::Executed,
                    request.proposal_identity(),
                ),
                vec![crate::validation::data::InvariantCheckResult {
                    execution_point: profile.execution_point(),
                    failure_effect: InvariantFailureEffect::BlockCommit,
                    rule: crate::validation::data::InvariantReportedRule::Native(
                        crate::validation::data::InvariantRule::RelationIntegrityScopeBudget(
                            self.runtime
                                .config
                                .execution
                                .relation_integrity_scope_budget
                                .max_planned_edges,
                        ),
                    ),
                    groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity)
                        .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
                    witness: violation.witness_key(),
                    cost: crate::validation::data::InvariantCostClass::Touched,
                    custom_provenance: None,
                    verdict: InvariantVerdict::Violation(violation),
                }],
            ));
        }
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
        match lease {
            Some(lease) => InvariantEngine::from_view(&view).execute_with_lease(request, lease),
            None => Ok(InvariantEngine::from_view(&view).execute(request)),
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
