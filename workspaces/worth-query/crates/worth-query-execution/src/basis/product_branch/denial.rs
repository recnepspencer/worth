#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchAdmissionDenial {
    ObservationStatePoisoned,
    OwnerUnavailable,
    ForeignOwner,
    RetiredBranch,
    IncarnationChanged,
    ObservationRejected,
    ObservationStaleSourceHead,
    ObservationCancelled,
    ObservationDeadlineExceeded,
    ObservationCapacityExhausted,
    CustodyCapacityExhausted,
    ObservationIdentityExhausted,
    ProductActivationUnavailable,
    RelationalBasisUnavailable,
    RelationalSnapshotUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgeSourceUnavailable,
}

impl WorthQueryProductBranchAdmissionDenial {
    pub const fn is_transient(self) -> bool {
        matches!(
            self,
            Self::OwnerUnavailable
                | Self::ObservationCancelled
                | Self::ObservationDeadlineExceeded
                | Self::ObservationCapacityExhausted
                | Self::CustodyCapacityExhausted
                | Self::ObservationStaleSourceHead
                | Self::ProductActivationUnavailable
                | Self::RelationalBasisUnavailable
                | Self::RelationalSnapshotUnavailable
                | Self::ActiveSnapshotCapacityExhausted { .. }
                | Self::RetentionCapacityExhausted
                | Self::BridgeSourceUnavailable
        )
    }
}

impl From<crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial>
    for WorthQueryProductBranchAdmissionDenial
{
    fn from(
        _denial: crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial,
    ) -> Self {
        Self::ProductActivationUnavailable
    }
}
