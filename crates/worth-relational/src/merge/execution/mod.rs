#[cfg(test)]
use worth_relational_merge::internal_port::merge::data::prepared_execution::test_support::PreparedMergeExecutionFixturePort as _;
use worth_relational_merge::internal_port::merge::data::prepared_execution::PreparedMergeExecutionOwnerPort as _;
pub(crate) use worth_relational_merge::internal_port::merge::execution::plan_compilation;
mod authority_verification;

#[cfg(test)]
use crate::merge::data::MergeExecutionRequest;
use crate::merge::data::{
    CompiledMergeExecution, ExecutionReadyLoweredMergePlan, MergeExecutionError,
    MergeExecutionPreparationError, OwnerBoundMergeExecutionRequest, PreparedMergeExecution,
};

use super::planning_artifact::selected_basis::materialize_planning_artifact;
use super::MergeAccess;

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn derive_merge_commit_mutation_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        compiled: &CompiledMergeExecution,
    ) -> Result<super::data::PreparedMergeMutationPlan, super::data::MergeExecutionMutationPlanError>
    {
        let runtime = self.runtime;
        super::execution_mutation_plan::derive_merge_commit_mutation_plan(
            &runtime.schema_contract_runtime.aspect_contract_plans,
            &mut |entity| {
                let current = runtime.storage_access().current_edition();
                super::execution_mutation_plan::current_entity_snapshot(
                    &current,
                    &runtime.services.symbols,
                    &runtime.config.schema.registry,
                    execution,
                    entity,
                )
            },
            compiled,
        )
    }

    #[cfg(test)]
    pub fn prepare_merge_execution(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergeExecutionRequest,
    ) -> Result<PreparedMergeExecution, MergeExecutionPreparationError> {
        let source_branch = request.source_branch().clone();
        let target_branch = request.target_branch().clone();
        let bound = self
            .runtime
            .bind_merge_execution_request(request)
            .map_err(|denial| match denial {
                crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(branch)
                    if branch == source_branch =>
                {
                    MergeExecutionPreparationError::Planning(
                        crate::merge::data::MergePlanningError::MissingSourceHead {
                            branch_id: branch,
                        },
                    )
                }
                crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(branch)
                    if branch == target_branch =>
                {
                    MergeExecutionPreparationError::Planning(
                        crate::merge::data::MergePlanningError::MissingTargetHead {
                            branch_id: branch,
                        },
                    )
                }
                other => MergeExecutionPreparationError::Planning(
                    crate::merge::data::MergePlanningError::RequestNormalization(
                        crate::merge::data::RelationalMergeRequestNormalizationDenial::OwnerBinding(
                            other,
                        ),
                    ),
                ),
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
        let inputs = super::request_normalization::normalize_bound_merge_execution_request(request)
            .map_err(crate::merge::data::MergePlanningError::from)
            .map_err(MergeExecutionPreparationError::Planning)?;
        let normalized_request = inputs.request.clone();
        let lowered_plan = self
            .lower_planning_scope(execution, &inputs)
            .map_err(MergeExecutionPreparationError::Planning)?;
        let artifact = materialize_planning_artifact(execution, self.runtime, lowered_plan.clone())
            .map_err(MergeExecutionPreparationError::Planning)?;
        let execution_ready_plan = ExecutionReadyLoweredMergePlan::try_from_lowered(
            lowered_plan,
            artifact.schema_snapshot.clone(),
        )
        .map_err(MergeExecutionPreparationError::NotExecutionReady)?;
        let bound_executable_plan =
            compile_bound_executable_plan(self.runtime, &normalized_request, &execution_ready_plan)
                .map_err(MergeExecutionPreparationError::Compilation)?;

        let compiled = CompiledMergeExecution::new(
            normalized_request,
            artifact,
            execution_ready_plan,
            bound_executable_plan,
        );
        let mutation_plan = self
            .derive_merge_commit_mutation_plan(execution, &compiled)
            .map_err(MergeExecutionPreparationError::MutationPlan)?;
        Ok(PreparedMergeExecution::from_compiled(
            compiled,
            mutation_plan,
            (inputs.target, inputs.source),
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
        compile_bound_executable_plan(
            self.runtime,
            &crate::merge::data::NormalizedRelationalMergeRequest::admit_full_branch(
                execution_ready.basis.owner_target_head().branch_id.clone(),
                execution_ready.basis.owner_source_head().branch_id.clone(),
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &mut PreparedMergeExecution,
        bound_executable_plan: crate::merge::data::BoundExecutableMergePlan,
    ) -> Result<(), crate::merge::data::MergeExecutionMutationPlanError> {
        let compiled = CompiledMergeExecution::new(
            prepared.request().clone(),
            prepared.artifact().clone(),
            prepared.execution_ready_plan().clone(),
            bound_executable_plan.clone(),
        );
        let mutation_plan = self.derive_merge_commit_mutation_plan(execution, &compiled)?;
        prepared
            .replace_bound_plan_and_mutation_plan_for_test(bound_executable_plan, mutation_plan);
        Ok(())
    }
}

fn compile_bound_executable_plan(
    runtime: &crate::runtime::RelationalRuntime,
    request: &super::data::NormalizedRelationalMergeRequest,
    execution_ready: &ExecutionReadyLoweredMergePlan,
) -> Result<super::data::BoundExecutableMergePlan, super::data::MergeExecutionCompilationError> {
    use super::data::{MergeExecutionCompilationError, RuntimeInstanceId};
    let records = plan_compilation::compile_merge_records(
        &runtime.schema_contract_runtime.aspect_contract_plans,
        request,
        execution_ready,
    )?;
    let (target_reference, target_truth_version) = runtime
        .history()
        .branch_reference_state(request.target_branch())
        .ok_or(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch {
                detail: "target branch is unavailable during merge compilation",
            },
        )?;
    let (source_reference, source_truth_version) = runtime
        .history()
        .branch_reference_state(request.source_branch())
        .ok_or(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch {
                detail: "source branch is unavailable during merge compilation",
            },
        )?;

    Ok(records.bind(
        request,
        execution_ready,
        RuntimeInstanceId(runtime.runtime_instance_id()),
        (target_reference, target_truth_version),
        (source_reference, source_truth_version),
    ))
}

#[cfg(test)]
use worth_relational_contracts::internal_port::history::data::merge_branch_basis::RelationalMergeBranchBasisFieldsPort as _;
