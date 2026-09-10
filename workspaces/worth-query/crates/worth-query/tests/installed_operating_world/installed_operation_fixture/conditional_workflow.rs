use worth_query::facade::{domain, runtime};

use super::conditional_workspace::{
    conditional_installation, conditional_lineage_model_graph_definition,
    conditional_model_graph_definition, ConditionalInstallation, ConditionalModelGraphProvider,
};
use super::executors::WorkflowStageExecutor;
use super::lineage_workflow::{deferred_lineage_workflow_package, lineage_stages};
use super::workflow::{
    valid_stages, workflow_package, workflow_package_with_operation_conditionals,
};
use super::workflow_parallel_providers::WorkflowParallelProvider;
use super::{
    configured_runtime_without_executors, ConditionalModelGraph, GeometryDomain, ReadFamily,
    WorkflowRead,
};

pub fn conditional_workflow_workspace(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_conditional_workflow_workspace(
        name,
        node,
        "publish",
        false,
        None,
        WorkflowConditionalCompute(1),
    )
}

pub fn reverted_conditional_lineage_workflow_workspace(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_conditional_workflow_workspace(
        name,
        node,
        "publish",
        true,
        None,
        WorkflowConditionalCompute(0),
    )
}

pub(crate) fn operation_conditional_workflow_workspace_with<P>(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    installation: ConditionalInstallation,
    compute: P,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>,
{
    let workflow = domain::WorthQueryPortableWorkflowDefinition::new("start", valid_stages());
    let package = workflow_package_with_operation_conditionals(workflow, true, vec![node])
        .operation_graph_participation::<WorkflowRead, ReadFamily, ConditionalModelGraph>(
        "model",
    );
    super::conditional_workspace::source_seed::seeded_builder(
        configured_runtime_without_executors(package),
        [installation.semantic().clone()],
    )
    .graph_participation(conditional_model_graph_definition())
    .graph_participation_provider(ConditionalModelGraph, ConditionalModelGraphProvider)
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalComparator,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTrigger,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .with_seeded_conditional_installation(move |builder, source, seed| {
        let installation = installation.prepare_seeded(source, seed);
        let builder = builder.conditional_node(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            ConditionalModelGraph,
            domain::WorthQueryConditionalNodeLocation::operation(installation.node_identity)
                .unwrap(),
            vec![installation.dependency],
            installation.providers,
            compute,
        );
        Ok((
            builder,
            installation.bridge,
            installation.graph,
            runtime::WorthQueryConditionalExecutionResources::development(),
        ))
    })
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
    .workspace(name)
}

pub(crate) fn stage_conditional_workflow_workspace_with<P>(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    conditional_stage: &str,
    installation: ConditionalInstallation,
    compute: P,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>,
{
    configured_conditional_workflow_workspace(
        name,
        node,
        conditional_stage,
        false,
        Some(installation),
        compute,
    )
}

fn configured_conditional_workflow_workspace<P>(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    conditional_stage: &str,
    with_lineage: bool,
    installation: Option<ConditionalInstallation>,
    compute: P,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>,
{
    let installation = installation.unwrap_or_else(|| conditional_installation(&node));
    let mut stages = if with_lineage {
        lineage_stages()
    } else {
        valid_stages()
    };
    let stage_index = stages
        .iter()
        .position(|stage| stage.identity() == conditional_stage)
        .expect("the conditional stage belongs to the standard workflow");
    let stage = stages.remove(stage_index);
    let mut semantics = stage.semantics().clone();
    semantics.conditional_nodes = vec![node];
    stages.insert(stage_index, stage.with_semantics(semantics));
    let workflow = domain::WorthQueryPortableWorkflowDefinition::new("start", stages);
    let package = if with_lineage {
        deferred_lineage_workflow_package(workflow)
    } else {
        workflow_package(workflow, true)
    }
    .operation_graph_participation::<WorkflowRead, ReadFamily, ConditionalModelGraph>("model");
    let conditional_stage = conditional_stage.to_string();
    let graph_definition = if with_lineage {
        conditional_lineage_model_graph_definition()
    } else {
        conditional_model_graph_definition()
    };
    super::conditional_workspace::source_seed::seeded_builder(
        configured_runtime_without_executors(package),
        [installation.semantic().clone()],
    )
    .graph_participation(graph_definition)
    .graph_participation_provider(ConditionalModelGraph, ConditionalModelGraphProvider)
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalComparator,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTrigger,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .with_seeded_conditional_installation(move |builder, source, seed| {
        let installation = installation.prepare_seeded(source, seed);
        let builder = builder.conditional_node(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            ConditionalModelGraph,
            domain::WorthQueryConditionalNodeLocation::workflow_stage(
                conditional_stage,
                installation.node_identity,
            )
            .unwrap(),
            vec![installation.dependency],
            installation.providers,
            compute,
        );
        Ok((
            builder,
            installation.bridge,
            installation.graph,
            runtime::WorthQueryConditionalExecutionResources::development(),
        ))
    })
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
    .workspace(name)
}

struct WorkflowConditionalCompute(u64);
impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowConditionalCompute
{
    type SemanticContract = u64;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        if context.workflow_run_identity().is_none() {
            return Err("workflow conditional compute lost its run identity".into());
        }
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.0,
            )]),
        ))
    }
}
