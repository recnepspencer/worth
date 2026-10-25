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
        Denial::Cancelled => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled,
        ),
        Denial::TimedOut => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut,
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
        native @ (Denial::AllocationDenied(_)
        | Denial::CardinalityOverflow
        | Denial::InputDirectoryAllocationDenied { .. }) => {
            return WorthQueryInvariantExecutionFailure::native_staging(
                native,
                "Relational staging owner refused backing or cardinality",
            );
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryInvariantExecutionDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointIdentityExhausted => {
            WorthQueryInvariantExecutionDenialKind::SavepointIdentityExhausted
        }
        Denial::MaterializationAuthorityRequired => {
            return provider_failure(
                "Relational invariant transaction requires materialization authority",
            );
        }
        Denial::MaterializationModeMismatch => {
            return provider_failure(
                "Relational invariant transaction materialization mode does not match its intents",
            );
        }
    };
    exhausted_failure(
        kind,
        "Relational invariant transaction staging exhausted an owner budget",
    )
}

pub(in crate::domain_computation::primary_graph) fn map_validation_failure(
    failure: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::{
        RelationalPublicationDeferred as Deferred, RelationalPublicationFailureKind as Failure,
        TransactionCommitError as Error,
    };
    use worth_relational::facade::transactions::CommitPreparationReason;
    use worth_relational::facade::transactions::ConflictClass;
    let kind = match failure {
        Error::Execution { denial, .. } => {
            return map_execution_failure(denial);
        }
        Error::Interrupted { interruption, .. } => {
            return request_interruption(interruption.interruption());
        }
        Error::Conflict { error, .. } => {
            if let Some(cause) = error.allocation_denial() {
                return WorthQueryInvariantExecutionFailure::physical_allocation(
                    cause.clone(),
                    error.detail(),
                );
            }
            let relational_detail = error.detail();
            let ConflictClass::InvariantViolation { fields, detail, .. } = error.class else {
                return provider_failure(relational_detail);
            };
            return map_invariant_failure(fields, detail);
        }
        Error::PublicationDeferred { deferred, .. } => match deferred {
            Deferred::CompanionRegistrationPending
            | Deferred::CompanionRebindRequired
            | Deferred::CompanionPreflight(_) => {
                WorthQueryInvariantExecutionDenialKind::RelationalDeferred(deferred)
            }
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
            _ => return owner_failure(),
        },
        Error::Preparation { error, .. }
            if error.reason() == CommitPreparationReason::ProposalIdentityOrdinalExhausted =>
        {
            WorthQueryInvariantExecutionDenialKind::ProposalIdentityExhausted
        }
        Error::Preparation { .. }
        | Error::Publication { .. }
        | Error::PublicationDenied { .. }
        | Error::PerformedButDurabilityDeferred { .. } => return owner_failure(),
    };
    exhausted_failure(
        kind,
        "Relational invariant candidate validation exhausted an owner budget",
    )
}

pub(super) fn map_execution_failure(
    denial: worth_relational::facade::transactions::CommitExecutionDenial,
) -> WorthQueryInvariantExecutionFailure {
    let translated = match denial.kind {
        worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) => {
            super::relational_execution_denial::relational_execution_kind(
                cause,
                denial.partition_identity,
            )
        }
    };
    WorthQueryInvariantExecutionFailure::new(
        match translated {
            Ok(kind) => WorthQueryInvariantExecutionDenialKind::ExecutionDenied(kind),
            Err(kind) => WorthQueryInvariantExecutionDenialKind::ExecutionControlStopped(kind),
        },
        format!("{denial:?}"),
    )
}

fn map_invariant_failure(
    fields: worth_relational::facade::transactions::InvariantViolationFields,
    detail: String,
) -> WorthQueryInvariantExecutionFailure {
    if matches!(
        fields,
        worth_relational::facade::transactions::InvariantViolationFields::Acyclicity { .. }
    ) {
        return WorthQueryInvariantExecutionFailure::relation_law_denied(fields, detail);
    }
    map_custom_invariant_failure(fields, detail.clone()).unwrap_or_else(|| {
        WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProviderRejected,
            detail,
        )
    })
}

fn map_custom_invariant_failure(
    fields: worth_relational::facade::transactions::InvariantViolationFields,
    detail: String,
) -> Option<WorthQueryInvariantExecutionFailure> {
    use crate::domain_computation::WorthQueryCustomInvariantDenial;
    use worth_relational::facade::transactions::InvariantViolationFields;

    let denial = match fields {
        InvariantViolationFields::CustomInvariantViolation { identity, refusal } => {
            WorthQueryCustomInvariantDenial::Violation { identity, refusal }
        }
        InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => WorthQueryCustomInvariantDenial::Failure {
            identity,
            phase,
            failure,
        },
        _ => return None,
    };
    Some(WorthQueryInvariantExecutionFailure::custom_invariant(
        denial, detail,
    ))
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
    provider_failure("Relational rejected the installed proposed-state invariant")
}

fn provider_failure(detail: impl Into<std::sync::Arc<str>>) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        detail,
    )
}

pub(super) fn request_interruption(
    reason: worth_relational::facade::mvcc::RelationalOperationInterruption,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(reason),
        "the admitted request interrupted Relational candidate validation",
    )
}

#[cfg(test)]
mod tests;
