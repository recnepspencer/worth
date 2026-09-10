#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchAdmissionDenial {
    ObservationStatePoisoned,
    OwnerUnavailable,
    ForeignOwner,
    RetiredBranch,
    IncarnationChanged,
    ObservationRejected,
    ProductActivationUnavailable,
    RelationalBasisUnavailable,
    RelationalSnapshotUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgeSourceUnavailable,
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
