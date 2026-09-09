use worth_runtime_bridge::facade::{
    BridgeConditionalProviderSet, BridgeSemanticCorrespondenceRegistration,
    BridgeSignalAspectTargetDeclaration, RelationalBridgeRecordIdentityParts,
};

mod authority_resolution;
mod compute_contract;
mod owned_topology;

pub(crate) use compute_contract::WorthQueryConditionalComputeContextParts;
pub use compute_contract::{
    WorthQueryConditionalComputeContext, WorthQueryConditionalNodeComputeProvider,
};
pub(crate) use owned_topology::PendingOwnedConditionalNode;
pub use owned_topology::WorthQueryOwnedConditionalDependencyInstallation;

use super::QueryComputeProvider;
use authority_resolution::{installed_conditional_graph, installed_conditional_operation};

#[derive(Clone)]
pub struct WorthQueryConditionalDependencyInstallation {
    source_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    observation_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    targets: Vec<BridgeSignalAspectTargetDeclaration>,
}

impl WorthQueryConditionalDependencyInstallation {
    pub fn new(
        source_record_identity: Option<RelationalBridgeRecordIdentityParts>,
        targets: Vec<BridgeSignalAspectTargetDeclaration>,
    ) -> Self {
        Self {
            source_record_identity,
            observation_record_identity: source_record_identity,
            targets,
        }
    }

    pub fn with_observation_record(
        mut self,
        observation_record_identity: RelationalBridgeRecordIdentityParts,
    ) -> Self {
        self.observation_record_identity = Some(observation_record_identity);
        self
    }

    /// Read-only Signal targets retained by this dependency installation.
    /// These declarations carry no installed correspondence authority.
    #[doc(hidden)]
    pub fn signal_targets(&self) -> &[BridgeSignalAspectTargetDeclaration] {
        &self.targets
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQueryConditionalNodeInstallationDenial {
    DomainNotInstalled,
    OperationNotInstalled,
    GraphNotInstalled,
    LocationNotDeclared,
    DeclarationLookupDrift,
    DependencyShape,
    InvalidConditionalContract,
    UnsupportedMaintenancePosture,
    UnsupportedArtifactPosture,
    CompositeCorrespondenceRebindRequired,
    Correspondence(worth_runtime_bridge::facade::BridgeCorrespondenceDenialKind),
    Bridge {
        kind: worth_runtime_bridge::facade::BridgeConditionalDenialKind,
        detail: String,
    },
    DuplicateInstallation,
}

pub(crate) fn build_correspondence_registrations<D: 'static, O: 'static, F: 'static, G: 'static>(
    operation: &super::super::WorthQueryInstalledDomainOperation<D, O, F>,
    graph: &super::super::WorthQueryInstalledGraphParticipation<G>,
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    dependencies: Vec<WorthQueryConditionalDependencyInstallation>,
) -> Result<
    Vec<BridgeSemanticCorrespondenceRegistration>,
    WorthQueryConditionalNodeInstallationDenial,
> {
    let declaration = declared_node(operation.definition(), &location)
        .ok_or(WorthQueryConditionalNodeInstallationDenial::LocationNotDeclared)?;
    if dependencies.len() != declaration.dependencies().len() {
        return Err(WorthQueryConditionalNodeInstallationDenial::DependencyShape);
    }
    dependencies
        .into_iter()
        .enumerate()
        .map(|(ordinal, dependency)| {
            operation
                .semantic_correspondence_registration_with_observation(
                    location.clone(),
                    ordinal,
                    graph,
                    dependency.source_record_identity,
                    dependency.observation_record_identity,
                    dependency.targets,
                )
                .map_err(|denial| {
                    WorthQueryConditionalNodeInstallationDenial::Correspondence(denial.kind())
                })
        })
        .collect()
}

pub(super) fn declared_node<'a>(
    definition: &'a worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
) -> Option<&'a worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration> {
    match location {
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::Operation {
            ..
        } => definition
            .semantics()
            .conditional_nodes
            .iter()
            .find(|node| location_matches(location, node)),
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity,
            ..
        } => match &definition.semantics().workflow {
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::Declared(
                workflow,
            ) => workflow
                .stages()
                .iter()
                .find(|stage| stage.identity() == stage_identity)
                .and_then(|stage| {
                    stage
                        .semantics()
                        .conditional_nodes
                        .iter()
                        .find(|node| location_matches(location, node))
                }),
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::NotRequired => {
                None
            }
        },
    }
}

fn location_matches(
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    node: &worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
) -> bool {
    location.node_identity() == node.identity()
}

pub(super) fn with_compute_provider<D: 'static, O: 'static, F: 'static, P>(
    providers: BridgeConditionalProviderSet,
    provider: std::sync::Arc<P>,
) -> BridgeConditionalProviderSet
where
    P: WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    providers.compute(QueryComputeProvider::<D, O, F, P>::new(provider))
}

