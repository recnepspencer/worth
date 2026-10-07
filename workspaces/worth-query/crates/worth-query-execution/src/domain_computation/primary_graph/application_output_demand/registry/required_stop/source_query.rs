//! Which retained source-query stops the row decides. The re-read runs under
//! the caller's own principal, scope, authorization and freshly selected
//! basis, while the row is keyed by producer and source epoch alone, so a stop
//! that depends on any of those, or on a shared resource that frees again,
//! stays with the caller.

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationQueryAdmissionDenialKind,
};

pub(super) fn admission_is_row(kind: WorthQueryApplicationQueryAdmissionDenialKind) -> bool {
    use WorthQueryApplicationQueryAdmissionDenialKind as Kind;
    match kind {
        Kind::Handle(_) => false,
        Kind::ForeignPrincipal
        | Kind::ForeignScope
        | Kind::StalePrincipal
        | Kind::StaleScope
        | Kind::ScopeTypeMismatch
        | Kind::Authorization(_)
        | Kind::DisclosureGovernanceRequired
        | Kind::DisclosureAuthorizationMismatch
        | Kind::InternalComputationDenied
        | Kind::Cancelled
        | Kind::DeadlineExceeded
        | Kind::WorkLimitExceeded
        | Kind::CanonicalWorkDenied
        | Kind::ReadmissionPreparationMemoryExhausted
        | Kind::StaleBasis
        | Kind::ExpiredBasis
        | Kind::BasisUnavailable
        | Kind::BranchMaterializationSuspended
        | Kind::ActiveSnapshotCapacityExhausted { .. }
        | Kind::RetentionCapacityExhausted
        | Kind::GraphWorkAdmissionUnavailable => false,
        Kind::InstalledQuery(_)
        | Kind::BasisUnsupported
        | Kind::ForeignBasis
        | Kind::WrongProviderBasis
        | Kind::ForeignHistoricalReceipt
        | Kind::RuntimeSupportUnavailable
        | Kind::SnapshotIdentityExhausted
        | Kind::RetentionIdentityExhausted
        | Kind::ForeignContinuation
        | Kind::StaleContinuation
        | Kind::ContinuationParameterMismatch
        | Kind::ContinuationScopeMismatch
        | Kind::ContinuationProviderMismatch
        | Kind::ContinuationPageWidthUnsupported
        | Kind::LaneUnsupported
        | Kind::DisclosureContractInvalid
        | Kind::Parameter(_)
        | Kind::GraphReadPlan(_)
        | Kind::ExecutionShapeUnsupported => true,
    }
}

pub(super) fn execution_is_row(kind: WorthQueryApplicationOneShotDenialKind) -> bool {
    use WorthQueryApplicationOneShotDenialKind as Kind;
    match kind {
        Kind::StalePrincipal
        | Kind::StaleScope
        | Kind::Authorization(_)
        | Kind::Cancelled
        | Kind::DeadlineExceeded
        | Kind::WorkLimitExceeded
        | Kind::ExpiredBasis
        | Kind::BasisUnavailable
        | Kind::BasisReleaseFailed
        | Kind::ActiveSnapshotCapacityExhausted { .. }
        | Kind::RetentionCapacityExhausted
        // An admission-to-execution race; the next advance reports the installation.
        | Kind::StaleInstalledQuery => false,
        Kind::Projection(kind) => projection_is_row(kind),
        Kind::ForeignPlan
        | Kind::RetentionIdentityExhausted
        | Kind::SnapshotIdentityExhausted
        | Kind::PredicateIndexUnavailable
        | Kind::PredicateLookupOverflow
        | Kind::ResultLimitExceeded
        | Kind::CardinalityMismatch
        | Kind::TraversalUnavailable
        | Kind::ProjectionUnavailable
        | Kind::ResultBufferLimitExceeded
        | Kind::SourceIdentityExhausted => true,
    }
}

/// Governed disclosure withholds per caller; the rest is the row's shape.
fn projection_is_row(kind: WorthQueryApplicationProjectionDenialKind) -> bool {
    use WorthQueryApplicationProjectionDenialKind as Kind;
    match kind {
        Kind::FieldOmitted | Kind::RelationOmitted => false,
        Kind::FieldNotProjected
        | Kind::FieldContractMismatch
        | Kind::FieldTypeMismatch
        | Kind::RelationNotProjected
        | Kind::RelationContractMismatch
        | Kind::RelationCardinalityMismatch
        | Kind::DomainProjectionRejected => true,
    }
}
