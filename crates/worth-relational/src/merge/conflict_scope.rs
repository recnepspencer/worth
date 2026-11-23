use super::MergeAccess;
use crate::merge::data::{
    ConflictClassifiedMergePlan, IdentityScopedMergePlan, MergePlanningError,
};
impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn plan_conflict_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<ConflictClassifiedMergePlan, MergePlanningError> {
        let identity_plan = self.plan_identity_scope(execution, request)?;
        self.classify_conflict_scope(execution, identity_plan)
    }

    fn classify_conflict_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity_plan: IdentityScopedMergePlan,
    ) -> Result<ConflictClassifiedMergePlan, MergePlanningError> {
        let target_view = self
            .runtime
            .read_truth()
            .read_version(execution, identity_plan.basis.target_head.version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;
        let history = self.runtime.history();
        let base_commit = history
            .immutable_commit_receipt(identity_plan.basis.merge_base.commit.commit_id)
            .ok_or(MergePlanningError::MissingMergeBaseCommit {
                commit_id: identity_plan.basis.merge_base.commit.commit_id,
            })?;
        let base_version_id = base_commit.version_id;
        let base_view = self
            .runtime
            .read_truth()
            .read_version(execution, base_version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;

        let (plan, topology) = super::conflicts::classify_conflict_scope(
            &self.runtime.schema_contract_runtime.aspect_contract_plans,
            identity_plan,
            base_version_id,
            &base_view,
            &target_view,
        )?;
        if topology.relation_candidates != 0 {
            self.runtime
                .performance_access()
                .count_merge_topology_region_detection(
                    topology.relation_candidates,
                    topology.endpoint_incidences,
                    topology.region_conflicts,
                    topology.escalated_records,
                );
        }
        Ok(plan)
    }
}
