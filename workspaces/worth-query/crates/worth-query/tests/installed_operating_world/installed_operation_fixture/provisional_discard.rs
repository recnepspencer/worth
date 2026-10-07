use worth_query::facade::{domain, runtime};

use super::{
    canonical_bundle, configured_runtime_without_executors, semantic_closure, GeometryDomain,
};

#[derive(Clone, Copy, Debug)]
pub struct ProvisionalWorkflow;

#[derive(Clone, Copy, Debug)]
pub struct ProvisionalDiscardFamily;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ProvisionalDiscardFamily>
    for ProvisionalWorkflow
{
    type Input = ();
    type Output = ();
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryWorkflowOperation;
}

#[derive(Clone, Copy)]
struct ProvisionalExecutor;

impl
    domain::WorthQueryDomainWorkflowStageExecutor<
        GeometryDomain,
        ProvisionalWorkflow,
        ProvisionalDiscardFamily,
    > for ProvisionalExecutor
{
    const LOWERING_FAMILY: &'static str = "provisional-workflow-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (_input, _computed, _context, _workspace) = application.into_parts();
        Ok(
            domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
                "provisional-result".into(),
            ))
            .with_result_state(domain::WorthQueryOperationResultState::Ready),
        )
    }
}

pub fn provisional_workflow_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::NotRequired,
        false,
    );
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::NotRequired;
    semantics.aftermath = None;
    semantics.lowering.family = "provisional-workflow-v1".into();
    semantics.terminal.failure_classes = vec![
        domain::WorthQueryOperationFailureClass::InvalidInput,
        domain::WorthQueryOperationFailureClass::Dependency,
    ];
    semantics.workflow =
        domain::WorthQueryOperationWorkflowContract::Declared(one_stage_workflow());
    let package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ProvisionalWorkflow,
        ProvisionalDiscardFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("provisional-workflow", 1),
        semantics,
    ));
    configured_runtime_without_executors(package)
        .workflow_stage_executor(
            GeometryDomain,
            ProvisionalWorkflow,
            ProvisionalDiscardFamily,
            ProvisionalExecutor,
        )
        .workspace(name)
}

fn one_stage_workflow() -> domain::WorthQueryPortableWorkflowDefinition {
    domain::WorthQueryPortableWorkflowDefinition::new(
        "apply",
        [domain::WorthQueryPortableWorkflowStage::new(
            "apply",
            std::iter::empty::<&str>(),
            true,
            false,
            std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
        )
        .with_semantics(domain::WorthQueryWorkflowStageSemantics {
            input: domain::WorthQueryWorkflowValueContract::EntityIdentity,
            output: domain::WorthQueryWorkflowValueContract::Text,
            effect_roles: Vec::new(),
            cost_roles: vec![
                domain::WorthQueryWorkflowCostRole::Admission,
                domain::WorthQueryWorkflowCostRole::Execution,
                domain::WorthQueryWorkflowCostRole::ResultValidation,
            ],
            resources: super::execution_resource_contract(),
            terminal_result_states: vec![domain::WorthQueryOperationResultState::Ready],
            failure_classes: vec![
                domain::WorthQueryOperationFailureClass::InvalidInput,
                domain::WorthQueryOperationFailureClass::Dependency,
            ],
            ..Default::default()
        })],
    )
}
