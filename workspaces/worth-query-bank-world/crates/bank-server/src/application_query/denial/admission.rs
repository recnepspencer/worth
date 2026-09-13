//! Bank kinds for application-query admission denial.

use worth_query_host::facade::admission::{
    application_query::WorthQueryApplicationQueryParameterDenialKind as QueryParameter,
    graph_read_access::WorthQueryGraphReadPlanReviewDenialKind as QueryGraphRead,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind as Query;

use crate::BankAuthorizationDenialKind;

use super::installation::{query_installation, BankApplicationQueryInstallationDenialKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationQueryParameterDenialKind {
    ParameterSetMismatch,
    ParameterTypeMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankGraphReadPlanReviewDenialKind {
    BudgetExceeded,
    RequiredAsyncMaterialization,
    RequiredAccessCapabilityRegistration,
    RequiredPersistentIndex,
    UnsupportedGraphIndexSupport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankApplicationQueryAdmissionDenialKind {
    InstalledQuery(BankApplicationQueryInstallationDenialKind),
    ForeignPrincipal,
    ForeignScope,
    StalePrincipal,
    StaleScope,
    ScopeTypeMismatch,
    Authorization(BankAuthorizationDenialKind),
    Cancelled,
    DeadlineExceeded,
    BasisUnsupported,
    ForeignBasis,
    StaleBasis,
    WrongProviderBasis,
    ExpiredBasis,
    BasisUnavailable,
    ForeignHistoricalReceipt,
    RuntimeSupportUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignContinuation,
    StaleContinuation,
    ContinuationParameterMismatch,
    ContinuationScopeMismatch,
    ContinuationProviderMismatch,
    ContinuationPageWidthUnsupported,
    LaneUnsupported,
    DisclosureGovernanceRequired,
    DisclosureContractInvalid,
    DisclosureAuthorizationMismatch,
    InternalComputationDenied,
    Parameter(BankApplicationQueryParameterDenialKind),
    WorkLimitExceeded,
    CanonicalWorkDenied,
    GraphReadPlan(BankGraphReadPlanReviewDenialKind),
    GraphWorkAdmissionUnavailable,
    ExecutionShapeUnsupported,
}

pub(super) const fn admission(kind: Query) -> BankApplicationQueryAdmissionDenialKind {
    use BankApplicationQueryAdmissionDenialKind as Bank;
    match kind {
        Query::InstalledQuery(kind) => Bank::InstalledQuery(query_installation(kind)),
        Query::ForeignPrincipal => Bank::ForeignPrincipal,
        Query::ForeignScope => Bank::ForeignScope,
        Query::StalePrincipal => Bank::StalePrincipal,
        Query::StaleScope => Bank::StaleScope,
        Query::ScopeTypeMismatch => Bank::ScopeTypeMismatch,
        Query::Authorization(kind) => {
            Bank::Authorization(crate::BankAuthorizationDenial::from_kind(kind, 0).kind())
        }
        Query::Cancelled => Bank::Cancelled,
        Query::DeadlineExceeded => Bank::DeadlineExceeded,
        Query::BasisUnsupported => Bank::BasisUnsupported,
        Query::ForeignBasis => Bank::ForeignBasis,
        Query::StaleBasis => Bank::StaleBasis,
        Query::WrongProviderBasis => Bank::WrongProviderBasis,
        Query::ExpiredBasis => Bank::ExpiredBasis,
        Query::BasisUnavailable => Bank::BasisUnavailable,
        Query::ForeignHistoricalReceipt => Bank::ForeignHistoricalReceipt,
        Query::RuntimeSupportUnavailable => Bank::RuntimeSupportUnavailable,
        Query::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Bank::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Query::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
        Query::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
        Query::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
        Query::ForeignContinuation => Bank::ForeignContinuation,
        Query::StaleContinuation => Bank::StaleContinuation,
        Query::ContinuationParameterMismatch => Bank::ContinuationParameterMismatch,
        Query::ContinuationScopeMismatch => Bank::ContinuationScopeMismatch,
        Query::ContinuationProviderMismatch => Bank::ContinuationProviderMismatch,
        Query::ContinuationPageWidthUnsupported => Bank::ContinuationPageWidthUnsupported,
        Query::LaneUnsupported => Bank::LaneUnsupported,
        Query::DisclosureGovernanceRequired => Bank::DisclosureGovernanceRequired,
        Query::DisclosureContractInvalid => Bank::DisclosureContractInvalid,
        Query::DisclosureAuthorizationMismatch => Bank::DisclosureAuthorizationMismatch,
        Query::InternalComputationDenied => Bank::InternalComputationDenied,
        Query::Parameter(kind) => Bank::Parameter(parameter(kind)),
        Query::WorkLimitExceeded => Bank::WorkLimitExceeded,
        Query::CanonicalWorkDenied => Bank::CanonicalWorkDenied,
        Query::GraphReadPlan(kind) => Bank::GraphReadPlan(graph_read(kind)),
        Query::GraphWorkAdmissionUnavailable => Bank::GraphWorkAdmissionUnavailable,
        Query::ExecutionShapeUnsupported => Bank::ExecutionShapeUnsupported,
    }
}

const fn parameter(kind: QueryParameter) -> BankApplicationQueryParameterDenialKind {
    use BankApplicationQueryParameterDenialKind as Bank;
    match kind {
        QueryParameter::ParameterSetMismatch => Bank::ParameterSetMismatch,
        QueryParameter::ParameterTypeMismatch => Bank::ParameterTypeMismatch,
        QueryParameter::CanonicalEntryBudgetExceeded => Bank::CanonicalEntryBudgetExceeded,
        QueryParameter::CanonicalEncodedByteBudgetExceeded => {
            Bank::CanonicalEncodedByteBudgetExceeded
        }
        QueryParameter::CanonicalDigestSlotRejected => Bank::CanonicalDigestSlotRejected,
    }
}

const fn graph_read(kind: QueryGraphRead) -> BankGraphReadPlanReviewDenialKind {
    use BankGraphReadPlanReviewDenialKind as Bank;
    match kind {
        QueryGraphRead::BudgetExceeded => Bank::BudgetExceeded,
        QueryGraphRead::RequiredAsyncMaterialization => Bank::RequiredAsyncMaterialization,
        QueryGraphRead::RequiredAccessCapabilityRegistration => {
            Bank::RequiredAccessCapabilityRegistration
        }
        QueryGraphRead::RequiredPersistentIndex => Bank::RequiredPersistentIndex,
        QueryGraphRead::UnsupportedGraphIndexSupport => Bank::UnsupportedGraphIndexSupport,
    }
}
