use super::{WorthQueryWorkspaceError, WorthQueryWorkspaceErrorKind};

pub(super) fn basis(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryWorkspaceError {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
    };
    WorthQueryWorkspaceError::with_kind(kind, format!("workspace basis denied: {denial:?}"))
}

pub(super) fn admission(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryWorkspaceError {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    let kind = match denial {
        Denial::RetentionCapacityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
        }
        Denial::RetentionIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
        }
        _ => WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
    };
    WorthQueryWorkspaceError::with_kind(
        kind,
        format!("workspace transaction admission denied: {denial:?}"),
    )
}

pub(super) fn staging(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryWorkspaceError {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        Denial::AllocationDenied(_)
        | Denial::CardinalityOverflow
        | Denial::InputDirectoryAllocationDenied { .. } => {
            WorthQueryWorkspaceErrorKind::Unclassified
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryWorkspaceErrorKind::SavepointCapacityExhausted { maximum_savepoints }
        }
        Denial::SavepointIdentityExhausted => {
            WorthQueryWorkspaceErrorKind::SavepointIdentityExhausted
        }
        Denial::MaterializationAuthorityRequired => {
            WorthQueryWorkspaceErrorKind::TransactionMaterializationAuthorityRequired
        }
        Denial::MaterializationModeMismatch => {
            WorthQueryWorkspaceErrorKind::TransactionMaterializationModeMismatch
        }
    };
    WorthQueryWorkspaceError::with_kind(kind, format!("workspace staging denied: {denial:?}"))
}

pub(super) fn commit(
    error: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryWorkspaceError {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
    };
    use worth_relational::facade::transactions::{
        CommitPreparationReason, TransactionCommitError as Error,
    };
    let kind = match &error {
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::PatchPositionReservationContended => {
                WorthQueryWorkspaceErrorKind::PatchPositionReservationContended
            }
            Deferred::RetentionBackpressure => {
                WorthQueryWorkspaceErrorKind::RetentionCapacityExhausted
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryWorkspaceErrorKind::CandidateCapacityExhausted {
                    maximum_candidates: *maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryWorkspaceErrorKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles: *maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => WorthQueryWorkspaceErrorKind::Unclassified,
            Deferred::CompanionRegistrationPending | Deferred::CompanionRebindRequired => {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionPending
            }
            Deferred::CompanionPreflight(stop)
                if crate::effect_lifecycle::companion_stop_is_transient(stop) =>
            {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionPending
            }
            Deferred::CompanionPreflight(_) => {
                WorthQueryWorkspaceErrorKind::InvalidationCompanionCapacityExhausted
            }
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryWorkspaceErrorKind::RetentionIdentityExhausted
            }
            _ => WorthQueryWorkspaceErrorKind::Unclassified,
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryWorkspaceErrorKind::ProposalIdentityExhausted
        }
        _ => WorthQueryWorkspaceErrorKind::Unclassified,
    };
    WorthQueryWorkspaceError::with_kind(
        kind,
        format!("workspace commit denied: {}", error.detail()),
    )
}
