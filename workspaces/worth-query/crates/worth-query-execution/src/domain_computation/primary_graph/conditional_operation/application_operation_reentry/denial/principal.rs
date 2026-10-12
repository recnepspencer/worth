//! principal refusal at conditional reentry.
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryPrincipalResolutionDenialKind as Kind, WorthQueryPrincipalResolutionDenial as Failure};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_principal(
        denial: Failure,
    ) -> Self {
        Self::principal_kind(denial.kind(), Terminal::Principal)
    }
    pub(super) fn principal_kind(kind: Kind, refused: impl FnOnce(Kind) -> Terminal) -> Self {
        match kind {
            Kind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::RetentionCapacityExhausted => Self::RetentionCapacityExhausted,
            Kind::ExecutionRequest(cause) => Self::from_read_stop(Ok(cause)),
            Kind::Cancelled => Self::from_native_stop(Native::Cancelled),
            Kind::DeadlineExceeded => Self::from_native_stop(Native::DeadlineElapsed),
            Kind::ProjectionWorkBudgetExceeded => Self::from_native_stop(Native::WorkExhausted),
            Kind::ProjectionPreparationMemoryExhausted => {
                Self::from_native_stop(Native::MemoryLimitExceedsParent)
            }
            kind @ (Kind::ForeignPhase
            | Kind::PrimaryGraphNotInstalled
            | Kind::BindingNotInstalled
            | Kind::ForeignRuntime
            | Kind::StaleInstalledSchema
            | Kind::ExpiredAuthentication
            | Kind::BranchMaterializationSuspended
            | Kind::IdentityIndexUnavailable
            | Kind::CorruptIdentityIndex
            | Kind::UnknownPrincipal
            | Kind::DisabledPrincipal
            | Kind::AmbiguousPrincipal
            | Kind::MissingPrincipalTarget
            | Kind::AmbiguousPrincipalTarget
            | Kind::WrongPrincipalTargetKind
            | Kind::StalePrincipalProof
            | Kind::SnapshotIdentityExhausted
            | Kind::RetentionIdentityExhausted) => Self::Terminal(refused(kind)),
        }
    }
}
