//! Merge validation keeps backend execution refusals visible to its caller.
use super::WorthQueryMergeAuthorityValidationError as Denial;
use crate::memory_workspace::WorthQueryWorkspaceErrorKind as Kind;

pub(crate) fn classify_merge_validation_kind(kind: Kind) -> Denial {
    match kind {
        Kind::ExecutionDenied(kind) => Denial::ExecutionDenied(kind),
        Kind::ExecutionControlStopped(kind) => Denial::ExecutionControlStopped(kind),
        Kind::TransactionAllocationDenied { kind, requested_payload_bytes } => Denial::TransactionBackingDenied(crate::effect_lifecycle::EffectExecutionDenialKind::TransactionAllocationDenied { kind, requested_payload_bytes }),
        Kind::TransactionStagingCardinalityOverflow => Denial::TransactionBackingDenied(crate::effect_lifecycle::EffectExecutionDenialKind::TransactionStagingCardinalityOverflow),
        Kind::TransactionInputDirectoryAllocationDenied { requested_batches } => Denial::TransactionBackingDenied(crate::effect_lifecycle::EffectExecutionDenialKind::TransactionInputDirectoryAllocationDenied { requested_batches }),
        Kind::RetentionCapacityExhausted => Denial::RetentionBackpressure,
        Kind::RetentionIdentityExhausted => Denial::RetentionIdentityExhausted,
        Kind::SnapshotIdentityExhausted => Denial::SnapshotIdentityExhausted,
        Kind::Unclassified
        | Kind::UnsupportedCollection
        | Kind::UnsupportedWriteFamily
        | Kind::EmptySchema
        | Kind::BatchAtomicityUnsupported
        | Kind::SavepointCapacityExhausted { .. }
        | Kind::SavepointIdentityExhausted
        | Kind::TransactionMaterializationAuthorityRequired
        | Kind::TransactionMaterializationModeMismatch
        | Kind::CandidateCapacityExhausted { .. }
        | Kind::PublishedSnapshotCapacityExhausted { .. }
        | Kind::CandidateIdentityExhausted
        | Kind::PatchPositionReservationContended
        | Kind::ProposalIdentityExhausted
        | Kind::RelationalBasisUnavailable
        | Kind::InvalidationCompanionPending
        | Kind::InvalidationCompanionCapacityExhausted => Denial::StaleSnapshot,
    }
}
