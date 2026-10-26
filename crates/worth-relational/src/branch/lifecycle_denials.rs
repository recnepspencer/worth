use crate::branch::RelationalBranchTransitionDenial;
use crate::durability::medium::RelationalDurablePublicationRefusal;
use crate::history::data::BranchId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchArchiveDenial {
    OwnerUnavailable,
    ForeignRuntime,
    UnknownBranch(BranchId),
    AlreadyArchived(BranchId),
    Deleting(BranchId),
    GenerationOverflow,
    /// The medium proved the archive had no durable effect; nothing changed.
    DurablePublicationRefused(RelationalDurablePublicationRefusal),
    /// No branch transition sequence remains.
    TransitionSequenceExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchDeleteDenial {
    OwnerUnavailable,
    ForeignRuntime,
    UnknownBranch(BranchId),
    MainBranch,
    RetentionBackpressure,
    RetentionIdentityExhausted,
    RetiredIdentityCapacityExhausted,
    OwnerFailure,
    /// The medium proved the deletion had no durable effect; nothing changed.
    DurablePublicationRefused(RelationalDurablePublicationRefusal),
    /// No branch transition sequence remains.
    TransitionSequenceExhausted,
}

impl RelationalBranchTransitionDenial for RelationalBranchArchiveDenial {
    fn durable_publication_refused(refusal: RelationalDurablePublicationRefusal) -> Self {
        Self::DurablePublicationRefused(refusal)
    }

    fn transition_sequence_exhausted() -> Self {
        Self::TransitionSequenceExhausted
    }
}

impl RelationalBranchTransitionDenial for RelationalBranchDeleteDenial {
    fn durable_publication_refused(refusal: RelationalDurablePublicationRefusal) -> Self {
        Self::DurablePublicationRefused(refusal)
    }

    fn transition_sequence_exhausted() -> Self {
        Self::TransitionSequenceExhausted
    }
}
