use worth_query::facade::{domain, runtime};

use super::super::{execution_resource_support, read_vertex_definition, GeometryDomain};
use super::{
    conditional_model_graph_definition, ConditionalInstallation, ConditionalModelGraph,
    ConditionalModelGraphProvider,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConditionalResourceOperation;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConditionalResourceFamily;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ConditionalResourceFamily>
    for ConditionalResourceOperation
{
    type Input = ();
    type Output = ();
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

struct ConditionalResourceExecutor;

impl
    domain::WorthQueryDomainOperationExecutor<
        GeometryDomain,
        ConditionalResourceOperation,
        ConditionalResourceFamily,
    > for ConditionalResourceExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        execution_resource_support()
    }

    fn execute(
        &self,
        _input: (),
        _context: &domain::WorthQueryOperationExecutionContext<'_>,
        _workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<()>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            (),
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

struct ConditionalResourceCompute;

impl
    domain::WorthQueryConditionalNodeComputeProvider<
        GeometryDomain,
        ConditionalResourceOperation,
        ConditionalResourceFamily,
    > for ConditionalResourceCompute
{
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

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
        execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}

pub(crate) fn conditional_resource_workspace(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
    installation: ConditionalInstallation,
    harness: &crate::support::public_bridge_runtime::PublicBridgeRuntimeHarness,
    resources: runtime::WorthQueryConditionalExecutionResources,
) -> Result<runtime::WorthQueryWorkspace, runtime::WorthQueryRuntimeError> {
    let dependency_contract = node.dependencies()[0].contract().clone();
    let package = conditional_resource_package(&node);
    let installation = installation.prepare_public();
    let builder = runtime::WorthQueryRuntime::builder()
        .domain_package(package)
        .expect("conditional resource package should admit")
        .graph_participation(conditional_model_graph_definition())
        .graph_participation_provider(ConditionalModelGraph, ConditionalModelGraphProvider)
        .relational_source_owner(installation.owner.clone())
        .conditional_execution_resources(resources)
        .conditional_signal_graph(installation.graph)
        .conditional_node(
            GeometryDomain,
            ConditionalResourceOperation,
            ConditionalResourceFamily,
            ConditionalModelGraph,
            domain::WorthQueryConditionalNodeLocation::operation(installation.node_identity)
                .unwrap(),
            vec![installation.dependency],
            installation.providers,
            ConditionalResourceCompute,
        )
        .domain_operation_executor(
            GeometryDomain,
            ConditionalResourceOperation,
            ConditionalResourceFamily,
            ConditionalResourceExecutor,
        );
    let builder = install_support_postures(builder);
    harness
        .configure_runtime_builder(
            builder,
            installation.bridge,
            [super::super::identity_contract(), dependency_contract],
            crate::support::public_bridge_runtime::public_graph_support_profile(),
        )
        .build_backend_from_parts()
        .build()?
        .workspace(name)
}

fn conditional_resource_package(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let base = read_vertex_definition(domain::WorthQuerySupportRequirement::NotRequired);
    let mut semantics = base.semantics().clone();
    let native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
        node.dependencies()[0].contract().clone(),
        node.dependencies()[0].projection_mask().clone(),
    )
    .unwrap();
    semantics.conditional_nodes = vec![node.clone()];
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
        roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
            role: "model".into(),
            participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                role: "model".into(),
            },
            access: domain::WorthQueryOperationGraphAccess::Observe,
            semantic_reads: vec![native_projection],
        }],
    };
    semantics.publication = domain::WorthQueryOperationPublicationContract::NotRequired;
    semantics.projection_consumption =
        domain::WorthQueryOperationProjectionConsumptionContract::NotRequired;
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ConditionalResourceOperation,
        ConditionalResourceFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("conditional-resource", 1),
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
    .operation_graph_participation::<
        ConditionalResourceOperation,
        ConditionalResourceFamily,
        ConditionalModelGraph,
    >("model")
}

fn install_support_postures(
    mut builder: runtime::WorthQueryRuntimeBuilder,
) -> runtime::WorthQueryRuntimeBuilder {
    for dimension in [
        domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
        domain::WorthQueryConsumerSupportDimension::ConditionalComparator,
        domain::WorthQueryConsumerSupportDimension::ConditionalTrigger,
        domain::WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
        domain::WorthQueryConsumerSupportDimension::Live,
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
        domain::WorthQueryConsumerSupportDimension::Invalidation,
    ] {
        builder = builder.consumer_support_posture(
            dimension,
            domain::WorthQueryConsumerSupportPosture::Supported,
        );
    }
    builder
}
