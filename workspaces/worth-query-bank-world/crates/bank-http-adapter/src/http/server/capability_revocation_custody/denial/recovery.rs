//! Recovery describes the still-retained command; denial never discharges its owner.
use super::*;

pub(super) fn managed(cause: WorthQueryManagedApplicationRecoveryDenial) -> BankHttpDenial {
    use WorthQueryManagedApplicationRecoveryDenial as D;
    match cause {
        D::ExecutionDenied(cause) => advancement_denial::advancement(cause),
        D::Authorization(cause) => authorization_denial::authorization_denial(cause.kind().into()),
        D::ProviderAdmission(cause) => companion(cause),
        D::Demand(cause) => query(WorthQueryApplicationRequestQueryDenial::OutputSettlement(
            cause,
        )),
        D::World(cause) => world(cause),
        D::Adoption(RuntimeWorldSettledRelationalAdoptionDenial::Recovery(cause)) => world(cause),
        D::Adoption(RuntimeWorldSettledRelationalAdoptionDenial::Publication(cause)) => {
            adoption(cause.cause())
        }
        D::ProviderBusy | D::ProviderCapacity | D::ProductUnavailable => unavailable(),
        D::ForeignAdmission | D::BindingMismatch => operator(),
    }
}

fn world(cause: RuntimeWorldRecoveryDenial) -> BankHttpDenial {
    use RuntimeWorldRecoveryDenial as D;
    match cause {
        D::Busy | D::TooYoung | D::OutputCapacityExhausted | D::CallerCapabilityLive => {
            unavailable()
        }
        D::OwnerUnavailable(_)
        | D::ForeignHandle
        | D::MissingRecord
        | D::ClockRegressed
        | D::SettlementRequired
        | D::SettlementEvidenceUnavailable => operator(),
    }
}

fn companion(cause: CompanionPreflightStop) -> BankHttpDenial {
    use CompanionPreflightStop as D;
    match cause {
        D::Execution(cause) => crate::http::server::relational_execution_denial::denial(cause),
        D::Interrupted(cause) => match cause.interruption() {
            RelationalOperationInterruption::Cancelled => {
                BankHttpDenial::new(Kind::Cancelled, Next::Retry)
            }
            RelationalOperationInterruption::TimedOut => {
                BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
            }
        },
        D::TopologyPending | D::RetainedCompanionCapacityExhausted { .. } => unavailable(),
        D::SelectedSourceMismatch
        | D::SelectedPositionUnavailable { .. }
        | D::RegistrationChanged => BankHttpDenial::new(Kind::Stale, Next::Refresh),
        D::WorkExhausted { .. }
        | D::WorkExhaustedWithoutRequirement { .. }
        | D::PreparationMemoryExhausted { .. } => {
            BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest)
        }
        D::ForeignCell | D::WorkCounterOverflow | D::PreparationMemoryCounterOverflow => operator(),
    }
}

fn adoption(cause: NoEffectCause) -> BankHttpDenial {
    match cause {
        NoEffectCause::ExecutionRequest(cause) => advancement_denial::request(cause),
        NoEffectCause::CancelledBeforeEffect => BankHttpDenial::new(Kind::Cancelled, Next::Retry),
        NoEffectCause::DeadlineBeforeEffect => {
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
        }
        NoEffectCause::StaleExpectedProductHead => BankHttpDenial::new(Kind::Stale, Next::Refresh),
        NoEffectCause::CapacityExhausted => unavailable(),
        NoEffectCause::RelationalDeferred(RelationalPublicationDeferred::CompanionPreflight(
            cause,
        )) => companion(cause),
        NoEffectCause::RelationalDeferred(_) => unavailable(),
        NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::PreEffectFailure => operator(),
    }
}
