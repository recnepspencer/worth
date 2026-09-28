/// Why the runtime refused to select and admit a basis on a product branch.
///
/// Returned when a request, read, or mutation names a product branch that the
/// owning runtime cannot observe now. The refusal happens before any effect.
/// Use [`Self::is_transient`] to tell capacity, cancellation, deadline, and
/// availability causes (retry later) from a branch that is foreign, retired, or
/// rejected (choose another branch).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductBranchAdmissionDenial {
    /// The runtime's branch-observation state is unusable after an internal failure.
    ObservationStatePoisoned,
    /// The owner of the product branch is not available right now.
    OwnerUnavailable,
    /// The branch token or observation belongs to a different runtime.
    ForeignOwner,
    /// The product branch has been retired.
    RetiredBranch,
    /// The branch changed lifecycle incarnation between selection and admission.
    IncarnationChanged,
    /// The owner rejected the observation request itself.
    ObservationRejected,
    /// The branch's source head moved while it was being observed.
    ObservationStaleSourceHead,
    /// The observation was cancelled before any effect.
    ObservationCancelled,
    /// The observation reached its deadline before any effect.
    ObservationDeadlineExceeded,
    /// The owner has no capacity for another branch observation.
    ObservationCapacityExhausted,
    /// The owner has no capacity for another custody entry.
    CustodyCapacityExhausted,
    /// The owner ran out of observation identities.
    ObservationIdentityExhausted,
    /// The program activation for the branch could not be read.
    ProductActivationUnavailable,
    /// The Relational basis for the branch could not be retained.
    RelationalBasisUnavailable,
    /// A Relational snapshot for the branch could not be admitted.
    RelationalSnapshotUnavailable,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// No capacity remains to retain another basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The branch basis could not be retained for the Runtime Bridge.
    BridgeSourceUnavailable,
}

impl std::fmt::Display for WorthQueryProductBranchAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "product branch admission denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryProductBranchAdmissionDenial {}

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
