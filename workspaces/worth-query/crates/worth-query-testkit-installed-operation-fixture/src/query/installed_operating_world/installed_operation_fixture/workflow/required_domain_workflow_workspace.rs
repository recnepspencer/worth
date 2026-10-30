//! required domain workflow workspace.
use super::*;

pub fn required_domain_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<
    worth_query::facade::consumer_kit::WorthQueryControlledTestWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.lowering.deterministic = true;
    semantics.replay = domain::WorthQueryOperationReplayContract::CertReplayable {
        comparator: domain::WorthQueryOperationReplayComparatorContract::new(
            "installed-workflow-exact-v1",
        )
        .expect("static replay comparator family is portable"),
    };
    semantics.workflow =
        domain::WorthQueryOperationWorkflowContract::Declared(workflow_definition(None));
    semantics
        .required_domains
        .push(domain::WorthQueryOperationRequiredDomainRole::new("auxiliary").unwrap());
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-read", 1),
        semantics,
    );
    let geometry = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain_identity::<GeometryDomain>("geometry"),
    )
    .operation(operation)
    .operation_required_domain::<WorkflowRead, ReadFamily, AuxiliaryDomain>("auxiliary");
    configured_runtime_without_executors(geometry)
        .domain_package(domain::WorthQueryDomainPackage::declare(
            AuxiliaryDomain,
            domain_identity::<AuxiliaryDomain>("auxiliary"),
        ))
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
