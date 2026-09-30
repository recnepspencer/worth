//! One instruction per authorization cause, shared by every route.
//!
//! A query, a mutation request, an estate operation and a recovery all refuse
//! with the same authorization kinds, so each kind names the one action that
//! can succeed wherever it arises. A lapse of the request itself is cleared by
//! repeating it or signing in again, never by a permission change.

use bank_server::BankAuthorizationDenialKind;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};

pub(super) fn authorization_denial(kind: BankAuthorizationDenialKind) -> BankHttpDenial {
    use BankAuthorizationDenialKind as Authorization;
    let (denial, next) = match kind {
        // The request lapsed before authorization decided.
        Authorization::Cancelled => (Kind::Cancelled, Next::Retry),
        Authorization::DeadlineExceeded => (Kind::DeadlineExceeded, Next::Retry),
        Authorization::ExpiredAuthentication => (Kind::Unauthenticated, Next::Authenticate),
        // Installed meaning, principal, scope or grants moved since the view
        // the request was built from.
        Authorization::StaleInstalledSchema
        | Authorization::StaleInstalledOperation
        | Authorization::StalePrincipal
        | Authorization::StaleScope
        | Authorization::StaleAuthorization
        | Authorization::DelegationLineageChanged => (Kind::Stale, Next::Refresh),
        // The decision itself is too large for its installed bound.
        Authorization::CanonicalWorkDenied | Authorization::GrantSelectionLimitExceeded => {
            (Kind::ResourceExhausted, Next::NarrowRequest)
        }
        // Capacity in use now frees as other work settles.
        Authorization::ActiveSnapshotCapacityExhausted { .. }
        | Authorization::RetentionCapacityExhausted => (Kind::Unavailable, Next::Retry),
        // Identity space never frees without reconfiguration.
        Authorization::SnapshotIdentityExhausted
        | Authorization::RetentionIdentityExhausted
        | Authorization::AdmissionIdentityExhausted => (Kind::Unavailable, Next::ContactOperator),
        Authorization::InvalidOperationInput => (Kind::MalformedRequest, Next::CorrectRequest),
        Authorization::ForeignRuntime
        | Authorization::MutationPreconditionRejected
        | Authorization::CapabilityGrantMissing
        | Authorization::CapabilityAuthorizationMissing
        | Authorization::PurposeMismatch
        | Authorization::ExplicitDenyRuleMatched
        | Authorization::ConflictRuleMatched
        | Authorization::SeparationOfDutyRuleMatched
        | Authorization::DistinctActorRuleMatched
        | Authorization::CapabilityRequired
        | Authorization::CapabilityNotRequired
        | Authorization::CapabilityExpired
        | Authorization::ElevationRequired
        | Authorization::ElevationNotApplicable
        | Authorization::ElevationExpired
        | Authorization::ElevationInactive
        | Authorization::ElevationSelfApproval
        | Authorization::ElevationApproverConflict
        | Authorization::ElevationTransitionRequired
        | Authorization::ElevationLifecycleRoleMismatch
        | Authorization::ElevationRequestRejected
        | Authorization::ElevationApprovalRejected
        | Authorization::ElevationCloseRejected
        | Authorization::MandatoryReviewRejected
        | Authorization::ElevationDurationExceeded
        | Authorization::DelegationRejected
        | Authorization::DelegationTransitionRequired
        | Authorization::DelegationDepthExceeded
        | Authorization::DelegationCycle
        | Authorization::ScopeMismatch
        | Authorization::ProductSecurityBasis(_)
        | Authorization::PermissionDenied => (Kind::PermissionDenied, Next::None),
        Authorization::TrustedTimeUnavailable | Authorization::GraphWorkAdmissionUnavailable => {
            (Kind::Unavailable, Next::Retry)
        }
        // Repeating the request cannot install or repair its policy.
        Authorization::PolicyNotInstalled => (Kind::Unavailable, Next::ContactOperator),
        Authorization::CapabilityProjectionRejected
        | Authorization::ElevationProjectionRejected
        | Authorization::InvalidInstalledPolicy
        | Authorization::RelationalObservationRejected
        | Authorization::BridgeEvaluationRejected
        | Authorization::InconsistentDecision => (Kind::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(denial, next)
}

#[cfg(test)]
#[path = "authorization_denial/instruction_tests.rs"]
mod instruction_tests;
