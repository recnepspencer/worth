use super::{
    provider_failure, WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

pub(super) fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
}

pub(super) fn native_output_witness_stop(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
        crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(
                    stop,
                ),
            ),
            "native output witness preparation denied before publication",
        ),
    )
}

pub(super) fn touched_record_preparation_stop(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
        crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(
                    stop,
                ),
            ),
            "touched-record receipt storage denied before publication",
        ),
    )
}

pub(super) fn index_preparation_stop(
    denial: worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind as Kind;
    if let Kind::CandidateLifetimeExpired {
        maximum_lifetime_millis,
    } = &denial.kind
    {
        return crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
                crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::CandidateLifetimeExpired {
                    maximum_lifetime_millis: *maximum_lifetime_millis,
                },
                "prepared candidate expired before primary index admission",
            ),
        );
    }
    let kind = match denial.kind {
        Kind::WorkBudgetExceeded | Kind::ColdReconstructionRequired => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexMaintenanceBudgetExceeded
        }
        Kind::GenerationIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexGenerationIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
        WorthQueryProviderSessionFailure::new(
            kind,
            WorthQueryProviderSessionProtocolStage::Commit,
            format!("primary index candidate preparation denied: {denial:?}"),
            crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
        ),
    )
}

/// Used only for `prepare_validated_proposal`, before product publication.
pub(super) fn transaction_commit_stop(
    error: worth_relational::facade::mvcc::TransactionCommitError,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped as ControlStopped,
        WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionControlStopKind as ControlKind,
    };
    use worth_relational::facade::{
        mvcc::TransactionCommitError as Error,
        transactions::CommitExecutionDenialKind as ExecutionKind,
    };
    match error {
        Error::Interrupted { interruption, .. } => {
            Stop::ControlStopped(interruption_control_stopped(interruption))
        }
        Error::PublicationDeferred { deferred, .. } => {
            Stop::Deferred(publication_deferred(deferred))
        }
        Error::PerformedButDurabilityDeferred {
            settlement, error, ..
        } => Stop::SettlementDeferred(
            crate::domain_computation::WorthQueryProviderSessionSettlementDeferred::new(
                error.detail,
                settlement,
            ),
        ),
        Error::Execution { denial, .. }
            if matches!(
                denial.kind,
                ExecutionKind::Cancelled | ExecutionKind::DeadlineElapsed
            ) =>
        {
            let kind = if denial.kind == ExecutionKind::Cancelled {
                ControlKind::Cancelled
            } else {
                ControlKind::TimedOut
            };
            Stop::ControlStopped(ControlStopped::new(kind, format!("{denial:?}")))
        }
        error @ (Error::Conflict { .. }
        | Error::Publication { .. }
        | Error::Preparation { .. }
        | Error::Execution { .. }
        | Error::PublicationDenied { .. }
        | Error::PublicationFailed { .. }) => {
            let failure = match &error {
                Error::PublicationFailed { failure, .. } => publication_failure(failure),
                _ => WorthQueryProviderSessionFailure::new(
                    crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
                    WorthQueryProviderSessionProtocolStage::Commit,
                    error.detail(),
                    crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
                ),
            }.with_native_preparation_error(error);
            Stop::PreEffectDenied(failure)
        }
    }
}

fn interruption_control_stopped(
    event: worth_relational::facade::mvcc::RelationalInterruptionEvent,
) -> crate::domain_computation::WorthQueryProviderSessionCommitControlStopped {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Kind;
    let kind = match event.interruption() {
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
            Kind::Cancelled
        }
        worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => Kind::TimedOut,
    };
    crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::new(
        kind,
        format!("{event:?}"),
    )
}

fn publication_deferred(
    deferred: worth_relational::facade::mvcc::RelationalPublicationDeferred,
) -> crate::domain_computation::WorthQueryProviderSessionCommitDeferred {
    use crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind as Kind;
    use worth_relational::facade::mvcc::RelationalPublicationDeferred as Deferred;
    let kind = match deferred {
        Deferred::CompanionRegistrationPending
        | Deferred::CompanionRebindRequired
        | Deferred::CompanionPreflight(_) => Kind::RelationalDeferred(deferred),
        Deferred::PatchPositionReservationContended => Kind::PatchPositionReservationContended,
        Deferred::RetentionBackpressure => Kind::RetentionCapacityExhausted,
        Deferred::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        } => Kind::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        },
        Deferred::CandidateCapacityExhausted { maximum_candidates } => {
            Kind::CandidateCapacityExhausted { maximum_candidates }
        }
        Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
            Kind::PublishedSnapshotCapacityExhausted { maximum_handles }
        }
    };
    crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
        kind,
        format!("{deferred:?}"),
    )
}

fn publication_failure(
    failure: &worth_relational::facade::mvcc::RelationalPublicationFailure,
) -> WorthQueryProviderSessionFailure {
    use worth_relational::facade::mvcc::RelationalPublicationFailureKind as Failure;
    let kind = match failure.kind() {
        Failure::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        Failure::CandidateIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::CandidateIdentityExhausted
        }
        Failure::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        Failure::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => crate::domain_computation::WorthQueryProviderSessionDenialKind::PreparedRootBudgetExhausted {
            maximum_bytes: *maximum_bytes,
            required_bytes: *required_bytes,
        },
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        failure.detail(),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}

#[cfg(test)]
mod tests;
