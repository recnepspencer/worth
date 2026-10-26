//! Bank kinds for application-query execution lanes.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationContinuationDenialKind as QueryContinuation,
    WorthQueryApplicationLiveOpenDenialKind as QueryLive,
    WorthQueryApplicationOneShotDenialKind as QueryOneShot,
    WorthQueryApplicationProjectionDenialKind as QueryProjection,
    WorthQueryProductBranchAdmissionDenial as QueryProductSelection,
};
use worth_query_host::facade::product::WorthQueryProductBranchAdmissionRefusalKind as QueryRefusal;

use crate::{BankAuthorizationDenial, BankAuthorizationDenialKind};

use super::admission::{admission, BankApplicationQueryAdmissionDenialKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationPreviewSessionDenialKind {
    Cancelled,
    DeadlineExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankProductSelectionDenialKind {
    ReadExecution(
        Result<
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind,
            worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ForeignPhase,
    ObservationStatePoisoned,
    OwnerUnavailable,
    ForeignOwner,
    RetiredBranch,
    IncarnationChanged,
    ObservationRejected,
    ObservationStaleSourceHead,
    ObservationCancelled,
    ObservationDeadlineExceeded,
    ObservationCapacityExhausted,
    CustodyCapacityExhausted,
    ObservationIdentityExhausted,
    ObservationAccountingOverflow,
    ProductActivationUnavailable,
    RelationalBasisUnavailable,
    RelationalSnapshotUnavailable,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgeSourceUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationProjectionDenialKind {
    ValueDecode(worth_query_host::facade::primary_graph::WorthQueryValueDecodeDenial),
    FieldNotProjected,
    FieldContractMismatch,
    FieldTypeMismatch,
    FieldOmitted,
    RelationNotProjected,
    RelationContractMismatch,
    RelationCardinalityMismatch,
    RelationOmitted,
    DomainProjectionRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationOneShotDenialKind {
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    StaleScope,
    Authorization(BankAuthorizationDenialKind),
    Cancelled,
    DeadlineExceeded,
    BasisUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    SourceIdentityExhausted,
    ExpiredBasis,
    BasisReleaseFailed,
    PredicateIndexUnavailable,
    ResultLimitExceeded,
    CardinalityMismatch,
    TraversalUnavailable,
    ProjectionUnavailable,
    ExpressionContextUnavailable,
    ExpressionContextIncomplete,
    Projection(BankApplicationProjectionDenialKind),
    ResultBufferLimitExceeded,
    WorkLimitExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationContinuationDenialKind {
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    StaleScope,
    Authorization(BankAuthorizationDenialKind),
    Cancelled,
    DeadlineExceeded,
    BasisUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ExpiredBasis,
    BasisReleaseFailed,
    PredicateIndexUnavailable,
    ResultLimitExceeded,
    CardinalityMismatch,
    TraversalUnavailable,
    ContinuationIndexUnavailable,
    ContinuationBoundaryRejected,
    ContinuationPageWidthInvalid,
    ContinuationGenerationChanged,
    ProjectionUnavailable,
    Projection(BankApplicationProjectionDenialKind),
    ResultBufferLimitExceeded,
    WorkLimitExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationLiveOpenDenialKind {
    LiveContractMissing,
    BindingMismatch,

    Admission(BankApplicationQueryAdmissionDenialKind),
    AuthorizationDenied(BankAuthorizationDenialKind),
    ScopeIdentityUnavailable,
    BasisReleaseFailed,
    ProviderVersionUnavailable,
    ForeignAdvancementPhase,
    BridgeBasisRejected,
}

pub(super) const fn product_selection(
    kind: QueryProductSelection,
) -> BankProductSelectionDenialKind {
    use BankProductSelectionDenialKind as Bank;
    match kind {
        QueryProductSelection::Stopped(stop) => Bank::ReadExecution(stop),
        QueryProductSelection::Refused(kind) => match kind {
            QueryRefusal::ForeignPhase => Bank::ForeignPhase,
            QueryRefusal::ObservationStatePoisoned => Bank::ObservationStatePoisoned,
            QueryRefusal::OwnerUnavailable => Bank::OwnerUnavailable,
            QueryRefusal::ForeignOwner => Bank::ForeignOwner,
            QueryRefusal::RetiredBranch => Bank::RetiredBranch,
            QueryRefusal::IncarnationChanged => Bank::IncarnationChanged,
            QueryRefusal::ObservationRejected => Bank::ObservationRejected,
            QueryRefusal::ObservationStaleSourceHead => Bank::ObservationStaleSourceHead,
            QueryRefusal::ObservationCapacityExhausted => Bank::ObservationCapacityExhausted,
            QueryRefusal::CustodyCapacityExhausted => Bank::CustodyCapacityExhausted,
            QueryRefusal::ObservationIdentityExhausted => Bank::ObservationIdentityExhausted,
            QueryRefusal::ProductActivationUnavailable => Bank::ProductActivationUnavailable,
            QueryRefusal::RelationalBasisUnavailable => Bank::RelationalBasisUnavailable,
            QueryRefusal::RelationalSnapshotUnavailable => Bank::RelationalSnapshotUnavailable,
            QueryRefusal::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Bank::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            QueryRefusal::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
            QueryRefusal::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
            QueryRefusal::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
            QueryRefusal::BridgeSourceUnavailable => Bank::BridgeSourceUnavailable,
        },
    }
}

pub(super) const fn one_shot(kind: QueryOneShot) -> BankApplicationOneShotDenialKind {
    use BankApplicationOneShotDenialKind as Bank;
    match kind {
        QueryOneShot::ExecutionRequest(cause) => Bank::ExecutionRequest(cause),
        QueryOneShot::ForeignPlan => Bank::ForeignPlan,
        QueryOneShot::StaleInstalledQuery => Bank::StaleInstalledQuery,
        QueryOneShot::StalePrincipal => Bank::StalePrincipal,
        QueryOneShot::StaleScope => Bank::StaleScope,
        QueryOneShot::Authorization(kind) => Bank::Authorization(authorization(kind)),
        QueryOneShot::Cancelled => Bank::Cancelled,
        QueryOneShot::DeadlineExceeded => Bank::DeadlineExceeded,
        QueryOneShot::BasisUnavailable => Bank::BasisUnavailable,
        QueryOneShot::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Bank::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        QueryOneShot::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
        QueryOneShot::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
        QueryOneShot::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
        QueryOneShot::SourceIdentityExhausted => Bank::SourceIdentityExhausted,
        QueryOneShot::ExpiredBasis => Bank::ExpiredBasis,
        QueryOneShot::BasisReleaseFailed => Bank::BasisReleaseFailed,
        QueryOneShot::PredicateIndexUnavailable => Bank::PredicateIndexUnavailable,
        QueryOneShot::ResultLimitExceeded => Bank::ResultLimitExceeded,
        QueryOneShot::CardinalityMismatch => Bank::CardinalityMismatch,
        QueryOneShot::TraversalUnavailable => Bank::TraversalUnavailable,
        QueryOneShot::ProjectionUnavailable => Bank::ProjectionUnavailable,
        QueryOneShot::ExpressionContextUnavailable => Bank::ExpressionContextUnavailable,
        QueryOneShot::ExpressionContextIncomplete => Bank::ExpressionContextIncomplete,
        QueryOneShot::Projection(kind) => Bank::Projection(projection(kind)),
        QueryOneShot::ResultBufferLimitExceeded => Bank::ResultBufferLimitExceeded,
        QueryOneShot::WorkLimitExceeded => Bank::WorkLimitExceeded,
    }
}

pub(super) const fn continuation(kind: QueryContinuation) -> BankApplicationContinuationDenialKind {
    use BankApplicationContinuationDenialKind as Bank;
    match kind {
        QueryContinuation::ExecutionRequest(cause) => Bank::ExecutionRequest(cause),
        QueryContinuation::ForeignPlan => Bank::ForeignPlan,
        QueryContinuation::StaleInstalledQuery => Bank::StaleInstalledQuery,
        QueryContinuation::StalePrincipal => Bank::StalePrincipal,
        QueryContinuation::StaleScope => Bank::StaleScope,
        QueryContinuation::Authorization(kind) => Bank::Authorization(authorization(kind)),
        QueryContinuation::Cancelled => Bank::Cancelled,
        QueryContinuation::DeadlineExceeded => Bank::DeadlineExceeded,
        QueryContinuation::BasisUnavailable => Bank::BasisUnavailable,
        QueryContinuation::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
        QueryContinuation::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
        QueryContinuation::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
        QueryContinuation::ExpiredBasis => Bank::ExpiredBasis,
        QueryContinuation::BasisReleaseFailed => Bank::BasisReleaseFailed,
        QueryContinuation::PredicateIndexUnavailable => Bank::PredicateIndexUnavailable,
        QueryContinuation::ResultLimitExceeded => Bank::ResultLimitExceeded,
        QueryContinuation::CardinalityMismatch => Bank::CardinalityMismatch,
        QueryContinuation::TraversalUnavailable => Bank::TraversalUnavailable,
        QueryContinuation::ContinuationIndexUnavailable => Bank::ContinuationIndexUnavailable,
        QueryContinuation::ContinuationBoundaryRejected => Bank::ContinuationBoundaryRejected,
        QueryContinuation::ContinuationPageWidthInvalid => Bank::ContinuationPageWidthInvalid,
        QueryContinuation::ContinuationGenerationChanged => Bank::ContinuationGenerationChanged,
        QueryContinuation::ProjectionUnavailable => Bank::ProjectionUnavailable,
        QueryContinuation::Projection(kind) => Bank::Projection(projection(kind)),
        QueryContinuation::ResultBufferLimitExceeded => Bank::ResultBufferLimitExceeded,
        QueryContinuation::WorkLimitExceeded => Bank::WorkLimitExceeded,
    }
}

pub(super) const fn live(kind: QueryLive) -> BankApplicationLiveOpenDenialKind {
    use BankApplicationLiveOpenDenialKind as Bank;
    match kind {
        QueryLive::LiveContractMissing => Bank::LiveContractMissing,
        QueryLive::BindingMismatch => Bank::BindingMismatch,

        QueryLive::Admission(kind) => Bank::Admission(admission(kind)),
        QueryLive::AuthorizationDenied(kind) => Bank::AuthorizationDenied(authorization(kind)),
        QueryLive::ScopeIdentityUnavailable => Bank::ScopeIdentityUnavailable,
        QueryLive::BasisReleaseFailed => Bank::BasisReleaseFailed,
        QueryLive::ProviderVersionUnavailable => Bank::ProviderVersionUnavailable,
        QueryLive::ForeignAdvancementPhase => Bank::ForeignAdvancementPhase,
        QueryLive::BridgeBasisRejected => Bank::BridgeBasisRejected,
    }
}

const fn projection(kind: QueryProjection) -> BankApplicationProjectionDenialKind {
    use BankApplicationProjectionDenialKind as Bank;
    match kind {
        QueryProjection::ValueDecode(cause) => Bank::ValueDecode(cause),
        QueryProjection::FieldNotProjected => Bank::FieldNotProjected,
        QueryProjection::FieldContractMismatch => Bank::FieldContractMismatch,
        QueryProjection::FieldTypeMismatch => Bank::FieldTypeMismatch,
        QueryProjection::FieldOmitted => Bank::FieldOmitted,
        QueryProjection::RelationNotProjected => Bank::RelationNotProjected,
        QueryProjection::RelationContractMismatch => Bank::RelationContractMismatch,
        QueryProjection::RelationCardinalityMismatch => Bank::RelationCardinalityMismatch,
        QueryProjection::RelationOmitted => Bank::RelationOmitted,
        QueryProjection::DomainProjectionRejected => Bank::DomainProjectionRejected,
    }
}

pub(super) const fn authorization(
    kind: worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind,
) -> BankAuthorizationDenialKind {
    BankAuthorizationDenial::from_kind(kind, 0).kind()
}
