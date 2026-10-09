use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalInterruptionEvent,
    RelationalOperationInterruption, RelationalPublicationDeferred,
    RelationalTransactionStagingDenial, TransactionCommitError,
};

use super::{
    EffectExecutionDeferredKind, EffectExecutionDenialKind, RelationalEffectExecutionFailure,
};

pub(super) fn transaction_admission(
    denial: RelationalBranchTransactionAdmissionDenial,
) -> RelationalEffectExecutionFailure {
    use RelationalBranchTransactionAdmissionDenial as Denial;
    let kind = match denial {
        Denial::RetentionCapacityExhausted => {
            Some(EffectExecutionDeferredKind::TransactionRetentionCapacityExhausted)
        }
        Denial::RetentionOwnerUnavailable => {
            return RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionRetentionOwnerUnavailable,
                message: format!("{denial:?}"),
            };
        }
        Denial::RetentionIdentityExhausted => {
            return RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionRetentionIdentityExhausted,
                message: format!("{denial:?}"),
            };
        }
        Denial::RetentionInvariantViolation => {
            return RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionRetentionInvariantViolation,
                message: format!("{denial:?}"),
            };
        }
        Denial::Cancelled => {
            return RelationalEffectExecutionFailure::ControlStopped {
                kind: crate::effect_lifecycle::EffectExecutionControlStopKind::Cancelled,
                message: format!("{denial:?}"),
            };
        }
        Denial::TimedOut => {
            return RelationalEffectExecutionFailure::ControlStopped {
                kind: super::EffectExecutionControlStopKind::TimedOut,
                message: format!("{denial:?}"),
            };
        }
        _ => None,
    };
    kind.map_or_else(
        || denied(&denial),
        |kind| RelationalEffectExecutionFailure::deferred(kind, format!("{denial:?}")),
    )
}

pub(super) fn transaction_staging(
    denial: RelationalTransactionStagingDenial,
) -> RelationalEffectExecutionFailure {
    use RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        Denial::AllocationDenied(ref allocation) => return allocation_failure(allocation),
        Denial::CardinalityOverflow => {
            EffectExecutionDenialKind::TransactionStagingCardinalityOverflow
        }
        Denial::InputDirectoryAllocationDenied { requested_batches } => {
            EffectExecutionDenialKind::TransactionInputDirectoryAllocationDenied {
                requested_batches,
            }
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            EffectExecutionDenialKind::TransactionSavepointBudgetExceeded { maximum_savepoints }
        }
        Denial::SavepointIdentityExhausted => {
            return RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionSavepointIdentityExhausted,
                message: format!("{denial:?}"),
            };
        }
        Denial::MaterializationAuthorityRequired => {
            EffectExecutionDenialKind::TransactionMaterializationAuthorityRequired
        }
        Denial::MaterializationModeMismatch => {
            EffectExecutionDenialKind::TransactionMaterializationModeMismatch
        }
    };
    RelationalEffectExecutionFailure::Denied {
        kind,
        message: format!("{denial:?}"),
    }
}

pub(super) fn transaction_commit(
    error: TransactionCommitError,
) -> RelationalEffectExecutionFailure {
    match error {
        TransactionCommitError::Execution { denial, .. } => {
            let translated = match denial.kind {
                worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) => {
                    worth_query_execution::facade::primary_graph::relational_execution_kind(
                        cause,
                        denial.partition_identity,
                    )
                }
            };
            match translated {
                Ok(kind) => RelationalEffectExecutionFailure::Denied {
                    kind: EffectExecutionDenialKind::RelationalExecutionDenied(kind),
                    message: format!("{denial:?}"),
                },
                Err(kind) => RelationalEffectExecutionFailure::ControlStopped {
                    kind: match kind {
                        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled => super::EffectExecutionControlStopKind::Cancelled,
                        worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut => super::EffectExecutionControlStopKind::TimedOut,
                    },
                    message: format!("{denial:?}"),
                },
            }
        }
        TransactionCommitError::Conflict { error, .. } => {
            if let Some(allocation) = error.allocation_denial() {
                return allocation_failure(allocation);
            }
            let kind = match &error.class {
                worth_relational::facade::transactions::ConflictClass::TransactionStagingCardinalityOverflow => EffectExecutionDenialKind::TransactionStagingCardinalityOverflow,
                worth_relational::facade::transactions::ConflictClass::TransactionInputDirectoryAllocationDenied { requested_batches } => EffectExecutionDenialKind::TransactionInputDirectoryAllocationDenied { requested_batches: *requested_batches },
                _ => return denied(&error),
            };
            RelationalEffectExecutionFailure::Denied {
                kind,
                message: format!("{error:?}"),
            }
        }
        TransactionCommitError::Interrupted { interruption, .. } => {
            interruption_event(interruption)
        }
        TransactionCommitError::PublicationDeferred { deferred, .. } => publication(deferred),
        TransactionCommitError::PublicationFailed { failure, .. } => publication_failure(failure),
        TransactionCommitError::PerformedButDurabilityDeferred {
            settlement, error, ..
        } => RelationalEffectExecutionFailure::settlement_deferred(error.detail, settlement),
        other @ (TransactionCommitError::Publication { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::PublicationDenied { .. }) => denied(&other),
    }
}

