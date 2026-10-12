//! Admission freshness reads retain request stops and ordinary staleness separately.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAdmissionDenialKind as Admission,
    WorthQueryEntityResolutionDenialKind as Entity,
    WorthQueryPrincipalResolutionDenialKind as Principal,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
pub(super) fn principal(cause: Principal) -> Admission {
    match cause {
        Principal::ExecutionRequest(kind) => Admission::ExecutionRequest(kind),
        Principal::Cancelled => Admission::Cancelled,
        Principal::DeadlineExceeded => Admission::DeadlineExceeded,
        Principal::ForeignPhase
        | Principal::PrimaryGraphNotInstalled
        | Principal::BindingNotInstalled
        | Principal::ForeignRuntime
        | Principal::StaleInstalledSchema
        | Principal::ExpiredAuthentication
        | Principal::BranchMaterializationSuspended
        | Principal::IdentityIndexUnavailable
        | Principal::ProjectionWorkBudgetExceeded
        | Principal::ProjectionPreparationMemoryExhausted
        | Principal::CorruptIdentityIndex
        | Principal::UnknownPrincipal
        | Principal::DisabledPrincipal
        | Principal::AmbiguousPrincipal
        | Principal::MissingPrincipalTarget
        | Principal::AmbiguousPrincipalTarget
        | Principal::WrongPrincipalTargetKind
        | Principal::StalePrincipalProof
        | Principal::ActiveSnapshotCapacityExhausted { .. }
        | Principal::SnapshotIdentityExhausted
        | Principal::RetentionCapacityExhausted
        | Principal::RetentionIdentityExhausted => Admission::StalePrincipal,
    }
}
pub(super) fn entity(cause: Entity) -> Admission {
    match cause {
        Entity::Stopped(cause) => match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None) {
            Ok(kind) => Admission::ExecutionRequest(kind),
            Err(Control::Cancelled) => Admission::Cancelled,
            Err(Control::TimedOut) => Admission::DeadlineExceeded,
        },
        Entity::Refused(_) => Admission::StaleScope,
    }
}
