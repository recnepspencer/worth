use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

/// The owner, retention, continuation or publication condition that stopped a
/// history operation. A changed inspection frontier requires a fresh page;
/// missing retained history must be handled as unavailable by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeHistoryCatalogDenial {
    Cancelled,
    DeadlineElapsed,
    ForeignOwner {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    ForeignParent {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    HistoryPinBasisMismatch,
    DuplicateCommit,
    RootAlreadyInstalled,
    MissingParent(CompositeCommitIdentity),
    CommitCapacityExhausted {
        maximum: usize,
    },
    ArithmeticOverflow,
    MetadataCapacityExhausted {
        maximum: usize,
        used: usize,
        requested: usize,
    },
    DependencyCountOverflow(CompositeCommitIdentity),
    ProtectionCountOverflow(CompositeCommitIdentity),
    UnknownProtectionTarget(CompositeCommitIdentity),
    ForeignInspectionCursor,
    InspectionFrontierChanged,
    NotPerformedPublication(CompositeCommitIdentity),
    ReservationMissing,
    ReservationCommitMismatch,
    ReservationParentMismatch,
    ReservationChargeMismatch,
}
