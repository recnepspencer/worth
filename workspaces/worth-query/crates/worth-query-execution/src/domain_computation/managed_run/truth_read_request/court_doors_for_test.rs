//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryManagedTruthReadRequest {
    #[doc(hidden)]
    pub fn from_relational_basis_for_test(
        relational_basis: RelationalBranchBasisDescriptor,
        packet: SnapshotReadPacket,
    ) -> Self {
        Self::from_relational_basis(relational_basis, packet)
    }
}

impl WorthQueryManagedTruthReadRequest {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        RelationalBranchBasisDescriptor,
        SnapshotReadPacket,
        BridgeReplayMode,
        BridgeDiagnosticsTier,
        BridgeDeliveryIntent,
        Option<worth_runtime_world::facade::ProductBranchObservation>,
    ) {
        Self::into_parts(self)
    }
}
