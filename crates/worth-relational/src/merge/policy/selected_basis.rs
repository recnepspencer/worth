use crate::merge::data::{CausallyAnnotatedMergePlan, MergePlanningError, PolicyResolvedMergePlan};
use crate::merge::MergeAccess;

use super::resolution::resolve_policy_scope;
use super::value_strategy::PolicyValueLookupCounters;

impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn plan_policy_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<PolicyResolvedMergePlan, MergePlanningError> {
        let causal_plan = self.plan_causal_scope(execution, request)?;
        self.resolve_policy_scope(execution, causal_plan)
    }

    fn resolve_policy_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        causal_plan: CausallyAnnotatedMergePlan,
    ) -> Result<PolicyResolvedMergePlan, MergePlanningError> {
        let runtime = self.runtime;
        let history = runtime.history();
        let base_envelope = history
            .demand_commit_envelope(execution, causal_plan.basis.merge_base().commit().commit_id)
            .map_err(MergePlanningError::Execution)?
            .ok_or(MergePlanningError::MissingMergeBaseCommit {
                commit_id: causal_plan.basis.merge_base().commit().commit_id,
            })?;
        let source_view = runtime
            .read_truth()
            .read_version(execution, causal_plan.basis.source_head().version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;
        let target_view = runtime
            .read_truth()
            .read_version(execution, causal_plan.basis.target_head().version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;
        let base_view = runtime
            .read_truth()
            .read_version(execution, base_envelope.commit.version_id)
            .map_err(crate::merge::data::MergePlanningError::Execution)?;
        let mut counters = PolicyValueLookupCounters::default();
        let outcome = resolve_policy_scope(
            &runtime.config.schema.registry,
            &runtime.schema_contract_runtime.aspect_contract_plans,
            &source_view,
            &target_view,
            &base_view,
            &base_envelope,
            &|source, target| history.max_commit_id_common_ancestor(source, target),
            &|commit_id| {
                history
                    .demand_commit_envelope(execution, commit_id)
                    .map_err(MergePlanningError::Execution)?
                    .ok_or(MergePlanningError::MissingMergeBaseCommit { commit_id })
            },
            &|version_id| {
                runtime
                    .read_truth()
                    .read_version(execution, version_id)
                    .map_err(MergePlanningError::Execution)
            },
            &mut counters,
            causal_plan,
        );
        if counters != PolicyValueLookupCounters::default() {
            runtime
                .performance_access()
                .count_merge_policy_value_lookup(
                    counters.source_state_hit,
                    counters.target_state_hit,
                    counters.base_state_hit,
                    counters.base_patch_authority_hit,
                    counters.missing_ancestor_basis,
                    counters.missing_visible_state,
                    counters.invalid_shape,
                );
        }
        outcome
    }
}
