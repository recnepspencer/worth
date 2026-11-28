use crate::domain_computation::primary_graph::{
    conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
    conditional_operation::installation::route_binding::InstalledConditionalRoute,
    conditional_operation::installation::ConditionalClockLease,
    managed_bridge::conditional_execution::ConditionalExecutionOwner,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationConditionalNode,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalExecutionRequest, BridgeConditionalProviderSet,
    BridgeConditionalRuntimeBuilder,
};

use crate::domain_computation::primary_graph::conditional_operation::QueryConditionalComputeContext;
use crate::domain_computation::primary_graph::conditional_operation::{
    prepare_dependency_conditional_installation, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

pub trait PendingOutputReadiness<Schema> {
    fn producer_identity(&self) -> &str;

    fn install(
        &self,
        bridge: &mut BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        definitions: &mut ConditionalDefinitionDeclarations,
    ) -> Result<
        WorthQueryInstalledOutputReadinessRoute,
        WorthQueryConditionalRuntimeInstallationDenial,
    >;
}

pub struct WorthQueryInstalledOutputReadinessRoute {
    pub lowering: InstalledConditionalRoute,
    pub delivery_dependency_ordinal: usize,
}

pub type WorthQueryInstalledOutputReadinessRoutes =
    std::collections::BTreeMap<String, WorthQueryInstalledOutputReadinessRoute>;

pub fn evaluate_output_readiness(
    execution: worth_execution::ExecutionRequest<'_, '_>,

    bridge: &ConditionalExecutionOwner<'_>,
    lowering: &InstalledConditionalRoute,
    truth: &crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis,
    producer_identity: &str,
    source_record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    query_identity: u64,
    attempt: u64,
) -> Result<
    worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    worth_runtime_bridge::facade::BridgeConditionalDenial,
> {
    let signal_basis = lowering.admit_signal_basis(bridge, truth.signal_basis())?;
    let mut compute = QueryConditionalComputeContext::exact(attempt);
    bridge.execute_for_source_record(
        execution,
        &signal_basis,
        BridgeConditionalExecutionRequest {
            lowering: signal_basis.installed_lowering_ref(),
            query_binding_identity: producer_identity,
            query_capability_identity: query_identity,
            snapshot_identity: &truth.snapshot_projection(),
            truth_branch_identity: Some(&truth.branch_projection()),
            bridge_snapshot_identity: Some(truth.snapshot()),
            execution_identity: producer_identity,
            attempt,
        },
        source_record,
        &mut compute,
    )
}

impl<Schema> super::WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn contains_producer<Binding>(&self) -> bool
    where
        Binding: super::WorthQueryApplicationProducerBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .is_some_and(|entry| entry.declaration.meaning_matches::<Schema, Binding>())
    }

    pub fn validate_readiness_routes(
        &self,
        routes: &WorthQueryInstalledOutputReadinessRoutes,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        if self.entries.len() != routes.len()
            || self
                .entries
                .keys()
                .any(|producer| !routes.contains_key(producer))
        {
            return Err(crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial::new(
                crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::MissingConditionalBinding,
                "every installed output producer requires one exact readiness conditional",
            ));
        }
        Ok(())
    }
}

pub fn install_output_readiness_routes<Schema>(
    pending: &[Box<dyn PendingOutputReadiness<Schema>>],
    bridge: &mut BridgeConditionalRuntimeBuilder,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    definitions: &mut ConditionalDefinitionDeclarations,
) -> Result<WorthQueryInstalledOutputReadinessRoutes, WorthQueryConditionalRuntimeInstallationDenial>
{
    let mut installed = WorthQueryInstalledOutputReadinessRoutes::new();
    for route in pending {
        let identity = route.producer_identity().to_owned();
        let lowering = route.install(bridge, graph, definitions)?;
        if installed.insert(identity.clone(), lowering).is_some() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::DuplicateBinding,
                identity,
            ));
        }
    }
    Ok(installed)
}

pub struct TypedPendingOutputReadiness<Schema, ApplicationOperation, Input, D, O, F, Node> {
    producer_identity: String,
    delivery_dependency_ordinal: usize,
    definition: Option<(
        std::sync::Arc<str>,
        std::sync::Arc<ConditionalClockLease>,
        BridgeConditionalProviderSet,
    )>,
    node: WorthQueryInstalledApplicationConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
    >,
}

impl<Schema, ApplicationOperation, Input, D, O, F, Node>
    TypedPendingOutputReadiness<Schema, ApplicationOperation, Input, D, O, F, Node>
{
    pub fn new(
        producer_identity: impl Into<String>,
        delivery_dependency_ordinal: usize,
        node: WorthQueryInstalledApplicationConditionalNode<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
        >,
        definition: Option<(
            std::sync::Arc<str>,
            std::sync::Arc<ConditionalClockLease>,
            BridgeConditionalProviderSet,
        )>,
    ) -> Self {
        Self {
            producer_identity: producer_identity.into(),
            delivery_dependency_ordinal,
            definition,
            node,
        }
    }
}

impl<Schema, ApplicationOperation, Input, D, O, F, Node> PendingOutputReadiness<Schema>
    for TypedPendingOutputReadiness<Schema, ApplicationOperation, Input, D, O, F, Node>
where
    Schema: ApplicationSchema + 'static,
    ApplicationOperation: 'static,
    Input: 'static,
    D: 'static,
    O: 'static,
    F: 'static,
    Node: 'static,
{
    fn producer_identity(&self) -> &str {
        &self.producer_identity
    }

    fn install(
        &self,
        bridge: &mut BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        definitions: &mut ConditionalDefinitionDeclarations,
    ) -> Result<
        WorthQueryInstalledOutputReadinessRoute,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let mut request = prepare_dependency_conditional_installation(&self.node, graph)?;
        if let Some((_, _, providers)) = &self.definition {
            request.providers = providers.clone();
        }
        let lowering = bridge
            .install_owned_conditional(request)
            .map_err(|denial| {
                WorthQueryConditionalRuntimeInstallationDenial::new(
                    WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    format!("{:?}: {}", denial.kind(), denial.detail()),
                )
            })?;
        let lowering = InstalledConditionalRoute::from_installation(lowering);
        if let Some((identity, lease, _)) = &self.definition {
            definitions.install_readiness(identity.clone(), lease.clone(), lowering.clone());
        }
        Ok(WorthQueryInstalledOutputReadinessRoute {
            lowering,
            delivery_dependency_ordinal: self.delivery_dependency_ordinal,
        })
    }
}
