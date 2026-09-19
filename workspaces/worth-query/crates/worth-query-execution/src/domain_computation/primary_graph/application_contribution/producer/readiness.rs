use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationConditionalNode,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalExecutionRequest, BridgeConditionalRuntimeBuilder,
    BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};

use crate::domain_computation::primary_graph::conditional_operation::QueryConditionalComputeContext;
use crate::domain_computation::primary_graph::conditional_operation::{
    prepare_dependency_conditional_installation, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

pub(in crate::domain_computation::primary_graph) trait PendingOutputReadiness<Schema> {
    fn producer_identity(&self) -> &str;

    fn install(
        self: Box<Self>,
        bridge: &mut BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<
        WorthQueryInstalledOutputReadinessRoute,
        WorthQueryConditionalRuntimeInstallationDenial,
    >;
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryInstalledOutputReadinessRoute {
    pub(in crate::domain_computation::primary_graph) lowering:
        Arc<BridgeInstalledConditionalLowering>,
    pub(in crate::domain_computation::primary_graph) delivery_dependency_ordinal: usize,
}

pub(in crate::domain_computation::primary_graph) type WorthQueryInstalledOutputReadinessRoutes =
    std::collections::BTreeMap<String, WorthQueryInstalledOutputReadinessRoute>;

pub(in crate::domain_computation::primary_graph) fn evaluate_output_readiness(
    bridge: &BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    truth: &crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis,
    producer_identity: &str,
    source_record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    query_identity: u64,
    attempt: u64,
) -> Result<
    worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    worth_runtime_bridge::facade::BridgeConditionalDenial,
> {
    let signal_basis =
        bridge.admit_exact_conditional_signal_basis(lowering, truth.signal_basis())?;
    let mut compute = QueryConditionalComputeContext {
        output_version: attempt,
    };
    bridge.execute_for_source_record(
        &signal_basis,
        BridgeConditionalExecutionRequest {
            lowering,
            query_binding_identity: producer_identity,
            query_capability_identity: query_identity,
            snapshot_identity: truth.snapshot_projection(),
            truth_branch_identity: Some(truth.branch_projection()),
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
    pub(in crate::domain_computation::primary_graph) fn contains_producer<Binding>(&self) -> bool
    where
        Binding: super::WorthQueryApplicationProducerBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .is_some_and(|entry| entry.declaration.meaning_matches::<Schema, Binding>())
    }

    pub(in crate::domain_computation::primary_graph) fn validate_readiness_routes(
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

pub(in crate::domain_computation::primary_graph) fn install_output_readiness_routes<Schema>(
    pending: Vec<Box<dyn PendingOutputReadiness<Schema>>>,
    bridge: &mut BridgeConditionalRuntimeBuilder,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) -> Result<WorthQueryInstalledOutputReadinessRoutes, WorthQueryConditionalRuntimeInstallationDenial>
{
    let mut installed = WorthQueryInstalledOutputReadinessRoutes::new();
    for route in pending {
        let identity = route.producer_identity().to_owned();
        let lowering = route.install(bridge, graph)?;
        if installed.insert(identity.clone(), lowering).is_some() {
            return Err(WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::DuplicateBinding,
                identity,
            ));
        }
    }
    Ok(installed)
}

pub(in crate::domain_computation::primary_graph) struct TypedPendingOutputReadiness<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
> {
    producer_identity: String,
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
}

impl<Schema, ApplicationOperation, Input, D, O, F, Node>
    TypedPendingOutputReadiness<Schema, ApplicationOperation, Input, D, O, F, Node>
{
    pub(in crate::domain_computation::primary_graph) fn new(
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
    ) -> Self {
        Self {
            producer_identity: producer_identity.into(),
            delivery_dependency_ordinal,
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
        self: Box<Self>,
        bridge: &mut BridgeConditionalRuntimeBuilder,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<
        WorthQueryInstalledOutputReadinessRoute,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let request = prepare_dependency_conditional_installation(&self.node, graph)?;
        let lowering = bridge
            .install_owned_conditional(request)
            .map_err(|denial| {
                WorthQueryConditionalRuntimeInstallationDenial::new(
                    WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                    format!("{:?}: {}", denial.kind(), denial.detail()),
                )
            })?;
        Ok(WorthQueryInstalledOutputReadinessRoute {
            lowering,
            delivery_dependency_ordinal: self.delivery_dependency_ordinal,
        })
    }
}
