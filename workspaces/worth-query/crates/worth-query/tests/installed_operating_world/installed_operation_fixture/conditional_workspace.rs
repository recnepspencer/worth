use worth_query::facade::{domain, runtime};

use super::executors::ReadVertexExecutor;
use super::{
    configured_runtime_without_executors, read_vertex_definition, GeometryDomain, ReadFamily,
    ReadVertex,
};

mod controlled_workspace;
mod installation;
mod providers;
mod resource_lifecycle;
pub(super) mod source_seed;

pub(crate) use controlled_workspace::{
    conditional_controlled_workspace, conditional_controlled_workspace_with_donor,
    ConditionalDonorWorkspaceScenario, ConditionalWorkspacePlacement,
};
pub(crate) use installation::{
    conditional_installation, conditional_installation_without_observation, ConditionalInstallation,
};
use providers::providers_for;
pub(crate) use providers::DirectConditionalCompute;
pub(crate) use resource_lifecycle::{
    conditional_resource_workspace, ConditionalResourceFamily, ConditionalResourceOperation,
};

#[derive(Clone, Copy, Debug)]
pub struct ConditionalModelGraph;

pub(crate) fn fixture_record_identity(
) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
    worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 0, 1)
}

pub(super) struct ConditionalModelGraphProvider;

pub(super) fn conditional_model_graph_definition(
) -> domain::WorthQueryGraphParticipationDefinition<ConditionalModelGraph> {
    conditional_model_graph_definition_with_identity(domain::WorthQueryGraphIdentityPosture::Opaque)
}

pub(super) fn conditional_lineage_model_graph_definition(
) -> domain::WorthQueryGraphParticipationDefinition<ConditionalModelGraph> {
    conditional_model_graph_definition_with_identity(
        domain::WorthQueryGraphIdentityPosture::EvolvingLineage,
    )
}

fn conditional_model_graph_definition_with_identity(
    identity: domain::WorthQueryGraphIdentityPosture,
) -> domain::WorthQueryGraphParticipationDefinition<ConditionalModelGraph> {
    domain::WorthQueryGraphParticipationDefinition::new(
        "model",
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity,
            locality: domain::WorthQueryGraphLocalityPosture::InProcess,
            budget: domain::WorthQueryGraphBudgetPosture::ConstantAdmission,
            commit: domain::WorthQueryGraphCommitPosture::ReadOnly,
            failure: domain::WorthQueryGraphFailureTopology::Local,
        },
    )
}

impl domain::WorthQueryGraphParticipationProvider<ConditionalModelGraph>
    for ConditionalModelGraphProvider
{
    type Execution = crate::suite::graph_provider_step::FixtureGraphProviderExecution;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
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
                Self::Execution::read("conditional-model-observe")
            }
            domain::WorthQueryGraphProviderCallKind::Project => {
                Self::Execution::read("conditional-model-project")
            }
            domain::WorthQueryGraphProviderCallKind::TouchEffect => {
                Self::Execution::effect("conditional-model-touch")
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

pub fn conditional_workspace(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let installation = conditional_installation(&node);
    conditional_workspace_with(name, node, installation, DirectConditionalCompute)
}

pub(crate) fn conditional_workspace_with<P>(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    installation: ConditionalInstallation,
    compute: P,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>,
{
    conditional_workspace_with_builder(node, installation, compute).workspace(name)
}

pub(crate) fn conditional_workspace_with_builder<P>(
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    installation: ConditionalInstallation,
    compute: P,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>,
{
    conditional_workspace_builder(vec![node])
        .with_seeded_conditional_installation(move |builder, source, seed| {
            let installation = installation.prepare_seeded(source, seed);
            let builder = builder.conditional_node(
                GeometryDomain,
                ReadVertex,
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
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, ReadVertexExecutor)
}
pub(crate) fn conditional_workspace_without_lowering(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    conditional_workspace_builder(vec![node])
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, ReadVertexExecutor)
        .workspace(name)
}

fn conditional_workspace_builder(
    nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let dependencies = nodes
        .iter()
        .flat_map(|node| node.dependencies().iter().cloned())
        .collect::<Vec<_>>();
    source_seed::seeded_builder(
        configured_runtime_without_executors(conditional_package(nodes)),
        dependencies,
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
}

fn conditional_package(
    nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    conditional_package_with_access(nodes, domain::WorthQueryOperationGraphAccess::Project)
}

fn conditional_package_with_access(
    nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
    graph_access: domain::WorthQueryOperationGraphAccess,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let base = read_vertex_definition(domain::WorthQuerySupportRequirement::Required);
    let mut semantics = base.semantics().clone();
    if let domain::WorthQueryOperationGraphReadContract::DeclaredDomain { roles } =
        &mut semantics.graph_reads
    {
        let model = roles
            .iter_mut()
            .find(|role| role.role == "model")
            .expect("conditional fixture declares the model graph read");
        model.access = graph_access;
        for dependency in nodes.iter().flat_map(|node| node.dependencies()) {
            let projection = domain::WorthQueryOperationNativeProjectionContract::new(
                dependency.contract().clone(),
                dependency.projection_mask().clone(),
            )
            .expect("conditional dependency projection is admitted by its contract");
            if !model.semantic_reads.contains(&projection) {
                model.semantic_reads.push(projection);
            }
        }
    }
    semantics.conditional_nodes = nodes;
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
    >::new(base.identity().clone(), semantics);

    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
    .operation_graph_participation::<ReadVertex, ReadFamily, ConditionalModelGraph>("model")
}

pub(crate) fn shared_signal_node_workspace(
    name: &str,
    first: domain::WorthQueryPortableConditionalNodeDeclaration,
    second: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let second_providers = providers_for(&second);
    let first_identity = first.identity().to_string();
    let second_identity = second.identity().to_string();
    let installation = conditional_installation(&first);
    let worth_proof::TransitionOutcome::Success(second_node) = installation
        .graph
        .admit_installed_node(installation.signal_node)
    else {
        panic!("the installed Signal node should remain current during fixture construction")
    };
    let second_target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new("geometry-signal"),
        second_node,
    );
    conditional_workspace_builder(vec![first, second])
        .with_seeded_conditional_installation(move |builder, source, seed| {
            let installation = installation.prepare_seeded(source, seed);
            let second_dependency = domain::WorthQueryConditionalDependencyInstallation::new(
                Some(installation.record),
                vec![second_target],
            );
            let builder = builder
                .conditional_node(
                    GeometryDomain,
                    ReadVertex,
                    ReadFamily,
                    ConditionalModelGraph,
                    domain::WorthQueryConditionalNodeLocation::operation(first_identity).unwrap(),
                    vec![installation.dependency],
                    installation.providers,
                    DirectConditionalCompute,
                )
                .conditional_node(
                    GeometryDomain,
                    ReadVertex,
                    ReadFamily,
                    ConditionalModelGraph,
                    domain::WorthQueryConditionalNodeLocation::operation(second_identity).unwrap(),
                    vec![second_dependency],
                    second_providers,
                    DirectConditionalCompute,
                );
            Ok((
                builder,
                installation.bridge,
                installation.graph,
                runtime::WorthQueryConditionalExecutionResources::development(),
            ))
        })
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, ReadVertexExecutor)
        .workspace(name)
}
