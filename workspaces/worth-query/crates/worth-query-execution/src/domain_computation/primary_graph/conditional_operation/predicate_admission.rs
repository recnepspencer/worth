use std::sync::Arc;

use worth_query_installation::facade::{
    WorthQueryBoundHostOutputComparator, WorthQueryBoundHostOutputVersion,
    WorthQueryHostConditionalOutputComparatorProvider,
    WorthQueryHostConditionalOutputVersionProvider, WorthQueryHostConditionalPredicateProvider,
    WorthQueryInstalledApplicationConditionalNode, WorthQueryInstalledGraphParticipationAuthority,
    WorthQueryInstalledHostConditionalProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryOutputEquivalenceRequirement,
    WorthQueryPortableConditionalNodeDeclaration, WorthQuerySemanticLocality,
    WorthQueryTemporalIntentProjector,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalProviderSemantics,
    BridgeConditionalProviderSet, BridgeOwnedConditionalInstallationRequest,
    BridgeSemanticDependencyCandidate, BridgeSemanticDependencyCandidateParts,
    BridgeSemanticLocality,
};

use super::installation::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use super::predicate_observation::QueryTemporalPredicateProvider;

mod authored_outputs;
mod contract_lowering;
use contract_lowering::{lower_dependency_contract, lower_location, lower_temporal_contract};
mod output_readiness_installation;
mod providers;
use authored_outputs::{QueryAuthoredConditionalComputeProvider, QueryAuthoredOutputComparator};
mod readiness_provider;
pub(in crate::domain_computation::primary_graph) use output_readiness_installation::prepare_dependency_conditional_installation;
pub(in crate::domain_computation::primary_graph) use providers::QueryConditionalComputeContext;
use providers::{
    QueryConditionalComputeProvider, QueryConditionalComputeSemanticContract,
    QueryHostOutputComparator,
};
use readiness_provider::QueryOutputReadinessPredicate;

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_temporal_predicate_installation<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
    Clock,
    Source,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Projector,
