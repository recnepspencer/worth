mod contexts;
mod decisions;
mod resolution;
#[cfg(test)]
mod tests;
mod value_strategy;

use crate::merge::data::{CausallyAnnotatedMergePlan, MergePlanningError, PolicyResolvedMergePlan};
use crate::merge::MergeAccess;
use crate::runtime::RelationalRuntime;

pub(crate) use decisions::current_topology_rewire_admission_policy;

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
        resolution::resolve_policy_scope(execution, self.runtime, causal_plan)
    }
}

fn binding_matches_aspect(
    runtime: &RelationalRuntime,
    binding: &crate::schema::data::LoweredAspectContractBinding,
    aspect_key: &worth_foundational::facade::AspectKey,
) -> bool {
    aspect_key_equivalent(runtime, binding.aspect_key(), aspect_key)
}

fn aspect_key_equivalent(
    _runtime: &RelationalRuntime,
    left: &worth_foundational::facade::AspectKey,
    right: &worth_foundational::facade::AspectKey,
) -> bool {
    left == right
}
