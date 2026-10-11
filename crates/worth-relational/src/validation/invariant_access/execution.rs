use crate::branch::SelectedRelationalBranchState;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::{InvariantGroupSet, InvariantPlanContract};
use crate::validation::engine::{
    InvariantEngine, InvariantExecutionDisposition, InvariantExecutionRequest,
    InvariantExecutionResult, InvariantObservation, InvariantRequestProfile,
};
use crate::validation::invariant_access::InvariantAccess;

impl<'runtime> InvariantAccess<'runtime> {
    pub(super) fn execute_for_runtime(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        profile: InvariantRequestProfile,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )?;
        self.execute_for_state(
            execution,
            profile,
            InvariantObservation::committed(self.runtime.storage_access().current_edition()),
            self.runtime.current_version_id(),
            None,
        )
    }

    pub(super) fn execute_for_selected_branch_committed_plan<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        profile: InvariantRequestProfile,
        selected_state: &'state SelectedRelationalBranchState,
        merged_plan: &'state MergedCommitPlan,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        let version_id = selected_state.version_id();
        self.execute_for_state_with_current_version(
            execution,
            profile,
            InvariantObservation::committed_branch(selected_state.state()),
            version_id,
            version_id,
            Some(merged_plan),
        )
    }

    pub(super) fn execute_for_state<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        profile: InvariantRequestProfile,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        self.execute_for_state_with_current_version(
            execution,
            profile,
            observation,
            version_id,
            self.runtime.current_version_id(),
            merged_plan,
        )
    }

    fn execute_for_state_with_current_version<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        profile: InvariantRequestProfile,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        let plan_contract = merged_plan.map(InvariantPlanContract::from_merged_plan);
        let consumed_groups = profile.consumed_groups();
        let observation_kind = observation.kind();
        let proposal_identity = observation.proposal_identity().cloned();
        if plan_contract
            .is_some_and(|contract| !contract.intersects_consumed_groups(consumed_groups))
        {
            return Ok(InvariantExecutionResult::skipped(self.execution_metadata(
                profile,
                observation_kind,
                version_id,
                current_version_id,
                merged_plan,
                plan_contract,
                InvariantGroupSet::empty(),
                crate::validation::data::InvariantCostClass::Global,
                InvariantExecutionDisposition::SkippedByPlanContract,
                proposal_identity.as_ref(),
            )));
        }

        let request = InvariantExecutionRequest::from_profile_with_contract_at_current_version(
            profile,
            &self.view,
            observation,
            version_id,
            current_version_id,
            merged_plan,
            plan_contract,
            &crate::validation::engine::InvariantPreparationControl::new(
                crate::mvcc::RelationalOperationControl::uninterrupted(),
                execution.allocation_policy(),
                crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
            ),
        )?;
        if !request.should_execute_anything() {
            return Ok(InvariantExecutionResult::skipped(self.execution_metadata(
                profile,
                observation_kind,
                version_id,
                current_version_id,
                merged_plan,
                plan_contract,
                request.applicable_groups(),
                request.max_cost(),
                InvariantExecutionDisposition::SkippedByMayBreakMask,
                request.proposal_identity(),
            )));
        }
        InvariantEngine::from_view(&self.view).execute(execution, request)
    }
}