pub(crate) trait PendingConditionalInstallation: Send {
    fn requires_external_signal_graph(&self) -> bool {
        true
    }

    fn installed_node_count(&self) -> usize {
        1
    }

    fn install(
        &self,
        domains: &super::super::WorthQueryDomainInstallationRegistry,
        graphs: &super::super::WorthQueryInstalledGraphParticipationRegistry,
        signal: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        registry: &mut super::WorthQueryConditionalExecutionRegistry,
    ) -> Result<(), WorthQueryConditionalNodeInstallationDenial>;
}

pub(crate) struct PendingConditionalNode<D, O, F, G, P> {
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    dependencies: Vec<WorthQueryConditionalDependencyInstallation>,
    providers: BridgeConditionalProviderSet,
    compute: std::sync::Arc<P>,
    _marker: std::marker::PhantomData<fn() -> (D, O, F, G)>,
}

impl<D, O, F, G, P> PendingConditionalNode<D, O, F, G, P> {
    pub(crate) fn new(
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        dependencies: Vec<WorthQueryConditionalDependencyInstallation>,
        providers: BridgeConditionalProviderSet,
        compute: P,
    ) -> Self {
        Self {
            location,
            dependencies,
            providers,
            compute: std::sync::Arc::new(compute),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<D: 'static, O: 'static, F: 'static, G: 'static, P> PendingConditionalInstallation
    for PendingConditionalNode<D, O, F, G, P>
where
    P: WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    fn install(
        &self,
        domains: &super::super::WorthQueryDomainInstallationRegistry,
        graphs: &super::super::WorthQueryInstalledGraphParticipationRegistry,
        signal: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        registry: &mut super::WorthQueryConditionalExecutionRegistry,
    ) -> Result<(), WorthQueryConditionalNodeInstallationDenial> {
        let operation = installed_conditional_operation::<D, O, F>(domains)?;
        let graph = installed_conditional_graph::<G>(graphs)?;
        let request = self.installation_request(&operation, &graph)?;
        let lowering = signal.install(request).map_err(|denial| {
            WorthQueryConditionalNodeInstallationDenial::Bridge {
                kind: denial.kind(),
                detail: denial.detail().to_string(),
            }
        })?;
        let declaration = declared_node(operation.definition(), &self.location)
            .cloned()
            .ok_or(WorthQueryConditionalNodeInstallationDenial::DeclarationLookupDrift)?;
        registry
            .install::<D, O, F>(super::WorthQueryInstalledConditionalNode {
                lowering,
                location: self.location.clone(),
                declaration,
                graph_authority: std::sync::Arc::clone(&graph.record.installation_authority),
                operation_identity: operation.definition().canonical_identity().to_string(),
                runtime_authority: operation.domain_authority().runtime_authority().as_u64(),
                installation_runtime_authority: operation.operation_authority().runtime_ordinal(),
                source_installation_generation: operation
                    .operation_authority()
                    .generation()
                    .ordinal(),
                installation_generation: operation.installation_generation().ordinal(),
                resource_support: self.compute.execution_resource_support(),
            })
            .map_err(|_| WorthQueryConditionalNodeInstallationDenial::DuplicateInstallation)
    }
}

impl<D: 'static, O: 'static, F: 'static, G: 'static, P> PendingConditionalNode<D, O, F, G, P>
where
    P: WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    fn installation_request(
        &self,
        operation: &super::super::WorthQueryInstalledDomainOperation<D, O, F>,
        graph: &super::super::WorthQueryInstalledGraphParticipation<G>,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalInstallationRequest,
        WorthQueryConditionalNodeInstallationDenial,
    > {
        let declaration = declared_node(operation.definition(), &self.location)
            .cloned()
            .ok_or(WorthQueryConditionalNodeInstallationDenial::DeclarationLookupDrift)?;
        let contract = super::bridge_lowering::lower_bridge_contract(&declaration)?;
        let bridge_location = super::bridge_lowering::lower_bridge_location(&self.location);
        if self.providers.has_compute_provider() {
            return Err(WorthQueryConditionalNodeInstallationDenial::Bridge {
                kind:
                    worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExtraComputeProvider,
                detail: "Query conditional registrations own the sole compute provider".into(),
            });
        }
        let registrations = build_correspondence_registrations(
            operation,
            graph,
            self.location.clone(),
            self.dependencies.clone(),
        )?;
        Ok(
            worth_runtime_bridge::facade::BridgeConditionalInstallationRequest {
                contract,
                location: bridge_location,
                registrations,
                providers: with_compute_provider::<D, O, F, P>(
                    self.providers.clone(),
                    std::sync::Arc::clone(&self.compute),
                ),
            },
        )
    }
}
