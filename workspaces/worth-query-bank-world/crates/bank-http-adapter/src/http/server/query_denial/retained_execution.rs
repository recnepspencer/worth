//! Retained query error wrappers keep the original content request stop.
use super::*;
use bank_server::{BankApplicationContinuationDenialKind, BankApplicationLiveOpenDenialKind};
use bank_server::{
    BankApplicationQueryAdmissionDenialKind as Admission,
    BankAuthorizationDenialKind as Authorization,
};
use worth_query_host::facade::primary_graph::{
    RelationalExecutionDenialCause as Native, WorthQueryProviderSessionDenialKind as Session,
};

pub(super) fn continuation(cause: BankApplicationContinuationDenialKind) -> BankHttpDenial {
    use BankApplicationContinuationDenialKind as D;
    match cause {
        D::Content(cause) => {
            crate::http::server::relational_execution_denial::denial(cause.into_execution_cause())
        }
        D::ExecutionRequest(cause @ (Session::Content(_) | Session::ContentInconsistency)) => {
            crate::http::server::mutation_application::pending_execution_denial(cause)
        }
        D::Authorization(
            cause @ (Authorization::InputNotAdmitted
            | Authorization::ExecutionRequest(
                Session::Content(_) | Session::ContentInconsistency,
            )
            | Authorization::RelationalExecution(Native::Content(_))),
        ) => authorization_denial(cause),
        D::ExecutionRequest(_)
        | D::ForeignPlan
        | D::StaleInstalledQuery
        | D::StalePrincipal
        | D::StaleScope
        | D::Authorization(_)
        | D::Cancelled
        | D::DeadlineExceeded
        | D::BasisUnavailable
        | D::RetentionCapacityExhausted
        | D::RetentionIdentityExhausted
        | D::SnapshotIdentityExhausted
        | D::ExpiredBasis
        | D::BasisReleaseFailed
        | D::PredicateIndexUnavailable
        | D::ResultLimitExceeded
        | D::CardinalityMismatch
        | D::TraversalUnavailable
        | D::ContinuationIndexUnavailable
        | D::ContinuationBoundaryRejected
        | D::ContinuationPageWidthInvalid
        | D::ContinuationGenerationChanged
        | D::ProjectionUnavailable
        | D::Projection(_)
        | D::ResultBufferLimitExceeded
        | D::WorkLimitExceeded => unavailable(),
    }
}
pub(super) fn live(cause: BankApplicationLiveOpenDenialKind) -> BankHttpDenial {
    use BankApplicationLiveOpenDenialKind as D;
    match cause {
        D::Content(cause) => {
            crate::http::server::relational_execution_denial::denial(cause.into_execution_cause())
        }
        D::AuthorizationDenied(
            cause @ (Authorization::InputNotAdmitted
            | Authorization::ExecutionRequest(
                Session::Content(_) | Session::ContentInconsistency,
            )
            | Authorization::RelationalExecution(Native::Content(_))),
        ) => authorization_denial(cause),
        D::Admission(
            cause @ (Admission::ExecutionRequest(
                Session::Content(_) | Session::ContentInconsistency,
            )
            | Admission::Authorization(
                Authorization::InputNotAdmitted
                | Authorization::ExecutionRequest(
                    Session::Content(_) | Session::ContentInconsistency,
                )
                | Authorization::RelationalExecution(Native::Content(_)),
            )),
        ) => admission(cause),
        D::LiveContractMissing
        | D::BindingMismatch
        | D::Admission(_)
        | D::AuthorizationDenied(_)
        | D::ScopeIdentityUnavailable
        | D::BasisReleaseFailed
        | D::ProviderVersionUnavailable
        | D::ForeignAdvancementPhase
        | D::BridgeBasisRejected => unavailable(),
    }
}
