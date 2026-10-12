//! Admission freshness reads retain request stops and ordinary staleness separately.
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAdmissionDenialKind as Admission,
    WorthQueryEntityResolutionDenialKind as Entity,
    WorthQueryPrincipalResolutionDenialKind as Principal,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
pub(super) fn principal(cause: Principal) -> Admission {
    match cause {
        Principal::Stopped(Ok(kind)) => Admission::ExecutionRequest(kind),
        Principal::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => Admission::Cancelled,
        Principal::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )) => Admission::DeadlineExceeded,
        Principal::Refused(PrincipalRefusal::ForeignPhase)
        | Principal::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
        | Principal::Refused(PrincipalRefusal::BindingNotInstalled)
        | Principal::Refused(PrincipalRefusal::ForeignRuntime)
        | Principal::Refused(PrincipalRefusal::StaleInstalledSchema)
        | Principal::Refused(PrincipalRefusal::ExpiredAuthentication)
        | Principal::Refused(PrincipalRefusal::BranchMaterializationSuspended)
        | Principal::Refused(PrincipalRefusal::IdentityIndexUnavailable)
        | Principal::Refused(PrincipalRefusal::CorruptIdentityIndex)
        | Principal::Refused(PrincipalRefusal::UnknownPrincipal)
        | Principal::Refused(PrincipalRefusal::DisabledPrincipal)
        | Principal::Refused(PrincipalRefusal::AmbiguousPrincipal)
        | Principal::Refused(PrincipalRefusal::MissingPrincipalTarget)
        | Principal::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
        | Principal::Refused(PrincipalRefusal::WrongPrincipalTargetKind)
        | Principal::Refused(PrincipalRefusal::StalePrincipalProof)
        | Principal::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted { .. })
        | Principal::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
        | Principal::Refused(PrincipalRefusal::RetentionCapacityExhausted)
        | Principal::Refused(PrincipalRefusal::RetentionIdentityExhausted) => {
            Admission::StalePrincipal
        }
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
