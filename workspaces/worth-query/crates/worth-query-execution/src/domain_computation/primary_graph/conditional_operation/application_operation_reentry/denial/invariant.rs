//! Invariant reads preserve their native stop; local work bounds are execution work.
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryInvariantProjectionDenial as Failure, WorthQueryInvariantProjectionDenialKind as Kind};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_invariant(
        denial: Failure,
    ) -> Self {
        if let Some(stop) = Self::retention_stop(&denial) {
            return stop;
        }
        Self::invariant_kind(denial.kind(), Terminal::Invariant)
    }
    pub(super) fn invariant_kind(kind: Kind, refused: impl FnOnce(Kind) -> Terminal) -> Self {
        match kind {
            Kind::ExecutionRequest(cause) => Self::from_native_stop(cause),
            Kind::WorkBudgetExceeded => Self::from_native_stop(Native::WorkExhausted),
            Kind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::RetentionCapacityExhausted => Self::RetentionCapacityExhausted,
            kind @ (Kind::BasisUnavailable
            | Kind::ForeignPhase
            | Kind::ForeignBasis
            | Kind::SnapshotIdentityExhausted
            | Kind::RetentionIdentityExhausted
            | Kind::SourceRetentionDenied) => Self::Terminal(refused(kind)),
        }
    }
}
