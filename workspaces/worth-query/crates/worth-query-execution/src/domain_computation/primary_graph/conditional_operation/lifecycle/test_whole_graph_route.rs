//! A test-only installed route exercising the real conditional commit journal.

use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

use super::{
    ConditionalClockLease, ErasedClockObservationOutcome, WorthQueryConditionalOperationRegistry,
    WorthQueryConditionalRetainedResourceCounts, WorthQueryConditionalTruthBasis,
    WorthQueryInstalledConditionalOperation, WorthQueryPreparedConditionalRuntimeBinding,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::conditional_operation::{
    installation::WorthQueryConditionalRuntimeInstallationDenial,
    lifecycle_inventory::WorthQueryConditionalOperationLiveness,
    temporal_reconstruction::WorthQueryTemporalReconstructionWork,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

struct WholeGraphRoute<Schema> {
    lease: Arc<ConditionalClockLease>,
    marker: std::marker::PhantomData<fn() -> Schema>,
}

pub(in crate::domain_computation::primary_graph) fn install_test_whole_graph_route<
    Schema: 'static,
>(
    registry: &mut WorthQueryConditionalOperationRegistry<Schema>,
) {
    registry
        .install(Box::new(WholeGraphRoute {
            lease: Arc::new(ConditionalClockLease),
            marker: std::marker::PhantomData,
        }))
        .expect("test whole-graph route identity is unique");
}

impl<Schema> WorthQueryInstalledConditionalOperation<Schema> for WholeGraphRoute<Schema> {
    fn binding_identity(&self) -> &str {
        "inbound-completion-whole-graph-court"
    }

    fn installation_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero()
    }

    fn clock_lease(&self) -> Arc<ConditionalClockLease> {
        Arc::clone(&self.lease)
    }

    fn operation_anchor(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        unreachable!("the delivery court does not evaluate the clock")
    }

    fn selected_lowering(
        &self,
    ) -> Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering> {
        unreachable!("the delivery court does not directly deliver")
    }

    fn select_product_binding(
        &mut self,
        _: &BridgeSealedRuntimeAssembly,
        _: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn reconstruct(
        &mut self,
        _: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn authoritative_commit_routes(
        &self,
    ) -> (Vec<worth_relational::facade::transactions::RecordRef>, bool) {
        (Vec::new(), true)
    }

    fn reconcile_reconstruction(
        &mut self,
        _: &mut BridgeSealedRuntimeAssembly,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn prepare_derived_runtime_reinstallation(
        &self,
        _: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        _: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryPreparedConditionalRuntimeBinding,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        unreachable!("the delivery court does not rebind")
    }

    fn apply_derived_runtime_reinstallation(
        &mut self,
        _: WorthQueryPreparedConditionalRuntimeBinding,
    ) {
        unreachable!("the delivery court does not rebind")
    }

    fn reconcile_prepared_runtime_reinstallation(
        &self,
        _: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        _: &mut WorthQueryPreparedConditionalRuntimeBinding,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        Ok(())
    }

    fn observe_clock(
        &mut self,
        _: &BridgeSealedRuntimeAssembly,
        _: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _: &WorthQueryConditionalTruthBasis,
    ) -> ErasedClockObservationOutcome {
        ErasedClockObservationOutcome::Stale
    }

    fn retained_resource_counts(&self) -> WorthQueryConditionalRetainedResourceCounts {
        Default::default()
    }

    fn reconstruction_work(&self) -> WorthQueryTemporalReconstructionWork {
        Default::default()
    }

    fn lifecycle_resources(&self) -> WorthQueryConditionalOperationLiveness {
        WorthQueryConditionalOperationLiveness {
            binding: Default::default(),
            lease: Default::default(),
            wakes: Vec::new(),
            intents: Vec::new(),
            attempts: Vec::new(),
        }
    }
}
