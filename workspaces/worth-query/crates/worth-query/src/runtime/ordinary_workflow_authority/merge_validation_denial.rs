//! Merge validation keeps backend execution refusals visible to its caller.
use super::WorthQueryMergeAuthorityValidationError as Denial;
use crate::memory_workspace::WorthQueryWorkspaceErrorKind as Kind;

pub(crate) fn classify_merge_validation_kind(kind: Kind) -> Denial {
    match kind {
        Kind::ExecutionDenied(kind) => Denial::ExecutionDenied(kind),
        Kind::ExecutionControlStopped(kind) => Denial::ExecutionControlStopped(kind),
        Kind::RetentionCapacityExhausted => Denial::RetentionBackpressure,
        Kind::RetentionIdentityExhausted => Denial::RetentionIdentityExhausted,
        Kind::SnapshotIdentityExhausted => Denial::SnapshotIdentityExhausted,
        Kind::Unclassified
        | Kind::UnsupportedCollection
        | Kind::UnsupportedWriteFamily
        | Kind::EmptySchema
        | Kind::BatchAtomicityUnsupported
        | Kind::TransactionOverlayCapacityExhausted { .. }
        | Kind::TransactionFootprintCapacityExhausted { .. }
        | Kind::SavepointCapacityExhausted { .. }
        | Kind::SavepointFootprintCapacityExhausted { .. }
        | Kind::SavepointIdentityExhausted
        | Kind::TransactionMaterializationAuthorityRequired
        | Kind::TransactionMaterializationModeMismatch
        | Kind::CandidateCapacityExhausted { .. }
        | Kind::PublishedSnapshotCapacityExhausted { .. }
        | Kind::CandidateIdentityExhausted
        | Kind::PreparedRootBudgetExhausted { .. }
        | Kind::PatchPositionReservationContended
        | Kind::ProposalIdentityExhausted
        | Kind::RelationalBasisUnavailable
        | Kind::InvalidationCompanionPending
        | Kind::InvalidationCompanionCapacityExhausted => Denial::StaleSnapshot,
    }
}