pub(super) fn interruption_event(
    event: RelationalInterruptionEvent,
) -> RelationalEffectExecutionFailure {
    let kind = match event.interruption() {
        RelationalOperationInterruption::Cancelled => {
            super::EffectExecutionControlStopKind::Cancelled
        }
        RelationalOperationInterruption::TimedOut => {
            super::EffectExecutionControlStopKind::TimedOut
        }
    };
    RelationalEffectExecutionFailure::ControlStopped {
        kind,
        message: format!("{event:?}"),
    }
}

pub(super) fn publication(
    deferred: RelationalPublicationDeferred,
) -> RelationalEffectExecutionFailure {
    use RelationalPublicationDeferred as Deferred;
    let kind = match deferred {
        Deferred::CompanionRegistrationPending | Deferred::CompanionRebindRequired => {
            EffectExecutionDeferredKind::InvalidationCompanionPending
        }
        Deferred::CompanionPreflight(stop) if super::companion_stop_is_transient(&stop) => {
            EffectExecutionDeferredKind::InvalidationCompanionPending
        }
        Deferred::CompanionPreflight(_) => {
            EffectExecutionDeferredKind::InvalidationCompanionCapacityExhausted
        }
        Deferred::PatchPositionReservationContended => {
            EffectExecutionDeferredKind::PatchPositionReservationContended
        }
        Deferred::RetentionBackpressure => EffectExecutionDeferredKind::RetentionBackpressure,
        Deferred::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        } => EffectExecutionDeferredKind::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        },
        Deferred::CandidateCapacityExhausted { maximum_candidates } => {
            EffectExecutionDeferredKind::CandidateCapacityExhausted { maximum_candidates }
        }
        Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
            EffectExecutionDeferredKind::PublishedSnapshotCapacityExhausted { maximum_handles }
        }
    };
    RelationalEffectExecutionFailure::deferred(kind, format!("{deferred:?}"))
}

pub(super) fn publication_failure(
    failure: worth_relational::facade::mvcc::RelationalPublicationFailure,
) -> RelationalEffectExecutionFailure {
    use worth_relational::facade::mvcc::RelationalPublicationFailureKind as Failure;
    let kind = match failure.kind() {
        Failure::SnapshotIdentityExhausted => EffectExecutionDenialKind::SnapshotIdentityExhausted,
        Failure::CandidateIdentityExhausted => {
            EffectExecutionDenialKind::CandidateIdentityExhausted
        }
        Failure::RetentionIdentityExhausted => {
            EffectExecutionDenialKind::TransactionRetentionIdentityExhausted
        }
        Failure::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => EffectExecutionDenialKind::PreparedRootBudgetExceeded {
            maximum_bytes: *maximum_bytes,
            required_bytes: *required_bytes,
        },
        _ => EffectExecutionDenialKind::RelationalCommitFailed,
    };
    RelationalEffectExecutionFailure::Denied {
        kind,
        message: failure.detail().to_owned(),
    }
}

fn allocation_failure(
    denial: &worth_execution::ExecutionAllocationDenial,
) -> RelationalEffectExecutionFailure {
    use worth_execution::ExecutionAllocationDenialKind as Kind;
    let control = match denial.kind() {
        Kind::Cancelled => Some(super::EffectExecutionControlStopKind::Cancelled),
        Kind::DeadlineElapsed => Some(super::EffectExecutionControlStopKind::TimedOut),
        Kind::Layout
        | Kind::Lease(_)
        | Kind::Allocator
        | Kind::CapacityMismatch
        | Kind::WriteBeyondReserved
        | Kind::IncompleteSeal => None,
    };
    if let Some(kind) = control {
        RelationalEffectExecutionFailure::ControlStopped {
            kind,
            message: denial.to_string(),
        }
    } else {
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::TransactionAllocationDenied {
                kind: denial.kind(),
                requested_payload_bytes: denial.requested_payload_bytes(),
            },
            message: denial.to_string(),
        }
    }
}

fn denied(error: &impl std::fmt::Debug) -> RelationalEffectExecutionFailure {
    RelationalEffectExecutionFailure::Denied {
        kind: EffectExecutionDenialKind::RelationalCommitFailed,
        message: format!("{error:?}"),
    }
}

#[cfg(test)]
mod tests;
