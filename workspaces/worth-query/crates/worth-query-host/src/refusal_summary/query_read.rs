//! Read-path refusals classified by their own kind, never by the outer wrapper.
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationContinuationDenial, WorthQueryApplicationContinuationDenialKind as Page,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind as Read,
    WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as Admit, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind as Resolve,
    WorthQueryEntityResolutionRefusalKind as Refused,
};

use super::request::{
    BAD_INPUT, BUSY, CANCELLED, CAPACITY, INTERNAL, MISSING_ITEM, PERMISSION, STALE, STORAGE,
    TIMED_OUT,
};
use super::stops::{authorization, native, projection, session};
use super::summary::{WorthQueryRefusalSummary as Summary, WorthQueryUserFacingRefusal};

impl WorthQueryUserFacingRefusal for WorthQueryApplicationOneShotDenial {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Read::StalePrincipal => PERMISSION,
            Read::Authorization(kind) => authorization(&kind),
            Read::Projection(kind) => projection(&kind),
            Read::ExecutionRequest(kind) => session(&kind),
            Read::StaleScope => MISSING_ITEM,
            Read::Cancelled => CANCELLED,
            Read::DeadlineExceeded => TIMED_OUT,
            Read::StaleInstalledQuery => STALE,
            Read::Content(_) => STORAGE,
            Read::BasisUnavailable
            | Read::ExpiredBasis
            | Read::BasisReleaseFailed
            | Read::PredicateIndexUnavailable
            | Read::TraversalUnavailable
            | Read::ProjectionUnavailable
            | Read::ExpressionContextUnavailable
            | Read::ExpressionContextIncomplete
            | Read::ProjectionInspectionUnavailable
            | Read::ProjectionInspectionIncomplete => BUSY,
            Read::ActiveSnapshotCapacityExhausted { .. }
            | Read::RetentionCapacityExhausted
            | Read::RetentionIdentityExhausted
            | Read::SnapshotIdentityExhausted
            | Read::ResultLimitExceeded
            | Read::ResultBufferLimitExceeded
            | Read::WorkLimitExceeded
            | Read::SourceIdentityExhausted
            | Read::SourceObservationAllocation { .. } => CAPACITY,
            Read::ForeignPlan | Read::CardinalityMismatch => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationContinuationDenial {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Page::StalePrincipal => PERMISSION,
            Page::Authorization(kind) => authorization(&kind),
            Page::Projection(kind) => projection(&kind),
            Page::ExecutionRequest(kind) => session(&kind),
            Page::StaleScope => MISSING_ITEM,
            Page::Cancelled => CANCELLED,
            Page::DeadlineExceeded => TIMED_OUT,
            Page::StaleInstalledQuery
            | Page::ContinuationBoundaryRejected
            | Page::ContinuationGenerationChanged => STALE,
            Page::Content(_) => STORAGE,
            Page::BasisUnavailable
            | Page::ExpiredBasis
            | Page::BasisReleaseFailed
            | Page::PredicateIndexUnavailable
            | Page::TraversalUnavailable
            | Page::ContinuationIndexUnavailable
            | Page::ProjectionUnavailable => BUSY,
            Page::RetentionCapacityExhausted
            | Page::RetentionIdentityExhausted
            | Page::SnapshotIdentityExhausted
            | Page::ResultLimitExceeded
            | Page::ResultBufferLimitExceeded
            | Page::WorkLimitExceeded => CAPACITY,
            Page::ForeignPlan | Page::CardinalityMismatch | Page::ContinuationPageWidthInvalid => {
                INTERNAL
            }
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationQueryAdmissionDenial {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Admit::Authorization(kind) => authorization(&kind),
            Admit::ExecutionRequest(kind) => session(&kind),
            Admit::StalePrincipal
            | Admit::DisclosureAuthorizationMismatch
            | Admit::InternalComputationDenied => PERMISSION,
            Admit::StaleScope
            | Admit::HistoricalSourceNotLive { .. }
            | Admit::HistoricalSourceKindMismatch { .. }
            | Admit::HistoricalSourceOutsideDomain { .. }
            | Admit::HistoricalSourceOwnershipUnavailable { .. } => MISSING_ITEM,
            Admit::Cancelled => CANCELLED,
            Admit::DeadlineExceeded => TIMED_OUT,
            Admit::StaleBasis | Admit::StaleContinuation => STALE,
            Admit::Parameter(_) => BAD_INPUT,
            Admit::ExpiredBasis
            | Admit::BasisUnavailable
            | Admit::BranchMaterializationSuspended
            | Admit::RuntimeSupportUnavailable
            | Admit::GraphWorkAdmissionUnavailable => BUSY,
            Admit::ActiveSnapshotCapacityExhausted { .. }
            | Admit::SnapshotIdentityExhausted
            | Admit::RetentionCapacityExhausted
            | Admit::RetentionIdentityExhausted
            | Admit::WorkLimitExceeded
            | Admit::ReadmissionPreparationMemoryExhausted
            | Admit::CanonicalWorkDenied => CAPACITY,
            Admit::InstalledQuery(_)
            | Admit::ForeignPrincipal
            | Admit::ForeignScope
            | Admit::ScopeTypeMismatch
            | Admit::BasisUnsupported
            | Admit::ForeignBasis
            | Admit::WrongProviderBasis
            | Admit::ForeignHistoricalReceipt
            | Admit::ForeignContinuation
            | Admit::ContinuationParameterMismatch
            | Admit::ContinuationScopeMismatch
            | Admit::ContinuationProviderMismatch
            | Admit::ContinuationPageWidthUnsupported
            | Admit::LaneUnsupported
            | Admit::DisclosureGovernanceRequired
            | Admit::DisclosureContractInvalid
            | Admit::GraphReadPlan(_)
            | Admit::ExecutionShapeUnsupported => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryEntityResolutionDenial {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Resolve::Stopped(cause) => native(&cause),
            Resolve::Refused(Refused::UnknownEntity) => MISSING_ITEM,
            Resolve::Refused(Refused::EqualityIndexUnavailable) => BUSY,
            Resolve::Refused(Refused::CorruptIdentityIndex) => STORAGE,
            Resolve::Refused(
                Refused::CandidateLimitExceeded { .. }
                | Refused::ActiveSnapshotCapacityExhausted { .. }
                | Refused::SnapshotIdentityExhausted
                | Refused::RetentionCapacityExhausted
                | Refused::RetentionIdentityExhausted,
            ) => CAPACITY,
            Resolve::Refused(
                Refused::ForeignPhase
                | Refused::PrimaryGraphNotInstalled
                | Refused::FieldNotInstalled
                | Refused::ValueEncodingRejected
                | Refused::AmbiguousEntity
                | Refused::InvalidCandidateLimit
                | Refused::ForeignResolutionTruth,
            ) => INTERNAL,
        }
    }
}
