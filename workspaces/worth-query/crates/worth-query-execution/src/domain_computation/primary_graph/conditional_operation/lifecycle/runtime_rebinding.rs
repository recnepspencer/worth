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
            WorthQueryConditionalRuntimeInstallationDenialKind::BridgeConditional(Box::new(
                denial.kind(),
            )),
            format!("{:?}: {}", denial.kind(), denial.detail()),
        )
    })?;
    let clock = binding.clocked_node();
    let signal_basis = bridge
        .admit_exact_conditional_signal_basis(&lowering, product.signal_basis())
        .map_err(|denial| {
            WorthQueryConditionalRuntimeInstallationDenial::new(
                WorthQueryConditionalRuntimeInstallationDenialKind::BridgeConditional(Box::new(
                    denial.kind(),
                )),
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
                WorthQueryConditionalRuntimeInstallationDenialKind::ManagedTemporal(denial.kind()),
                denial.detail(),
            )
        })?;
    Ok(WorthQueryPreparedConditionalRuntimeBinding {
        pending_invalidations:
            super::pending_invalidations::WorthQueryPendingGranularInvalidations::empty(),
        lowering,
        managed_clock,
        affinity: super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity::new(
            product,
            signal_basis,
        ),
        authoritative_commit_cursor: None,
        commit_watch: Default::default(),
        reconstructed_intent_count: 0,
        authoritative_reconstruction: Box::new(()),
    })
}

/// Installs the prepared binding while retaining independent native cleanup evidence.
pub(super) fn apply_temporal_runtime_binding<B, R, E, Clock: 'static, Input: 'static>(
    operation: &mut super::WorthQueryInstalledTemporalOperation<B, R, E, Clock, Input>,
    prepared: WorthQueryPreparedConditionalRuntimeBinding,
) {
    operation.pending_invalidations = prepared.pending_invalidations;
    operation.bootstrap_lowering = prepared.lowering;
    operation.active_affinity = Some(prepared.affinity);
    operation.managed_clock = Some(prepared.managed_clock);
    operation.authoritative_commit_cursor = prepared.authoritative_commit_cursor;
    operation.bootstrap_commit_catch_up_pending = false;
    operation.commit_watch = prepared.commit_watch;
    let reconstruction = *prepared
            .authoritative_reconstruction
            .downcast::<super::super::temporal_reconstruction::WorthQueryTemporalReconstruction<Clock, Input>>()
            .expect("prepared temporal reconstruction retains its installed binding type");
    operation.reconstructed_intents = reconstruction.intents;
    operation.reconstruction_work = reconstruction.work;
    super::super::retained_managed_cleanup::retain_cleanup_wakes(&mut operation.retained_wakes);
    operation.inactive_bindings.retain(|_, binding| {
        super::super::retained_managed_cleanup::has_cleanup_wakes(&binding.retained_wakes)
    });
}
