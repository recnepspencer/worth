//! principal refusal at conditional reentry.
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryPrincipalResolutionDenialKind as Kind, WorthQueryPrincipalResolutionDenial as Failure};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_principal(
        denial: Failure,
    ) -> Self {
        Self::principal_kind(denial.kind(), Terminal::Principal)
    }
    pub(super) fn principal_kind(kind: Kind, refused: impl FnOnce(Kind) -> Terminal) -> Self {
        match kind {
            Kind::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            }) => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::Refused(PrincipalRefusal::RetentionCapacityExhausted) => {
                Self::RetentionCapacityExhausted
            }
            Kind::Stopped(stop) => Self::from_read_stop(stop),
            kind @ (Kind::Refused(PrincipalRefusal::ForeignPhase)
            | Kind::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
            | Kind::Refused(PrincipalRefusal::BindingNotInstalled)
            | Kind::Refused(PrincipalRefusal::ForeignRuntime)
            | Kind::Refused(PrincipalRefusal::StaleInstalledSchema)
            | Kind::Refused(PrincipalRefusal::ExpiredAuthentication)
            | Kind::Refused(PrincipalRefusal::BranchMaterializationSuspended)
            | Kind::Refused(PrincipalRefusal::IdentityIndexUnavailable)
            | Kind::Refused(PrincipalRefusal::CorruptIdentityIndex)
            | Kind::Refused(PrincipalRefusal::UnknownPrincipal)
            | Kind::Refused(PrincipalRefusal::DisabledPrincipal)
            | Kind::Refused(PrincipalRefusal::AmbiguousPrincipal)
            | Kind::Refused(PrincipalRefusal::MissingPrincipalTarget)
            | Kind::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
            | Kind::Refused(PrincipalRefusal::WrongPrincipalTargetKind)
            | Kind::Refused(PrincipalRefusal::StalePrincipalProof)
            | Kind::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
            | Kind::Refused(PrincipalRefusal::RetentionIdentityExhausted)) => {
                Self::Terminal(refused(kind))
            }
        }
    }
}
