use worth_query::facade::{domain, runtime};

use super::executors::{
    MismatchedWorkflowDeterminismExecutor, MismatchedWorkflowStageExecutor, WorkflowStageExecutor,
};
use super::workflow_parallel_providers::{SerialParallelProvider, WorkflowParallelProvider};
use super::{
    canonical_bundle, configured_runtime_without_executors, semantic_closure, AuxiliaryDomain,
    GeometryDomain, ReadFamily, WorkflowRead,
};

#[path = "workflow/definitions.rs"]
mod definitions;
#[path = "workflow/graph_projection.rs"]
mod graph_projection;
#[path = "workflow/resource_admission.rs"]
mod resource_admission;
pub(super) use definitions::valid_stages;
pub use definitions::InvalidWorkflow;
use definitions::{no_value, projection, stage, text, workflow_definition};
pub use graph_projection::workflow_graph_projection_workspace;
pub(crate) use resource_admission::workflow_workspace_with_parallel_provider;

pub fn workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    build_workspace(request_execution, name, workflow_definition(None))
}

pub fn controlled_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    worth_query::facade::consumer_kit::WorthQueryControlledTestWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .controlled_workspace(request_execution, name)
}

pub fn failing_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    failed_closes: usize,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .fail_next_live_closes(failed_closes)
        .workspace(request_execution, name)
}

#[path = "workflow/required_domain_workflow_workspace.rs"]
mod required_domain_workflow_workspace;
pub use required_domain_workflow_workspace::required_domain_workflow_workspace;

pub fn reversed_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let mut stages = valid_stages();
    stages.reverse();
    build_workspace(
        request_execution,
        name,
        domain::WorthQueryPortableWorkflowDefinition::new("start", stages),
    )
}

pub fn divergent_frontier_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    build_workspace(
        request_execution,
        name,
        domain::WorthQueryPortableWorkflowDefinition::new(
            "start",
            [
                stage("start", [], false, false, no_value(), text()),
                stage("left", ["start"], false, false, text(), text()),
                stage("bridge", ["start"], false, false, text(), text()),
                stage("right", ["bridge"], false, false, text(), text()),
                stage(
                    "publish",
                    ["left", "right"],
                    true,
                    true,
                    text(),
                    projection(),
                ),
            ],
        ),
    )
}

pub fn invalid_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    invalid: InvalidWorkflow,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    build_workspace(request_execution, name, workflow_definition(Some(invalid)))
}

pub fn mismatched_workflow_lowering_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            MismatchedWorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)
}

pub fn mismatched_workflow_determinism_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            MismatchedWorkflowDeterminismExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)
}

pub fn missing_replay_comparator_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)
}

pub fn missing_parallel_provider_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workspace(request_execution, name)
}

pub fn serial_parallel_provider_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            SerialParallelProvider,
        )
        .workspace(request_execution, name)
}

pub fn nondeterministic_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow_definition(None), false))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            MismatchedWorkflowDeterminismExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)
}

pub(super) fn build_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    workflow: domain::WorthQueryPortableWorkflowDefinition,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_package(workflow, true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)
}

pub(super) fn workflow_package(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    deterministic: bool,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    workflow_package_with_operation_conditionals(workflow, deterministic, Vec::new())
}

pub(super) fn workflow_package_with_operation_conditionals(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    deterministic: bool,
    conditional_nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.lowering.deterministic = deterministic;
    semantics.replay = domain::WorthQueryOperationReplayContract::CertReplayable {
        comparator: domain::WorthQueryOperationReplayComparatorContract::new(
            "installed-workflow-exact-v1",
        )
        .expect("static replay comparator family is portable"),
    };
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(workflow);
    semantics.conditional_nodes = conditional_nodes;
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-read", 1),
        semantics,
    );
    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
}

fn domain_identity<D>(name: &str) -> domain::WorthQueryDomainIdentityDeclaration<D> {
    domain::WorthQueryDomainIdentityDeclaration::new(
        domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        domain::WorthQueryDomainIdentityName::new(name).unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
    )
}
