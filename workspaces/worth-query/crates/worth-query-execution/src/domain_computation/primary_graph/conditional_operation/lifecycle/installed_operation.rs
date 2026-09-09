use std::{collections::BTreeMap, sync::Arc};

use worth_runtime_bridge::facade::{
    BridgeInstalledConditionalLowering, BridgeManagedClockBinding, BridgeSealedRuntimeAssembly,
};

use super::{
    ConditionalClockLease, ErasedClockObservationOutcome, WorthQueryConditionalTruthBasis,
};
use crate::domain_computation::primary_graph::conditional_operation::{
    installation::WorthQueryConditionalRuntimeInstallationDenial,
    signal_decision_reentry::WorthQueryRetainedConditionalWake,
    temporal_reconstruction::WorthQueryReconstructedTemporalIntent,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation::primary_graph) trait WorthQueryInstalledConditionalOperation<
    Schema,
>: Send
{
    fn binding_identity(&self) -> &str;

    fn installation_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

    fn clock_lease(&self) -> Arc<ConditionalClockLease>;

    fn operation_anchor(&self) -> Arc<BridgeInstalledConditionalLowering>;

    fn selected_lowering(&self) -> Arc<BridgeInstalledConditionalLowering>;

    fn retain_direct_delivery(
        &mut self,
        _receipt: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    ) {
        unreachable!("conditional operation did not admit direct delivery retention")
    }

    fn select_product_binding(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>;

    fn reconstruct(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>;

    fn authoritative_commit_routes(
        &self,
    ) -> (Vec<worth_relational::facade::transactions::RecordRef>, bool) {
        (Vec::new(), false)
    }

    fn bootstrap_commit_route_pending(&self) -> bool {
        false
    }

    fn reconcile_reconstruction(
        &mut self,
        bridge: &mut BridgeSealedRuntimeAssembly,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>;

    fn prepare_derived_runtime_reinstallation(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryPreparedConditionalRuntimeBinding,
        WorthQueryConditionalRuntimeInstallationDenial,
    >;

    fn reconcile_prepared_runtime_reinstallation(
        &self,
        bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        prepared: &mut WorthQueryPreparedConditionalRuntimeBinding,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>;

    fn apply_derived_runtime_reinstallation(
        &mut self,
        prepared: WorthQueryPreparedConditionalRuntimeBinding,
    );

    fn observe_clock(
        &mut self,
        bridge: &BridgeSealedRuntimeAssembly,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        truth: &WorthQueryConditionalTruthBasis,
    ) -> ErasedClockObservationOutcome;

    fn retained_resource_counts(&self) -> WorthQueryConditionalRetainedResourceCounts;

    fn reconstruction_work(
        &self,
    ) -> super::super::temporal_reconstruction::WorthQueryTemporalReconstructionWork;

    fn lifecycle_resources(
        &self,
    ) -> super::super::lifecycle_inventory::WorthQueryConditionalOperationLiveness;
}

#[derive(Clone, Copy, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryConditionalRetainedResourceCounts
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) wakes: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) intents: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) attempts: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) direct_deliveries:
        usize,
}

impl WorthQueryConditionalRetainedResourceCounts {
    pub(super) fn add(&mut self, other: Self) {
        self.wakes += other.wakes;
        self.intents += other.intents;
        self.attempts += other.attempts;
        self.direct_deliveries += other.direct_deliveries;
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryPreparedConditionalRuntimeBinding
{
    pub(super) pending_direct_delivery: super::direct_delivery::WorthQueryPendingDirectDelivery,
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
    pub(super) managed_clock: BridgeManagedClockBinding,
    pub(super) affinity: super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity,
    pub(super) authoritative_commit_cursor: u64,
    pub(super) commit_watch: super::commit_watch::WorthQueryConditionalCommitWatchSet,
    pub(in crate::domain_computation::primary_graph::conditional_operation) reconstructed_intent_count:
        usize,
    pub(super) authoritative_reconstruction: Box<dyn std::any::Any + Send>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryInstalledTemporalOperation<
    Binding,
    Reconstruction,
    Execution,
    Clock,
    Input,
> {
    pub(in crate::domain_computation::primary_graph::conditional_operation) lifecycle_token:
        Arc<()>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) definition:
        Arc<super::super::definition::WorthQueryTemporalOperationDefinition<Binding, Reconstruction, Execution>>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) bootstrap_lowering:
        Arc<BridgeInstalledConditionalLowering>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) active_affinity:
        Option<super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) managed_clock:
        Option<BridgeManagedClockBinding>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) runtime_binding_identity:
        Arc<str>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) runtime_canonical_identity:
        Arc<crate::domain_computation::primary_graph::conditional_operation::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) runtime_capability_identity:
        u64,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_wakes:
        Vec<WorthQueryRetainedConditionalWake>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) reconstructed_intents:
        BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) reconstruction_work:
        crate::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryTemporalReconstructionWork,
    pub(in crate::domain_computation::primary_graph::conditional_operation) authoritative_commit_cursor:
        u64,
    pub(in crate::domain_computation::primary_graph::conditional_operation) bootstrap_commit_catch_up_pending:
        bool,
    pub(in crate::domain_computation::primary_graph::conditional_operation) commit_watch:
        super::commit_watch::WorthQueryConditionalCommitWatchSet,
    pub(in crate::domain_computation::primary_graph::conditional_operation) operation_totals:
        super::operation_totals::WorthQueryTemporalOperationTotals,
    pub(in crate::domain_computation::primary_graph::conditional_operation) pending_direct_delivery:
        super::direct_delivery::WorthQueryPendingDirectDelivery,
    pub(in crate::domain_computation::primary_graph::conditional_operation) inactive_bindings:
        BTreeMap<
            crate::basis::WorthQueryProductBranchReadIdentity,
            super::evaluation_binding::WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>,
        >,
    pub(in crate::domain_computation::primary_graph::conditional_operation) next_evaluation_binding_ordinal:
        u64,
}
