//! Product-facing explanations derived from native denial kinds.
use super::*;

/// Coarse, explainable cause of an authorization denial, fit to show a user.
///
/// Only policy-level denials have one; operational causes such as deadlines or
/// capacity do not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationAuthorizationExplanationCause {
    /// The principal lacks a required capability.
    MissingCapability,
    /// An explicit deny rule applied.
    ExplicitPolicyDenial,
    /// The scope is outside what the grant covers.
    ScopeMismatch,
    /// The declared purpose does not match the grant.
    PurposeMismatch,
    /// A conflict rule applied.
    Conflict,
    /// A separation-of-duty or distinct-actor rule applied.
    SeparationOfDuty,
    /// The action requires an approved elevation.
    ElevationRequired,
    /// The elevation or its lifecycle step was refused.
    ElevationDenied,
    /// The capability or elevation has expired.
    ElevationExpired,
}

impl WorthQueryApplicationAuthorizationExplanationCause {
    pub(super) const fn from_denial_kind(
        kind: WorthQueryOperationAuthorizationDenialKind,
    ) -> Option<Self> {
        use WorthQueryOperationAuthorizationDenialKind as Denial;
        match kind {
            Denial::Refused(AuthorizationRefusal::CapabilityGrantMissing)
            | Denial::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
            | Denial::Refused(AuthorizationRefusal::CapabilityRequired) => {
                Some(Self::MissingCapability)
            }
            Denial::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched) => {
                Some(Self::ExplicitPolicyDenial)
            }
            Denial::Refused(AuthorizationRefusal::ScopeMismatch) => Some(Self::ScopeMismatch),
            Denial::Refused(AuthorizationRefusal::PurposeMismatch) => Some(Self::PurposeMismatch),
            Denial::Refused(AuthorizationRefusal::ConflictRuleMatched) => Some(Self::Conflict),
            Denial::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched)
            | Denial::Refused(AuthorizationRefusal::DistinctActorRuleMatched) => {
                Some(Self::SeparationOfDuty)
            }
            Denial::Refused(AuthorizationRefusal::ElevationRequired) => {
                Some(Self::ElevationRequired)
            }
            Denial::Refused(AuthorizationRefusal::CapabilityExpired)
            | Denial::Refused(AuthorizationRefusal::ElevationExpired) => {
                Some(Self::ElevationExpired)
            }
            Denial::Refused(AuthorizationRefusal::ElevationProjectionRejected)
            | Denial::Refused(AuthorizationRefusal::ElevationInactive)
            | Denial::Refused(AuthorizationRefusal::ElevationSelfApproval)
            | Denial::Refused(AuthorizationRefusal::ElevationApproverConflict)
            | Denial::Refused(AuthorizationRefusal::ElevationTransitionRequired)
            | Denial::Refused(AuthorizationRefusal::ElevationLifecycleRoleMismatch)
            | Denial::Refused(AuthorizationRefusal::ElevationRequestRejected)
            | Denial::Refused(AuthorizationRefusal::ElevationApprovalRejected)
            | Denial::Refused(AuthorizationRefusal::ElevationCloseRejected)
            | Denial::Refused(AuthorizationRefusal::MandatoryReviewRejected)
            | Denial::Refused(AuthorizationRefusal::ElevationDurationExceeded) => {
                Some(Self::ElevationDenied)
            }
            Denial::Stopped(_) | Denial::Refused(_) => None,
        }
    }
}
