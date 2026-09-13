/// Why at least one owner effect survived without a product-reference move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductUnpublishedCause {
    SiblingOwnerDenied,
    CallerAbandoned,
    SettlementPending,
    CancellationAfterEffect,
    DeadlineAfterEffect,
    StaleProductHead,
    OwnerLost,
    CorrespondenceRebindRequired,
    /// A performed fork could not admit its destination observation or protection.
    DestinationAdmissionDenied,
    RetentionAdmissionDenied,
    ProductPublicationLost,
}

impl ProductUnpublishedCause {
    /// Capacity, affinity, and arithmetic failures do not establish owner loss.
    pub(crate) fn from_retention_denial(
        denial: &crate::retention::RetentionObligationDenial,
    ) -> Self {
        use crate::retention::RetentionObligationDenial;
        use worth_relational::facade::branch::RelationalBranchBasisDenial;
        use worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial;
        match denial {
            RetentionObligationDenial::Relational(
                RelationalBranchBasisDenial::OwnerUnavailable,
            )
            | RetentionObligationDenial::Signal(
                SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_),
            ) => Self::OwnerLost,
            _ => Self::RetentionAdmissionDenied,
        }
    }
}
