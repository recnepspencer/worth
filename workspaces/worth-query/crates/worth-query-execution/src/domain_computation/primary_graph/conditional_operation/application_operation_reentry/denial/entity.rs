//! An index error names its stop before its refusal kind is inspected.
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryEntityResolutionDenial as Failure, WorthQueryEntityResolutionDenialKind as Error, WorthQueryEntityResolutionRefusalKind as Kind};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_entity(
        denial: Failure,
    ) -> Self {
        match denial.kind() {
            Error::Stopped(stop) => Self::from_native_stop(stop),
            Error::Refused(kind) => match kind {
                Kind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => Self::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
                Kind::RetentionCapacityExhausted => Self::RetentionCapacityExhausted,
                kind @ (Kind::ForeignPhase
                | Kind::PrimaryGraphNotInstalled
                | Kind::FieldNotInstalled
                | Kind::ValueEncodingRejected
                | Kind::EqualityIndexUnavailable
                | Kind::UnknownEntity
                | Kind::AmbiguousEntity
                | Kind::CorruptIdentityIndex
                | Kind::InvalidCandidateLimit
                | Kind::CandidateLimitExceeded { .. }
                | Kind::SnapshotIdentityExhausted
                | Kind::RetentionIdentityExhausted
                | Kind::ForeignResolutionTruth) => Self::Terminal(Terminal::Entity(kind)),
            },
        }
    }
}
