//! Publication failures: the attempt stopped before any reference moved
//! because an owner-side bound or invariant failed, not because the caller
//! must retry or was denied.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalPublicationFailureKind {
    SnapshotIdentityExhausted,
    SnapshotCountOverflow,
    CandidateIdentityExhausted,
    PreparedRootMismatch,
    PreparedBasisDescriptor(crate::branch::RelationalBranchBasisDenial),
    NextBasisAdmission(crate::branch::RelationalBranchBasisDenial),
    /// The selected branch root was already gone when the critical section
    /// looked for it, so publication stopped before any comparison or any
    /// reference movement.
    SelectedRootUnavailable,
    BranchObservation(crate::branch::RelationalBranchBasisDenial),
    PatchPositionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwner,
    /// A pending settlement record already exists for this candidate's
    /// owner-issued commit identity, so the pre-effect reservation would have
    /// aliased another attempt's recovery state.
    PendingSettlementIdentityConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalPublicationFailure {
    kind: RelationalPublicationFailureKind,
    detail: String,
}

impl RelationalPublicationFailure {
    pub(crate) fn new(kind: RelationalPublicationFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> &RelationalPublicationFailureKind {
        &self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
