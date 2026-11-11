//! World recovery and publication refusals retain their actual request disposition.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::primary_graph::{
    CompanionPreflightStop, CurrentProductReadDependencyDenial, NoEffectCause,
    RelationalOperationInterruption, RelationalPublicationDeferred,
    RelationalPublicationReadDependencyDenial, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldRecoveryDenial, RuntimeWorldServiceDenial,
};

pub(super) fn recovery(cause: RuntimeWorldRecoveryDenial) -> BankHttpDenial {
    use RuntimeWorldRecoveryDenial as D;
    match cause {
        D::StaleIncarnation => stale(),
        D::HomeProgramTransitionRequired => {
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        }
        D::Busy | D::TooYoung | D::OutputCapacityExhausted | D::CallerCapabilityLive => {
            unavailable()
        }
        D::RecoveredCustodyRequired
        | D::ContinuationAlreadyCommitted
        | D::ContinuationAlreadyStarted
        | D::ReleaseAlreadyStarted
        | D::OriginalSourceUnavailable
        | D::OriginalSourceMismatch
        | D::ReopenRequired
        | D::OwnerUnavailable(_)
        | D::ForeignHandle
        | D::MissingRecord
        | D::ClockRegressed
        | D::SettlementRequired
        | D::SettlementEvidenceUnavailable => operator(),
    }
}

pub(super) fn native_admission(cause: CompanionPreflightStop) -> BankHttpDenial {
    use CompanionPreflightStop as D;
    match cause {
        D::Execution(cause) => super::relational_execution_denial::denial(cause),
        D::Interrupted(cause) => match cause.interruption() {
            RelationalOperationInterruption::Cancelled => cancelled(),
            RelationalOperationInterruption::TimedOut => deadline(),
        },
        D::TopologyPending | D::ManagedSourceNotPrepared => unavailable(),
        D::ManagedSourceChanged
        | D::SelectedSourceMismatch
        | D::SelectedPositionUnavailable { .. }
        | D::RegistrationChanged => stale(),
        D::ForeignCell | D::WorkCounterOverflow | D::PreparationMemoryCounterOverflow => operator(),
    }
}

pub(super) fn publication(cause: NoEffectCause) -> BankHttpDenial {
    match cause {
        NoEffectCause::ExecutionRequest(cause) => super::advancement_denial::request(cause),
        NoEffectCause::CancelledBeforeEffect => cancelled(),
        NoEffectCause::DeadlineBeforeEffect => deadline(),
        NoEffectCause::StaleExpectedProductHead | NoEffectCause::StaleIncarnation => stale(),
        NoEffectCause::CapacityExhausted => unavailable(),
        NoEffectCause::CurrentReadDependency(cause) => current_read(cause),
        NoEffectCause::RelationalDeferred(cause) => match cause {
            RelationalPublicationDeferred::CompanionPreflight(cause) => native_admission(cause),
            RelationalPublicationDeferred::CompanionRegistrationPending
            | RelationalPublicationDeferred::CompanionRebindRequired
            | RelationalPublicationDeferred::PatchPositionReservationContended
            | RelationalPublicationDeferred::RetentionBackpressure
            | RelationalPublicationDeferred::CandidateLifetimeExpired { .. }
            | RelationalPublicationDeferred::CandidateCapacityExhausted { .. }
            | RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted { .. } => {
                unavailable()
            }
        },
        NoEffectCause::DurableOpenRequired
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::PreEffectFailure => operator(),
    }
}

pub(super) fn current_read(cause: CurrentProductReadDependencyDenial) -> BankHttpDenial {
    use CurrentProductReadDependencyDenial as D;
    match cause {
        D::CutoverDeadlineElapsed => deadline(),
        D::Branch(cause) => branch(cause),
        D::WorldHeadUnavailable => unavailable(),
        D::Native(cause) => match cause {
            RelationalPublicationReadDependencyDenial::NotCurrent => stale(),
            RelationalPublicationReadDependencyDenial::CandidateUnavailable => unavailable(),
            RelationalPublicationReadDependencyDenial::EmptySourceUnsupported
            | RelationalPublicationReadDependencyDenial::ForeignOwner
            | RelationalPublicationReadDependencyDenial::AlreadyRequired
            | RelationalPublicationReadDependencyDenial::ConflictingSource => operator(),
        },
        D::AlreadyRequired
        | D::UnsupportedComponentPlan
        | D::CurrentReadRequired
        | D::CutoverNotPerformed
        | D::DurabilityUnsupported => operator(),
    }
}

pub(super) fn service(
    cause: RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>,
) -> BankHttpDenial {
    match cause {
        RuntimeWorldServiceDenial::OwnerUnavailable(_) => unavailable(),
        RuntimeWorldServiceDenial::Denied(cause) => branch(cause),
    }
}

fn branch(cause: RuntimeWorldBranchAdmissionDenial) -> BankHttpDenial {
    use RuntimeWorldBranchAdmissionDenial as D;
    match cause {
        D::CancelledBeforeEffect => cancelled(),
        D::DeadlineBeforeEffect => deadline(),
        D::OwnerUnavailable | D::CapacityExhausted | D::CustodyCapacityExhausted => unavailable(),
        D::StaleIncarnation
        | D::StaleSourceHead
        | D::ForkSourceChanged
        | D::RetiredBranch
        | D::HistoryEntryUnavailable => stale(),
        D::DurableBranchPublicationRequired
        | D::HomeCreationRequestRequired
        | D::MemoryCreationRequestRequired
        | D::ProgramSourceRequired
        | D::ForeignProgramSource
        | D::ProgramSourceMismatch
        | D::EmptyNativeForkSource
        | D::ForeignOwner
        | D::DuplicateName
        | D::IdentityExhausted
        | D::PlansOmitted
        | D::UnsupportedRetainedSourcePlan
        | D::RetainedSourceDurabilityUnsupported => operator(),
    }
}

fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Unavailable, Next::Retry)
}
fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}
fn cancelled() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Cancelled, Next::Retry)
}
fn deadline() -> BankHttpDenial {
    BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
}
fn stale() -> BankHttpDenial {
    BankHttpDenial::new(Kind::Stale, Next::Refresh)
}

#[cfg(test)]
mod incarnation_tests {
    use super::*;
    #[test]
    fn stale_world_capabilities_require_refresh_without_requesting_effect_replay() {
        let expected = BankHttpDenial::new(Kind::Stale, Next::Refresh);
        assert_eq!(
            recovery(RuntimeWorldRecoveryDenial::StaleIncarnation),
            expected
        );
        assert_eq!(publication(NoEffectCause::StaleIncarnation), expected);
        assert_eq!(
            branch(RuntimeWorldBranchAdmissionDenial::StaleIncarnation),
            expected
        );
    }
    #[test]
    fn pending_home_program_transition_requires_operator_action() {
        assert_eq!(
            recovery(RuntimeWorldRecoveryDenial::HomeProgramTransitionRequired),
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        );
    }
}
