use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure,
};

pub(super) fn map_transaction_admission_failure(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    match denial {
        Denial::StaleBasis => WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProductBasisStale,
            "the exact product basis became stale before invariant candidate admission",
        ),
        Denial::RetentionCapacityExhausted => retention_capacity_failure(),
        Denial::RetentionIdentityExhausted => exhausted_failure(
            WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted,
            "Relational invariant execution exhausted retention identity space",
        ),
        _ => owner_failure(),
    }
}

pub(super) fn map_transaction_staging_failure(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        Denial::OverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        } => WorthQueryInvariantExecutionDenialKind::TransactionOverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        },
        Denial::FootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryInvariantExecutionDenialKind::TransactionFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryInvariantExecutionDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => WorthQueryInvariantExecutionDenialKind::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointIdentityExhausted => {
            WorthQueryInvariantExecutionDenialKind::SavepointIdentityExhausted
        }
    };
    exhausted_failure(
        kind,
        "Relational invariant transaction staging exhausted an owner budget",
    )
}

pub(super) fn map_validation_failure(
    failure: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
        TransactionCommitError as Error,
    };
    use worth_relational::facade::transactions::CommitPreparationReason;
    let kind = match failure {
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::RetentionBackpressure => return retention_capacity_failure(),
            Deferred::PatchPositionReservationContended => {
                WorthQueryInvariantExecutionDenialKind::PatchPositionReservationContended
            }
            Deferred::CandidateCapacityExhausted { maximum_candidates } => {
                WorthQueryInvariantExecutionDenialKind::CandidateCapacityExhausted {
                    maximum_candidates,
                }
            }
            Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                WorthQueryInvariantExecutionDenialKind::PublishedSnapshotCapacityExhausted {
                    maximum_handles,
                }
            }
            Deferred::CandidateLifetimeExpired { .. } => return owner_failure(),
        },
        Error::PublicationFailed { failure, .. } => match failure.kind() {
            Failure::SnapshotIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::SnapshotIdentityExhausted
            }
            Failure::CandidateIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::CandidateIdentityExhausted
            }
            Failure::RetentionIdentityExhausted => {
                WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted
            }
            Failure::PreparedRootBudgetExhausted {
                maximum_bytes,
                required_bytes,
            } => WorthQueryInvariantExecutionDenialKind::PreparedRootBudgetExhausted {
                maximum_bytes: *maximum_bytes,
                required_bytes: *required_bytes,
            },
            _ => return owner_failure(),
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryInvariantExecutionDenialKind::ProposalIdentityExhausted
        }
        _ => return owner_failure(),
    };
    exhausted_failure(
        kind,
        "Relational invariant candidate validation exhausted an owner budget",
    )
}

fn retention_capacity_failure() -> WorthQueryInvariantExecutionFailure {
    exhausted_failure(
        WorthQueryInvariantExecutionDenialKind::RetentionCapacityExhausted,
        "Relational invariant execution exhausted owner retention capacity",
    )
}

fn exhausted_failure(
    kind: WorthQueryInvariantExecutionDenialKind,
    detail: &'static str,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::exhausted(kind, detail)
}

fn owner_failure() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        "Relational rejected the installed proposed-state invariant",
    )
}
