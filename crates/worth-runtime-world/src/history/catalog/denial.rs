use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeHistoryCatalogDenial {
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
    ReservationMissing,
    ReservationCommitMismatch,
    ReservationParentMismatch,
    ReservationChargeMismatch,
}
