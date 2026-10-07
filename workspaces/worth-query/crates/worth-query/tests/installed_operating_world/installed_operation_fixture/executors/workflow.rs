use worth_query::facade::{domain, read};

use super::super::{GeometryDomain, ReadFamily, WorkflowRead};
use super::installed_read_declaration;

#[derive(Clone, Copy)]
pub(crate) struct WorkflowStageExecutor;

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowStageExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = Some("installed-workflow-exact-v1");

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, _computed, context, workspace) = application.into_parts();
        materialize_owned_workflow_stage(input, context, workspace)
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowStageExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::compare_exact_workflow_traces(original, replay, noise)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MismatchedWorkflowStageExecutor;

#[derive(Clone, Copy)]
pub(crate) struct MismatchedWorkflowDeterminismExecutor;

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for MismatchedWorkflowDeterminismExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = false;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = Some("installed-workflow-exact-v1");

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        domain::WorthQueryDomainWorkflowStageExecutor::apply(&WorkflowStageExecutor, application)
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for MismatchedWorkflowDeterminismExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::compare_exact_workflow_traces(original, replay, noise)
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for MismatchedWorkflowStageExecutor
{
    const LOWERING_FAMILY: &'static str = "foreign-workflow-lowering-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        domain::WorthQueryDomainWorkflowStageExecutor::apply(&WorkflowStageExecutor, application)
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for MismatchedWorkflowStageExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::compare_exact_workflow_traces(original, replay, noise)
    }
}

pub(crate) fn materialize_owned_workflow_stage(
    input: domain::WorthQueryWorkflowValue,
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    if context.stage().identity() == "left" {
        match &input {
            domain::WorthQueryWorkflowValue::Text(value) if value == "fail-dependency" => {
                return Err(domain::WorthQueryWorkflowStageExecutorFailure::new(
                    domain::WorthQueryOperationFailureClass::Dependency,
                    "declared dependency failure",
                ));
            }
            domain::WorthQueryWorkflowValue::Text(value) if value == "fail-unsupported" => {
                return Err(domain::WorthQueryWorkflowStageExecutorFailure::new(
                    domain::WorthQueryOperationFailureClass::Unsupported,
                    "undeclared unsupported failure",
                ));
            }
            domain::WorthQueryWorkflowValue::Text(value) if value == "read-undeclared" => {
                let _ = context.execute_installed_read("model", workspace)?;
                unreachable!("stage-local admission must deny before the read")
            }
            _ => {}
        }
    }
    let material = if context.stage().identity() == "publish"
        && matches!(&input, domain::WorthQueryWorkflowValue::Text(value) if value == "skip-read")
    {
        domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
            "dishonest-publication".into(),
        ))
        .with_result_state(domain::WorthQueryOperationResultState::Ready)
    } else if context.stage().identity() == "publish" {
        domain::WorthQueryWorkflowStageMaterial::projection(
            "model",
            context.execute_installed_read("model", workspace)?,
        )
        .with_result_state(domain::WorthQueryOperationResultState::Ready)
    } else {
        domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
            context.stage().identity().into(),
        ))
    };
    Ok(material)
}
