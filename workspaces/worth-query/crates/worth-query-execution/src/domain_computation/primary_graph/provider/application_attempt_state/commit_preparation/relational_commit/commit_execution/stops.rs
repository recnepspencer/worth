use super::{
    provider_failure, WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};
use crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop;

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
        Kind::Execution(cause) => return relational_execution_stop(cause, None),
        Kind::WorkBudgetExceeded | Kind::ColdReconstructionRequired => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexMaintenanceBudgetExceeded
        }
        Kind::GenerationIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexGenerationIdentityExhausted
        }
        Kind::Basis(_)
        | Kind::SnapshotUnavailable
        | Kind::CommitMismatch
        | Kind::BeforeRootMismatch
        | Kind::IndexUnavailable(_)
        | Kind::GenerationKindMismatch(_)
        | Kind::PriorEntryMismatch
        | Kind::ForeignCandidate
        | Kind::CandidateLifetimeExpired { .. }
        | Kind::CandidateUnavailable
        | Kind::CandidateIndexesAlreadyPrepared => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected
        }
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
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    use worth_relational::facade::mvcc::TransactionCommitError as Error;
    let native_execution_error = matches!(&error, Error::Execution { .. }).then(|| error.clone());
    match error {
        Error::Execution { denial, .. } => match denial.kind {
            worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) => {
                match relational_execution_stop(cause, denial.partition_identity) {
                    Stop::PreEffectDenied(failure) => Stop::PreEffectDenied(
                        WorthQueryProviderSessionFailure::new(
                            crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
                            WorthQueryProviderSessionProtocolStage::Commit,
                            failure.detail(),
                            Default::default(),
                        ).with_native_preparation_error(native_execution_error.expect("native execution error retained")),
                    ),
                    stopped => stopped,
                }
            }
        },
        Error::Interrupted { interruption, .. } => {
            Stop::ControlStopped(interruption_control_stopped(interruption))
        }
        Error::PublicationDeferred { deferred, .. } => {
            Stop::Deferred(publication_deferred(deferred))
        }
        // The durable medium may hold the commit, so it is never a pre-effect
        // denial: the session requires commit recovery.
        error @ Error::DurabilityUnresolved { .. } => Stop::Denied(
            WorthQueryProviderSessionFailure::new(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
                WorthQueryProviderSessionProtocolStage::Commit,
                error.detail(),
                crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
            )
            .with_native_preparation_error(error),
        ),
        error @ (Error::Conflict { .. }
        | Error::Publication { .. }
        | Error::Preparation { .. }
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

pub(super) fn interruption_control_stopped(
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
        Failure::SnapshotCountOverflow
        | Failure::PreparedRootMismatch
        | Failure::PreparedBasisDescriptor(_)
        | Failure::NextBasisAdmission(_)
        | Failure::SelectedRootUnavailable
        | Failure::BranchObservation(_)
        | Failure::PatchPositionCapacityExhausted
        | Failure::RetentionOwner
        | Failure::PendingSettlementIdentityConflict => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected
        }
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        failure.detail(),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}

#[cfg(test)]
mod leased_requests;
#[cfg(test)]
mod tests;

pub(super) fn source_rebase_preparation_stop(
    cause: super::super::PreparedRebaseDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use super::super::PreparedRebaseDenial as Denial;
    use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped as Control,
        WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionControlStopKind as Kind,
    };
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interrupted;
    if let Denial::Admission(stop) = &cause {
        if let worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) = stop {
            return relational_execution_stop(*cause, None);
        }
        if let worth_relational::facade::mvcc::CompanionPreflightStop::Interrupted(event) = stop {
            return Stop::ControlStopped(interruption_control_stopped(*event));
        }
        return Stop::Deferred(crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(*stop)),
            "source rebase selection refused before publication"));
    }
    if let Denial::Retention(StoreDenial::RequestInterruption(interruption)) = &cause {
        return Stop::ControlStopped(Control::new(
            match interruption {
                Interrupted::Cancelled => Kind::Cancelled,
                Interrupted::DeadlineExceeded => Kind::TimedOut,
            },
            "source rebase preparation interrupted",
        ));
    }
    if let Denial::Retention(StoreDenial::Allocation(allocation)) = cause {
        use worth_execution::{
            ExecutionAllocationDenialClass as Class, MapKernelStop as Kernel,
            WorkCeilingDenial as Work,
        };
        if let Class::Request(Work::Stopped(stop @ (Kernel::Cancelled | Kernel::DeadlineElapsed))) =
            allocation.classify()
        {
            return Stop::ControlStopped(Control::new(
                if stop == Kernel::Cancelled {
                    Kind::Cancelled
                } else {
                    Kind::TimedOut
                },
                "source rebase preparation interrupted",
            ));
        }
        return Stop::PreEffectDenied(WorthQueryProviderSessionFailure::allocation_denied(
            allocation,
        ));
    }
    Stop::PreEffectDenied(WorthQueryProviderSessionFailure::new(
        crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("source rebase preparation refused before publication: {cause:?}"),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    ))
}
