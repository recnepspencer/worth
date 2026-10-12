mod aspect_plan_compilation;
mod authority_verification;
mod lineage_continuity;
mod plan_compilation;
mod prepared_execution;
mod record_plan_compilation;

pub(super) use prepared_execution::execute_prepared_merge;

#[cfg(test)]
use crate::merge::data::MergeExecutionRequest;
use crate::merge::data::{
    CompiledMergeExecution, ExecutionReadyLoweredMergePlan, MergeExecutionError,
    MergeExecutionPreparationError, OwnerBoundMergeExecutionRequest, PreparedMergeExecution,
};

use super::planning_artifact::materialize_planning_artifact;
use super::MergeAccess;

impl<'runtime> MergeAccess<'runtime> {
    #[cfg(test)]
    pub fn prepare_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergeExecutionRequest,
    ) -> Result<PreparedMergeExecution, MergeExecutionPreparationError> {
        let source_branch = request.source_branch().clone();
        let target_branch = request.target_branch().clone();
        let bound =
            self.runtime
                .bind_merge_execution_request(request)
                .map_err(|denial| match denial {
                    crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(
                        branch,
                    ) if branch == source_branch => MergeExecutionPreparationError::Planning(
                        crate::merge::data::MergePlanningError::MissingSourceHead {
                            branch_id: branch,
                        },
                    ),
                    crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(
                        branch,
                    ) if branch == target_branch => MergeExecutionPreparationError::Planning(
                        crate::merge::data::MergePlanningError::MissingTargetHead {
                            branch_id: branch,
                        },
                    ),
                    other => MergeExecutionPreparationError::OwnerBinding(other),
                })?;
        self.prepare_bound_merge_execution(execution, bound)
    }

    #[cfg(not(test))]
    pub fn prepare_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: OwnerBoundMergeExecutionRequest,
    ) -> Result<PreparedMergeExecution, MergeExecutionPreparationError> {
        self.prepare_bound_merge_execution(execution, request)
    }

    fn prepare_bound_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: OwnerBoundMergeExecutionRequest,
    ) -> Result<PreparedMergeExecution, MergeExecutionPreparationError> {
        let normalized_request =
            super::request_normalization::normalize_bound_merge_execution_request(request)
                .map_err(crate::merge::data::MergePlanningError::from)
                .map_err(MergeExecutionPreparationError::Planning)?;
        let lowered_plan = self
            .lower_planning_scope(execution, normalized_request.clone())
            .map_err(MergeExecutionPreparationError::Planning)?;
        let artifact = materialize_planning_artifact(execution, self.runtime, lowered_plan.clone());
        let execution_ready_plan = ExecutionReadyLoweredMergePlan::try_from_lowered(
            lowered_plan,
            artifact.schema_snapshot.clone(),
        )
        .map_err(MergeExecutionPreparationError::NotExecutionReady)?;
        let bound_executable_plan = plan_compilation::compile_bound_executable_plan(
            self.runtime,
            &normalized_request,
            &execution_ready_plan,
        )
        .map_err(MergeExecutionPreparationError::Compilation)?;

        let compiled = CompiledMergeExecution::new(
            normalized_request,
            artifact,
            execution_ready_plan,
            bound_executable_plan,
        );
        let mutation_plan = self
            .derive_merge_commit_mutation_plan(&compiled)
            .map_err(MergeExecutionPreparationError::MutationPlan)?;
        Ok(PreparedMergeExecution::from_compiled(
            compiled,
            mutation_plan,
        ))
    }

    pub fn verify_prepared_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &PreparedMergeExecution,
    ) -> Result<(), MergeExecutionError> {
        authority_verification::verify_prepared_merge_execution(execution, self.runtime, prepared)
    }

    #[cfg(test)]
    pub(crate) fn compile_execution_ready_merge_plan_for_test(
        &self,
        execution_ready: &ExecutionReadyLoweredMergePlan,
    ) -> Result<
        crate::merge::data::BoundExecutableMergePlan,
        crate::merge::data::MergeExecutionCompilationError,
    > {
        plan_compilation::compile_bound_executable_plan(
            self.runtime,
            &crate::merge::data::NormalizedRelationalMergeRequest::admit_full_branch(
                execution_ready.basis.target_head.branch_id.clone(),
                execution_ready.basis.source_head.branch_id.clone(),
                crate::merge::data::MergeIntent::ReconcileIntoTarget,
                crate::merge::data::RelationalMergeCorrespondencePosture::Advisory,
                crate::merge::data::RelationalMergeSchemaReconciliationPosture::Participate,
                crate::merge::data::RelationalMergeTopologyIntent::PreserveTopologySemantics,
            )
            .expect("test compilation request should admit"),
            execution_ready,
        )
    }

    #[cfg(test)]
    pub(crate) fn replace_bound_merge_plan_for_test(
        &self,
        prepared: &mut PreparedMergeExecution,
        bound_executable_plan: crate::merge::data::BoundExecutableMergePlan,
    ) -> Result<(), crate::merge::data::MergeExecutionMutationPlanError> {
        let compiled = CompiledMergeExecution::new(
            prepared.request().clone(),
            prepared.artifact().clone(),
            prepared.execution_ready_plan().clone(),
            bound_executable_plan.clone(),
        );
        let mutation_plan = self.derive_merge_commit_mutation_plan(&compiled)?;
        prepared
            .replace_bound_plan_and_mutation_plan_for_test(bound_executable_plan, mutation_plan);
        Ok(())
    }
}
