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

use worth_execution::report::Disposition::*;
use worth_execution::report::RetryHint::AfterBasisAdvance;

crate::denial_report::world_denial_report!(CompositeHistoryCatalogDenial, "history", {
    Self::Cancelled => ("cancelled", Cancelled, Final, "The history operation was cancelled."),
    Self::DeadlineElapsed => ("deadline_elapsed", DeadlineElapsed, Final, "The history operation deadline elapsed."),
    Self::ForeignOwner { .. } => ("foreign_owner", Authorization, Reauthorize, "The history subject belongs to another World."),
    Self::ForeignParent { .. } => ("foreign_parent", Authorization, Reauthorize, "The history parent belongs to another World."),
    Self::HistoryPinBasisMismatch => ("pin_basis_mismatch", RelationLaw, CorrectInput, "The history pin does not match the selected basis."),
    Self::DuplicateCommit => ("duplicate_commit", Conflict, Final, "The composite commit is already installed."),
    Self::RootAlreadyInstalled => ("root_already_installed", Conflict, Final, "The composite history root is already installed."),
    Self::MissingParent(_) => ("missing_parent", NotFound, Final, "The requested history parent is unavailable."),
    Self::CommitCapacityExhausted { .. } => ("commit_capacity_exhausted", Exhausted, Final, "The World history commit capacity is exhausted."),
    Self::ArithmeticOverflow => ("arithmetic_overflow", Exhausted, Escalate, "The World history counters cannot represent this operation."),
    Self::MetadataCapacityExhausted { .. } => ("metadata_capacity_exhausted", Exhausted, Final, "The World history metadata capacity is exhausted."),
    Self::DependencyCountOverflow(_) => ("dependency_count_overflow", Exhausted, Escalate, "The World cannot count another dependency on this commit."),
    Self::ProtectionCountOverflow(_) => ("protection_count_overflow", Exhausted, Escalate, "The World cannot count another protection for this commit."),
    Self::UnknownProtectionTarget(_) => ("unknown_protection_target", NotFound, Final, "The requested publication is unavailable for protection."),
    Self::ForeignInspectionCursor => ("foreign_inspection_cursor", InvalidInput, CorrectInput, "The inspection cursor belongs to another history."),
    Self::InspectionFrontierChanged => ("inspection_frontier_changed", Conflict, Retry(AfterBasisAdvance), "The history frontier changed. Start a fresh inspection page."),
    Self::NotPerformedPublication(_) => ("not_performed_publication", InvalidInput, CorrectInput, "The selected commit has no performed publication to protect."),
    Self::ReservationMissing => ("reservation_missing", Defect, Escalate, "The history reservation is missing."),
    Self::ReservationCommitMismatch => ("reservation_commit_mismatch", Defect, Escalate, "The history reservation does not match its commit."),
    Self::ReservationParentMismatch => ("reservation_parent_mismatch", Defect, Escalate, "The history reservation does not match its parent."),
    Self::ReservationChargeMismatch => ("reservation_charge_mismatch", Defect, Escalate, "The history reservation does not match its charge."),
});

impl std::error::Error for CompositeHistoryCatalogDenial {}
