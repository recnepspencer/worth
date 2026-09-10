use worth_query::facade::{domain, read, runtime};

use super::super::super::graph_read_material::graph_read_material;
use super::super::workflow_parallel_providers::WorkflowParallelProvider;
use super::super::{
    canonical_bundle, configured_runtime_without_executors, semantic_closure, GeometryDomain,
    ReadFamily, WorkflowRead,
};
use super::definitions::valid_stages;

#[derive(Clone, Copy, Debug)]
struct WorkflowRemoteGraph;

#[derive(Clone, Copy)]
struct WorkflowGraphProjectionExecutor;

struct WorkflowGraphProjectionProvider;

pub fn workflow_graph_projection_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(workflow_graph_projection_package())
        .graph_participation(workflow_graph_definition())
        .graph_participation_provider(WorkflowRemoteGraph, WorkflowGraphProjectionProvider)
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowGraphProjectionExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(name)
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowGraphProjectionExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = Some("installed-workflow-exact-v1");

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::super::executors::installed_read_declaration())
    }

    fn execute_stage(
        &self,
        _input: domain::WorthQueryWorkflowValue,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        if context.stage().identity() != "publish" {
            return Ok(domain::WorthQueryWorkflowStageMaterial::new(
                domain::WorthQueryWorkflowValue::Text(context.stage().identity().into()),
            ));
        }
        let remote = context.graph_projection("remote-a").ok_or_else(|| {
            workflow_failure("the remote-a execution product was not carried into the stage")
        })?;
        let first_remote_identity = remote
            .rows()
            .next()
            .map(domain::WorthQueryGraphReadRow::entity_identity)
            .ok_or_else(|| workflow_failure("the remote-a execution product contained no rows"))?;
        let completion = context.execute_installed_read("model", workspace)?;
        Ok(
            domain::WorthQueryWorkflowStageMaterial::projection("model", completion)
                .with_warning(domain::WorthQueryWorkflowStageWarning::Advisory(format!(
                    "remote-a-rows={};first={first_remote_identity}",
                    remote.rows().len()
                )))
                .with_result_state(domain::WorthQueryOperationResultState::Ready),
        )
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for WorkflowGraphProjectionExecutor
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

impl domain::WorthQueryGraphParticipationProvider<WorkflowRemoteGraph>
    for WorkflowGraphProjectionProvider
{
    type Execution = crate::suite::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn begin(
        &self,
        call: &domain::WorthQueryGraphProviderCall,
        start: &mut domain::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        domain::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        domain::WorthQueryGraphProviderFailure,
    > {
        let execution = match call.kind() {
            domain::WorthQueryGraphProviderCallKind::Observe => {
                Self::Execution::read("workflow-remote-observe")
            }
            domain::WorthQueryGraphProviderCallKind::Project => Self::Execution::projection(
                "workflow-remote-project",
                graph_read_material("workflow-remote-row"),
            ),
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect("workflow-remote-touch")
            }
            domain::WorthQueryGraphProviderCallKind::CommitAdmission => {
                unreachable!("graph participation never receives commit admission")
            }
        };
        start
            .admit_cooperative_execution(|| execution)
            .map_err(|denial| domain::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}

fn workflow_graph_projection_package() -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let mut stages = valid_stages();
    let mut publish_semantics = stages[3].semantics().clone();
    publish_semantics.graph_read_roles.push("remote-a".into());
    stages[3] = stages[3].clone().with_semantics(publish_semantics);
    let workflow = domain::WorthQueryPortableWorkflowDefinition::new("start", stages);
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    if let domain::WorthQueryOperationGraphReadContract::DeclaredDomain { roles } =
        &mut semantics.graph_reads
    {
        roles.push(domain::WorthQueryDomainOperationGraphReadRole {
            role: "remote-a".into(),
            participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                role: "remote-a".into(),
            },
            access: domain::WorthQueryOperationGraphAccess::Project,
            semantic_reads: Vec::new(),
        });
    }
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(workflow);
    semantics.cost.execution = domain::WorthQueryOperationCostClass::ExternalBoundary;
    semantics.replay = domain::WorthQueryOperationReplayContract::CertReplayable {
        comparator: domain::WorthQueryOperationReplayComparatorContract::new(
            "installed-workflow-exact-v1",
        )
        .expect("static replay comparator family is portable"),
    };
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-graph-projection", 1),
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
    .operation_graph_participation::<WorkflowRead, ReadFamily, WorkflowRemoteGraph>("remote-a")
}

fn workflow_graph_definition() -> domain::WorthQueryGraphParticipationDefinition<WorkflowRemoteGraph>
{
    domain::WorthQueryGraphParticipationDefinition::new(
        "remote-a",
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::ReadOnly,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}

fn workflow_failure(detail: impl Into<String>) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        detail,
    )
}
