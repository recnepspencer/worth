//! Recovery describes the still-retained command; denial never discharges its owner.
use super::*;
use worth_query_host::facade::primary_graph::{
    CurrentProductReadDependencyDenial, RelationalPublicationReadDependencyDenial,
    RuntimeWorldBranchAdmissionDenial,
};

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
        D::Adoption(cause) => match *cause {
            RuntimeWorldSettledRelationalAdoptionDenial::Recovery(cause) => world(cause),
            RuntimeWorldSettledRelationalAdoptionDenial::Publication(cause) => {
                adoption(cause.cause())
            }
        },
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

use crate::http::server::relational_execution_denial::companion;

fn adoption(cause: NoEffectCause) -> BankHttpDenial {
    match cause {
        NoEffectCause::ExecutionRequest(cause) => advancement_denial::request(cause),
        NoEffectCause::CancelledBeforeEffect => BankHttpDenial::new(Kind::Cancelled, Next::Retry),
        NoEffectCause::DeadlineBeforeEffect => {
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
        }
        NoEffectCause::CurrentReadDependency(cause) => current_read(cause),
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

fn current_read(cause: CurrentProductReadDependencyDenial) -> BankHttpDenial {
    use CurrentProductReadDependencyDenial as Read;
    use RelationalPublicationReadDependencyDenial as Native;
    use RuntimeWorldBranchAdmissionDenial as Branch;
    match cause {
        Read::CutoverDeadlineElapsed | Read::Branch(Branch::DeadlineBeforeEffect) => {
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
        }
        Read::Branch(Branch::CancelledBeforeEffect) => {
            BankHttpDenial::new(Kind::Cancelled, Next::Retry)
        }
        Read::Branch(_) | Read::Native(Native::NotCurrent) => {
            BankHttpDenial::new(Kind::Stale, Next::Refresh)
        }
        Read::WorldHeadUnavailable | Read::Native(Native::CandidateUnavailable) => unavailable(),
        Read::AlreadyRequired
        | Read::UnsupportedComponentPlan
        | Read::CurrentReadRequired
        | Read::CutoverNotPerformed
        | Read::DurabilityUnsupported
        | Read::Native(
            Native::EmptySourceUnsupported
            | Native::ForeignOwner
            | Native::AlreadyRequired
            | Native::ConflictingSource,
        ) => operator(),
    }
}
#[cfg(test)]
mod current_read_tests {
    use super::*;
    #[test]
    fn current_read_controls_keep_their_original_http_instruction() {
        for (cause, kind) in [
            (
                CurrentProductReadDependencyDenial::CutoverDeadlineElapsed,
                Kind::DeadlineExceeded,
            ),
            (
                CurrentProductReadDependencyDenial::Branch(
                    RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect,
                ),
                Kind::DeadlineExceeded,
            ),
            (
                CurrentProductReadDependencyDenial::Branch(
                    RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect,
                ),
                Kind::Cancelled,
            ),
        ] {
            assert_eq!(
                adoption(NoEffectCause::CurrentReadDependency(cause)),
                BankHttpDenial::new(kind, Next::Retry)
            );
        }
    }
}
