use worth_query::facade::{domain, read, runtime};

use super::executors::{
    execute_reconstructed_read_for_sabotage, installed_read_declaration,
    mismatched_read_declaration, CountVerticesExecutor, ReadVertexExecutor,
};
use super::{
    configured_runtime_without_executors, package, workspace, CountVertices, CountVerticesInput,
    GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

#[derive(Clone, Copy)]
struct ForeignRuntimeReadExecutor;

#[derive(Clone, Copy)]
struct MismatchedReadPlanExecutor;

#[derive(Clone, Copy)]
struct MissingReadExecutionExecutor;

#[derive(Clone, Copy)]
struct MismatchedCostExecutor;

#[derive(Clone, Copy)]
struct MismatchedDeterminismExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for MismatchedDeterminismExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = false;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        _: ReadExecutionInput,
        _: &domain::WorthQueryOperationExecutionContext<'_>,
        _: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        unreachable!("runtime construction must reject executor determinism drift")
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for MismatchedCostExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::GraphBreadth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        _: ReadExecutionInput,
        _: &domain::WorthQueryOperationExecutionContext<'_>,
        _: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        unreachable!("runtime construction must reject executor cost drift")
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, CountVertices, ReadFamily>
    for MissingReadExecutionExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        _: CountVerticesInput,
        _: &domain::WorthQueryOperationExecutionContext<'_>,
        _: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<u64>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            0,
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for MismatchedReadPlanExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(mismatched_read_declaration())
    }

    fn execute(
        &self,
        _: ReadExecutionInput,
        _: &domain::WorthQueryOperationExecutionContext<'_>,
        _: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        unreachable!("runtime construction must reject semantic plan drift")
    }
}

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for ForeignRuntimeReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        input: ReadExecutionInput,
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        owner_workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        let context_execution = context.execution_request();
        let request_execution = &context_execution;
        let _owner_completion = context.execute_installed_read(owner_workspace)?;
        let mut foreign =
            workspace(request_execution, "foreign-material-source", false).map_err(|error| {
                domain::WorthQueryOperationExecutorFailure::new(
                    domain::WorthQueryOperationFailureClass::Dependency,
                    format!("{error:?}"),
                )
            })?;
        let completion = execute_reconstructed_read_for_sabotage(request_execution, &mut foreign)?;
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            completion,
            input.state,
        ))
    }
}

pub fn foreign_material_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(package(false, false))
        .domain_operation_executor(
            GeometryDomain,
            ReadVertex,
            ReadFamily,
            ForeignRuntimeReadExecutor,
        )
        .domain_operation_executor(
            GeometryDomain,
            CountVertices,
            ReadFamily,
            CountVerticesExecutor,
        )
        .workspace(request_execution, name)
}

pub fn mismatched_read_plan_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(package(false, false))
        .domain_operation_executor(
            GeometryDomain,
            ReadVertex,
            ReadFamily,
            MismatchedReadPlanExecutor,
        )
        .domain_operation_executor(
            GeometryDomain,
            CountVertices,
            ReadFamily,
            CountVerticesExecutor,
        )
        .workspace(request_execution, name)
}

pub fn missing_read_execution_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(package(false, false))
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, ReadVertexExecutor)
        .domain_operation_executor(
            GeometryDomain,
            CountVertices,
            ReadFamily,
            MissingReadExecutionExecutor,
        )
        .workspace(request_execution, name)
}

pub fn mismatched_cost_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(package(false, false))
        .domain_operation_executor(
            GeometryDomain,
            ReadVertex,
            ReadFamily,
            MismatchedCostExecutor,
        )
        .domain_operation_executor(
            GeometryDomain,
            CountVertices,
            ReadFamily,
            CountVerticesExecutor,
        )
        .workspace(request_execution, name)
}

pub fn mismatched_determinism_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(package(false, false))
        .domain_operation_executor(
            GeometryDomain,
            ReadVertex,
            ReadFamily,
            MismatchedDeterminismExecutor,
        )
        .domain_operation_executor(
            GeometryDomain,
            CountVertices,
            ReadFamily,
            CountVerticesExecutor,
        )
        .workspace(request_execution, name)
}
