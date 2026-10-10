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
        profile: InvariantRequestProfile,
    ) -> InvariantExecutionResult {
        self.execute_for_state(
            profile,
            InvariantObservation::committed(self.runtime.storage_access().current_edition()),
            self.runtime.current_version_id(),
            None,
        )
    }

    pub(super) fn execute_for_selected_branch_committed_plan<'state>(
        &self,
        profile: InvariantRequestProfile,
        selected_state: &'state SelectedRelationalBranchState,
        merged_plan: &'state MergedCommitPlan,
    ) -> InvariantExecutionResult
    where
        'runtime: 'state,
    {
        let version_id = selected_state.version_id();
        self.execute_for_state_with_current_version(
            profile,
            InvariantObservation::committed_branch(selected_state.state()),
            version_id,
            version_id,
            Some(merged_plan),
        )
    }

    pub(super) fn execute_for_state<'state>(
        &self,
        profile: InvariantRequestProfile,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
    ) -> InvariantExecutionResult
    where
        'runtime: 'state,
    {
        self.execute_for_state_with_current_version(
            profile,
            observation,
            version_id,
            self.runtime.current_version_id(),
            merged_plan,
        )
    }

    fn execute_for_state_with_current_version<'state>(
        &self,
        profile: InvariantRequestProfile,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
    ) -> InvariantExecutionResult
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
            return InvariantExecutionResult::skipped(self.execution_metadata(
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
            ));
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
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
            ),
        )
        .expect("uninterrupted System scope preparation");
        if !request.should_execute_anything() {
            return InvariantExecutionResult::skipped(self.execution_metadata(
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
            ));
        }
        InvariantEngine::from_view(&self.view).execute(request)
    }
}
