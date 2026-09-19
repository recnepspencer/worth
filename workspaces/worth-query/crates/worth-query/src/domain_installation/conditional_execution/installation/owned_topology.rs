use worth_runtime_bridge::facade::{
    BridgeConditionalProviderSet, BridgeOwnedConditionalInstallationRequest,
    BridgeSemanticDependencyCandidate, RelationalBridgeRecordIdentityParts,
};

use super::authority_resolution::{installed_conditional_graph, installed_conditional_operation};
use super::{declared_node, with_compute_provider, PendingConditionalInstallation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOwnedConditionalDependencyInstallation {
    source_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    observation_record_identity: Option<RelationalBridgeRecordIdentityParts>,
}

impl WorthQueryOwnedConditionalDependencyInstallation {
    pub const fn new(source_record_identity: Option<RelationalBridgeRecordIdentityParts>) -> Self {
        Self {
            source_record_identity,
            observation_record_identity: source_record_identity,
        }
    }

    pub const fn with_observation_record(
        mut self,
        observation_record_identity: RelationalBridgeRecordIdentityParts,
    ) -> Self {
        self.observation_record_identity = Some(observation_record_identity);
        self
    }
}

pub(crate) struct PendingOwnedConditionalNode<D, O, F, G, P> {
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    dependencies: Vec<WorthQueryOwnedConditionalDependencyInstallation>,
    providers: BridgeConditionalProviderSet,
    compute: std::sync::Arc<P>,
    _marker: std::marker::PhantomData<fn() -> (D, O, F, G)>,
}

impl<D, O, F, G, P> PendingOwnedConditionalNode<D, O, F, G, P> {
    pub(crate) fn new(
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        dependencies: Vec<WorthQueryOwnedConditionalDependencyInstallation>,
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
    for PendingOwnedConditionalNode<D, O, F, G, P>
where
    P: super::super::WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    fn requires_external_signal_graph(&self) -> bool {
        false
    }

    fn install(
        &self,
        domains: &super::super::super::WorthQueryDomainInstallationRegistry,
        graphs: &super::super::super::WorthQueryInstalledGraphParticipationRegistry,
        signal: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        registry: &mut super::super::WorthQueryConditionalExecutionRegistry,
    ) -> Result<(), super::WorthQueryConditionalNodeInstallationDenial> {
        let node = self.install_node_with_builder(domains, graphs, signal)?;
        registry
            .install::<D, O, F>(node)
            .map_err(|_| super::WorthQueryConditionalNodeInstallationDenial::DuplicateInstallation)
    }
}

impl<D: 'static, O: 'static, F: 'static, G: 'static, P> PendingOwnedConditionalNode<D, O, F, G, P>
where
    P: super::super::WorthQueryConditionalNodeComputeProvider<D, O, F>,
{
    fn install_node_with_builder(
        &self,
        domains: &super::super::super::WorthQueryDomainInstallationRegistry,
        graphs: &super::super::super::WorthQueryInstalledGraphParticipationRegistry,
        signal: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
    ) -> Result<
        super::super::WorthQueryInstalledConditionalNode,
        super::WorthQueryConditionalNodeInstallationDenial,
    > {
        self.install_node_with(domains, graphs, |request| {
            signal.install_owned_conditional(request)
        })
    }

    fn install_node_with(
        &self,
        domains: &super::super::super::WorthQueryDomainInstallationRegistry,
        graphs: &super::super::super::WorthQueryInstalledGraphParticipationRegistry,
        install: impl FnOnce(
            BridgeOwnedConditionalInstallationRequest,
        ) -> Result<
            std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
            worth_runtime_bridge::facade::BridgeConditionalDenial,
        >,
    ) -> Result<
        super::super::WorthQueryInstalledConditionalNode,
        super::WorthQueryConditionalNodeInstallationDenial,
    > {
        let operation = installed_conditional_operation::<D, O, F>(domains)?;
        let graph = installed_conditional_graph::<G>(graphs)?;
        let declaration = declared_node(operation.definition(), &self.location)
            .cloned()
            .ok_or(super::WorthQueryConditionalNodeInstallationDenial::DeclarationLookupDrift)?;
        if declaration.dependencies().len() != self.dependencies.len() {
            return Err(super::WorthQueryConditionalNodeInstallationDenial::DependencyShape);
        }
        let dependencies = self
            .dependencies
            .iter()
            .enumerate()
            .map(|(ordinal, dependency)| {
                operation.semantic_correspondence_candidate_with_observation(
                    self.location.clone(),
                    ordinal,
                    &graph,
                    dependency.source_record_identity,
                    dependency.observation_record_identity,
                )
            })
            .collect::<Result<Vec<BridgeSemanticDependencyCandidate>, _>>()
            .map_err(|denial| {
                super::WorthQueryConditionalNodeInstallationDenial::Correspondence(denial.kind())
            })?;
        if self.providers.has_compute_provider() {
            return Err(super::WorthQueryConditionalNodeInstallationDenial::Bridge {
                kind:
                    worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExtraComputeProvider,
                detail: "Query conditional registrations own the sole compute provider".into(),
            });
        }
        let lowering = install(BridgeOwnedConditionalInstallationRequest {
            contract: super::super::bridge_lowering::lower_bridge_contract(&declaration)?,
            location: super::super::bridge_lowering::lower_bridge_location(&self.location),
            dependencies,
            providers: with_compute_provider::<D, O, F, P>(
                self.providers.clone(),
                std::sync::Arc::clone(&self.compute),
            ),
        })
        .map_err(
            |denial| super::WorthQueryConditionalNodeInstallationDenial::Bridge {
                kind: denial.kind(),
                detail: denial.detail().to_string(),
            },
        )?;
        Ok(super::super::WorthQueryInstalledConditionalNode {
            lowering,
            location: self.location.clone(),
            declaration,
            graph_authority: std::sync::Arc::clone(&graph.record.installation_authority),
            operation_identity: operation.definition().canonical_identity().to_string(),
            runtime_authority: operation.domain_authority().runtime_authority().as_u64(),
            installation_runtime_authority: operation.operation_authority().runtime_ordinal(),
            source_installation_generation: operation.operation_authority().generation().ordinal(),
            installation_generation: operation.installation_generation().ordinal(),
            resource_support: self.compute.execution_resource_support(),
        })
    }
}