>(
    binding: &WorthQueryInstalledTemporalConditionalOperation<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<BridgeOwnedConditionalInstallationRequest, WorthQueryConditionalRuntimeInstallationDenial>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    Node: 'static,
{
    prepare_host_predicate_installation(binding.clocked_node().provider(), graph)
}

pub(super) fn prepare_host_predicate_installation<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
>(
    installed_provider: &WorthQueryInstalledHostConditionalProvider<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
    >,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<BridgeOwnedConditionalInstallationRequest, WorthQueryConditionalRuntimeInstallationDenial>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Node: 'static,
{
    let node = installed_provider.node();
    let contract = lower_temporal_contract(node.declaration())?;
    let location = lower_location(node.location());
    let dependencies = dependency_candidates(node, graph)?;
    let node_authority: Arc<str> = Arc::from(node.authority_identity());
    let portable_node: Arc<str> = Arc::from(node.declaration().identity());
    let output_version = installed_provider.retain_output_version_for_runtime();
    let output_comparator = installed_provider.retain_output_comparator_for_runtime();
    validate_output_comparator_identity(node.declaration(), output_comparator.as_ref())?;
    let authored_predicate = installed_provider.retain_authored_predicate_for_runtime();
    let authored = authored_predicate.is_some();
    let providers = match authored_predicate {
        Some(source) => BridgeConditionalProviderSet::new().authored_wake(
            super::authored_predicate::QueryAuthoredPredicateProvider::new(
                source,
                portable_node.clone(),
                node_authority.clone(),
            ),
        ),
        None => BridgeConditionalProviderSet::new().wake(QueryTemporalPredicateProvider::<
            Node,
            Provider,
        >::new(
            installed_provider.retain_provider_for_runtime(),
            node_authority.clone(),
        )),
    };
    let providers = match output_version {
        None if authored => providers.authored_compute(QueryAuthoredConditionalComputeProvider::<
            Node,
        >::new(portable_node.clone(), None)),
        None => providers.compute(QueryConditionalComputeProvider::<Node> {
            semantics: QueryConditionalComputeSemanticContract(
                format!("{node_authority}|output-version=query-attempt").into(),
            ),
            output_version: None,
        }),
        Some(WorthQueryBoundHostOutputVersion::Authored(provider)) => {
            providers.authored_compute(QueryAuthoredConditionalComputeProvider::<Node>::new(
                portable_node.clone(),
                Some(provider),
            ))
        }
        Some(WorthQueryBoundHostOutputVersion::Memory(provider)) => {
            let compute_identity: Arc<str> = format!(
                "{}|output-version={}",
                node_authority,
                provider.semantic_identity()
            )
            .into();
            providers.compute(QueryConditionalComputeProvider::<Node> {
                semantics: QueryConditionalComputeSemanticContract(compute_identity),
                output_version: Some(provider),
            })
        }
    };
    let providers = match output_comparator {
        Some(WorthQueryBoundHostOutputComparator::Authored(provider)) => providers
            .authored_output_comparator(QueryAuthoredOutputComparator::<Node>::new(
                portable_node,
                provider,
            )),
        Some(WorthQueryBoundHostOutputComparator::Memory(provider)) => {
            providers.output_comparator(QueryHostOutputComparator::<Node> {
                identity: Arc::from(provider.semantic_identity()),
                provider,
            })
        }
        None => providers,
    };
    Ok(BridgeOwnedConditionalInstallationRequest {
        contract,
        location,
        dependencies,
        providers,
    })
}

fn validate_output_comparator_identity<Node: 'static>(
    declaration: &WorthQueryPortableConditionalNodeDeclaration,
    installed: Option<&WorthQueryBoundHostOutputComparator<Node>>,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    let WorthQueryOutputEquivalenceRequirement::Registered(required) =
        declaration.output_equivalence()
    else {
        return Ok(());
    };
    let Some(installed) = installed else {
        return Err(bridge_denial(format!(
            "registered output comparator `{}` has no bound host provider",
            required.as_str()
        )));
    };
    let provider = installed.provider();
    let installed_identity = provider.semantic_identity();
    if installed_identity != required.as_str() {
        return Err(bridge_denial(format!(
            "registered output comparator `{}` was bound to provider `{installed_identity}`",
            required.as_str()
        )));
    }
    Ok(())
}

fn dependency_candidates<Schema, ApplicationOperation, Input, D, O, F, Node>(
    node: &WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
    >,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<Vec<BridgeSemanticDependencyCandidate>, WorthQueryConditionalRuntimeInstallationDenial>
{
    let operation = node.operation().domain_operation();
    node.declaration()
        .dependencies()
        .iter()
        .enumerate()
        .map(|(ordinal, _)| {
            let authority = operation
                .conditional_dependency(node.location().clone(), ordinal)
                .map_err(|_| bridge_denial("installed conditional dependency was not retained"))?;
            candidate_from_authority(&authority, graph)
        })
        .collect()
}

fn candidate_from_authority(
    authority: &worth_query_installation::facade::WorthQueryInstalledConditionalDependencyAuthority,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<BridgeSemanticDependencyCandidate, WorthQueryConditionalRuntimeInstallationDenial> {
    let dependency = authority.dependency();
    if authority.runtime_ordinal() != graph.runtime_ordinal()
        || dependency.graph_read_role().as_str() != graph.role()
    {
        return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
            WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
            dependency.graph_read_role().as_str(),
        ));
    }
    let locality = match dependency.locality() {
        WorthQuerySemanticLocality::SourceRecord => BridgeSemanticLocality::ManagedSourceRecord,
        WorthQuerySemanticLocality::SourcePartition(role)
            if graph.truth_partition_role() == Some(role) =>
        {
            BridgeSemanticLocality::SourcePartition(role.clone())
        }
        WorthQuerySemanticLocality::SourcePartition(role) => {
            return Err(bridge_denial(format!(
                "primary application conditional runtime has no matching source-partition binding for `{}`",
                role.as_str()
            )))
        }
        WorthQuerySemanticLocality::WholeLogicalGraph => BridgeSemanticLocality::WholeLogicalGraph,
    };
    BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
        source_installation_identity: Arc::from(format!(
            "{}|generation={}|operation={}|node={}|dependency={}",
            authority.owner(),
            authority.generation().ordinal(),
            authority.operation_slot(),
            authority.location().node_identity(),
            authority.dependency_ordinal(),
        )),
        source_basis: Arc::from(authority.operation_canonical_identity()),
        source_runtime_authority: authority.runtime_ordinal(),
        source_installation_generation: authority.generation().ordinal(),
        source_authority_binding_identity: Arc::from(authority.authority_binding_identity()),
        source_stage_identity: authority.location().stage_identity().map(Arc::from),
        source_node_identity: Arc::from(authority.location().node_identity()),
        dependency_ordinal: authority.dependency_ordinal(),
        declared_graph_role: Arc::from(dependency.graph_read_role().as_str()),
        graph_participation_identity: Arc::from(graph.authority_identity()),
        graph_adapter_identity: Arc::from(graph.provider_identity()),
        source_record_identity: None,
        observation_record_identity: None,
        contract: dependency.contract().clone(),
        projection_mask: dependency.projection_mask().clone(),
        binding: dependency.binding().clone(),
        locality,
        relevant_changes: dependency.relevant_changes().to_vec(),
    })
    .map_err(|denial| bridge_denial(format!("semantic dependency denied: {:?}", denial.kind())))
}

fn bridge_denial(subject: impl Into<String>) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
        subject,
    )
}
