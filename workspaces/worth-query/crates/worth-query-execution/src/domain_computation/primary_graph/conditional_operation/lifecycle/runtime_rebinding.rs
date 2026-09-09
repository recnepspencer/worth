use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
    WorthQueryInstalledTemporalConditionalOperation, WorthQueryNamedClock,
    WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
};
use worth_runtime_bridge::facade::{
    BridgeManagedClockInstallationParts, BridgePreparedConditionalReconstitution,
};

use super::WorthQueryPreparedConditionalRuntimeBinding;
use crate::domain_computation::primary_graph::conditional_operation::installation::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_temporal_runtime_binding<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node: 'static,
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
    bridge: &mut BridgePreparedConditionalReconstitution,
    predecessor: &Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    runtime_binding_identity: Arc<str>,
    product: &crate::basis::WorthQueryProductBranchLease,
) -> Result<
    WorthQueryPreparedConditionalRuntimeBinding,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
{
    let bounds = binding.bounds();
    let lowering = bridge.readmit_lowering(predecessor).map_err(|denial| {
        WorthQueryConditionalRuntimeInstallationDenial::new(
            WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
            format!("{:?}: {}", denial.kind(), denial.detail()),
        )
    })?;
    let clock = binding.clocked_node();
    let signal_basis = bridge
        .admit_exact_conditional_signal_basis(&lowering, product.signal_basis())
        .map_err(|denial| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                denial.detail(),
            )
        })?;
    let managed_clock = bridge
        .install_managed_clock(BridgeManagedClockInstallationParts {
            lowering: &lowering,
            binding_identity: Arc::clone(&runtime_binding_identity),
            source_identity: Arc::from(clock.source_identity().as_str()),
            timeline_identity: Arc::from(clock.timeline_identity().as_str()),
            maximum_active_intents: bounds.maximum_reconstruction_rows(),
            maximum_due_wakes_per_observation: bounds.maximum_due_wakes_per_observation(),
        })
        .map_err(|denial| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
                denial.detail(),
            )
        })?;
    Ok(WorthQueryPreparedConditionalRuntimeBinding {
        pending_direct_delivery: super::direct_delivery::empty(),
        lowering,
        managed_clock,
        affinity: super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity::new(
            product,
            signal_basis,
        ),
        authoritative_commit_cursor: 0,
        commit_watch: Default::default(),
        reconstructed_intent_count: 0,
        authoritative_reconstruction: Box::new(()),
    })
}
