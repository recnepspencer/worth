use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_runtime_bridge::facade::{
    BridgeDeliveryIntent, BridgeDiagnosticsTier, BridgeReplayMode, SnapshotReadPacket,
};

pub struct WorthQueryManagedTruthReadRequest {
    relational_basis: RelationalBranchBasisDescriptor,
    packet: SnapshotReadPacket,
    replay_mode: BridgeReplayMode,
    diagnostics_tier: BridgeDiagnosticsTier,
    delivery_intent: BridgeDeliveryIntent,
    product_observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

impl WorthQueryManagedTruthReadRequest {
    pub fn from_relational_basis(
        relational_basis: RelationalBranchBasisDescriptor,
        packet: SnapshotReadPacket,
    ) -> Self {
        Self {
            relational_basis,
            packet,
            replay_mode: BridgeReplayMode::Disabled,
            diagnostics_tier: BridgeDiagnosticsTier::Standard,
            delivery_intent: BridgeDeliveryIntent::PrepareSignalEvaluation,
            product_observation: None,
        }
    }

    pub fn for_product(
        product: &crate::basis::WorthQueryProductBranchLease,
        packet: SnapshotReadPacket,
    ) -> Self {
        Self {
            relational_basis: product.relational_basis_descriptor().clone(),
            packet,
            replay_mode: BridgeReplayMode::Disabled,
            diagnostics_tier: BridgeDiagnosticsTier::Standard,
            delivery_intent: BridgeDeliveryIntent::PrepareSignalEvaluation,
            product_observation: Some(product.observation().clone()),
        }
    }

    pub fn with_replay_mode(mut self, replay_mode: BridgeReplayMode) -> Self {
        self.replay_mode = replay_mode;
        self
    }

    pub fn with_diagnostics_tier(mut self, diagnostics_tier: BridgeDiagnosticsTier) -> Self {
        self.diagnostics_tier = diagnostics_tier;
        self
    }

    pub fn with_delivery_intent(mut self, delivery_intent: BridgeDeliveryIntent) -> Self {
        self.delivery_intent = delivery_intent;
        self
    }

    pub fn into_parts(
        self,
    ) -> (
        RelationalBranchBasisDescriptor,
        SnapshotReadPacket,
        BridgeReplayMode,
        BridgeDiagnosticsTier,
        BridgeDeliveryIntent,
        Option<worth_runtime_world::facade::ProductBranchObservation>,
    ) {
        (
            self.relational_basis,
            self.packet,
            self.replay_mode,
            self.diagnostics_tier,
            self.delivery_intent,
            self.product_observation,
        )
    }

    pub(crate) fn matches_operation_product(
        &self,
        operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        match (
            operation.application_product_observation(),
            self.product_observation.as_ref(),
        ) {
            (Some(expected), Some(selected)) => expected == selected,
            (None, None) => true,
            _ => false,
        }
    }
}
