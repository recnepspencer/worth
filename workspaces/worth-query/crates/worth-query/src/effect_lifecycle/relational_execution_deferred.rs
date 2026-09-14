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
        Denial::OverlayCapacityExhausted {
            maximum_bytes,
            required_bytes,
        } => EffectExecutionDenialKind::TransactionOverlayBudgetExceeded {
            maximum_bytes,
            required_bytes,
        },
        Denial::FootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => EffectExecutionDenialKind::TransactionFootprintBudgetExceeded {
            maximum_loci,
            required_loci,
        },
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            EffectExecutionDenialKind::TransactionSavepointBudgetExceeded { maximum_savepoints }
        }
        Denial::SavepointFootprintCapacityExhausted {
            maximum_loci,
            required_loci,
        } => EffectExecutionDenialKind::TransactionSavepointFootprintBudgetExceeded {
            maximum_loci,
            required_loci,
        },
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
        TransactionCommitError::Interrupted { interruption, .. } => {
            interruption_event(interruption)
        }
        TransactionCommitError::PublicationDeferred { deferred, .. } => publication(deferred),
        TransactionCommitError::PublicationFailed { failure, .. } => publication_failure(failure),
        TransactionCommitError::PerformedButDurabilityDeferred {
            settlement, error, ..
        } => RelationalEffectExecutionFailure::settlement_deferred(error.detail, settlement),
        other => denied(&other),
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

fn denied(error: &impl std::fmt::Debug) -> RelationalEffectExecutionFailure {
    RelationalEffectExecutionFailure::Denied {
        kind: EffectExecutionDenialKind::RelationalCommitFailed,
        message: format!("{error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_pressure_remains_typed_across_the_effect_boundary() {
        let staging = transaction_staging(
            RelationalTransactionStagingDenial::OverlayCapacityExhausted {
                maximum_bytes: 64,
                required_bytes: 65,
            },
        );
        assert!(matches!(
            staging,
            RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionOverlayBudgetExceeded {
                    maximum_bytes: 64,
                    required_bytes: 65,
                },
                ..
            }
        ));

        let publication = publication(RelationalPublicationDeferred::CandidateCapacityExhausted {
            maximum_candidates: 7,
        });
        assert!(matches!(
            publication,
            RelationalEffectExecutionFailure::Deferred {
                kind: EffectExecutionDeferredKind::CandidateCapacityExhausted {
                    maximum_candidates: 7,
                },
                ..
            }
        ));
    }

    #[test]
    fn interruption_and_retention_admission_remain_typed() {
        assert!(matches!(
            transaction_admission(
                RelationalBranchTransactionAdmissionDenial::RetentionCapacityExhausted
            ),
            RelationalEffectExecutionFailure::Deferred {
                kind: EffectExecutionDeferredKind::TransactionRetentionCapacityExhausted,
                ..
            }
        ));
        assert!(matches!(
            transaction_admission(RelationalBranchTransactionAdmissionDenial::Cancelled),
            RelationalEffectExecutionFailure::ControlStopped {
                kind: crate::effect_lifecycle::EffectExecutionControlStopKind::Cancelled,
                ..
            }
        ));
    }
}
