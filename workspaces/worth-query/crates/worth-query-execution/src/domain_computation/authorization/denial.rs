//! Typed authorization denial topology.

use crate::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity;

/// The specific reason authorization refused an operation or query.
///
/// Families: cancellation and deadline; stale installed meaning, principal,
/// scope, or authorization; capability, policy, and rule outcomes; elevation
/// and delegation lifecycle; basis, capacity, and identity exhaustion; and
/// internal consistency failures. Every kind is decided at admission, before
/// any effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationAuthorizationDenialKind {
    /// The authorization check was cancelled.
    Cancelled,
    /// The authorization check reached its deadline.
    DeadlineExceeded,
    /// The principal's authentication has expired.
    ExpiredAuthentication,
    /// The authorization material belongs to a different runtime.
    ForeignRuntime,
    /// The installed schema binding changed since the material was admitted.
    StaleInstalledSchema,
    /// The installed operation changed or is no longer installed.
    StaleInstalledOperation,
    /// The principal no longer resolves as it did.
    StalePrincipal,
    /// The request scope no longer resolves as it did.
    StaleScope,
    /// A declared mutation precondition did not hold.
    MutationPreconditionRejected,
    /// The bounded canonical work for the decision was refused.
    CanonicalWorkDenied,
    /// The trusted clock needed for the decision was unavailable.
    TrustedTimeUnavailable,
    /// The operation input could not be evaluated for authorization.
    InvalidOperationInput,
    /// The capability projection read for the decision was rejected.
    CapabilityProjectionRejected,
    /// No capability grant covers the principal for this action.
    CapabilityGrantMissing,
    /// The action needs a capability authorization that was not presented.
    CapabilityAuthorizationMissing,
    /// The declared purpose does not match the grant.
    PurposeMismatch,
    /// An installed deny rule matched.
    ExplicitDenyRuleMatched,
    /// An installed conflict rule matched.
    ConflictRuleMatched,
    /// An installed separation-of-duty rule matched.
    SeparationOfDutyRuleMatched,
    /// An installed distinct-actor rule matched.
    DistinctActorRuleMatched,
    /// The operation requires a capability and none was used.
    CapabilityRequired,
    /// A capability lane was used for an operation that requires no capability.
    CapabilityNotRequired,
    /// The capability grant has expired.
    CapabilityExpired,
    /// The action requires an approved elevation.
    ElevationRequired,
    /// An approved elevation was presented for a capability that declares none.
    ElevationNotApplicable,
    /// The elevation state read for the decision was rejected.
    ElevationProjectionRejected,
    /// The elevation has expired.
    ElevationExpired,
    /// The elevation is not active.
    ElevationInactive,
    /// The requester tried to approve their own elevation.
    ElevationSelfApproval,
    /// The approver conflicts with the elevation request.
    ElevationApproverConflict,
    /// The action must go through the elevation lifecycle transition.
    ElevationTransitionRequired,
    /// The operation's role in the elevation lifecycle does not match its use.
    ElevationLifecycleRoleMismatch,
    /// The elevation request was rejected.
    ElevationRequestRejected,
    /// The elevation approval was rejected.
    ElevationApprovalRejected,
    /// Closing the elevation was rejected.
    ElevationCloseRejected,
    /// A mandatory review was rejected.
    MandatoryReviewRejected,
    /// The requested elevation exceeds the permitted duration.
    ElevationDurationExceeded,
    /// The delegation was rejected.
    DelegationRejected,
    /// The action must go through its delegation or revocation transition.
    DelegationTransitionRequired,
    /// The delegation chain is deeper than permitted.
    DelegationDepthExceeded,
    /// The delegation chain contains a cycle.
    DelegationCycle,
    /// The delegation lineage changed since it was observed.
    DelegationLineageChanged,
    /// Previously admitted authorization is no longer current.
    StaleAuthorization,
    /// The security basis on the product branch could not be admitted.
    ProductSecurityBasis(crate::basis::WorthQueryProductBranchAdmissionDenial),
    /// The runtime ran out of admission identities.
    AdmissionIdentityExhausted,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain the authorization basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// Graph work for the decision could not be admitted.
    GraphWorkAdmissionUnavailable,
    /// The request scope is outside what the grant covers.
    ScopeMismatch,
    /// The policy the decision needs is not installed.
    PolicyNotInstalled,
    /// The installed policy is invalid.
    InvalidInstalledPolicy,
    /// A Relational observation needed for the decision was rejected.
    RelationalObservationRejected,
    /// Selecting grants exceeded the installed bound.
    GrantSelectionLimitExceeded,
    /// The Runtime Bridge rejected the policy evaluation.
    BridgeEvaluationRejected,
    /// The authorization material disagreed with itself; the decision is refused.
    InconsistentDecision,
    /// The installed policy evaluated and did not allow the action.
    PermissionDenied,
}

/// Why an admitted operation's own request authority lapsed after admission.
///
/// These are the only causes current-authority revalidation can report, so a
/// later phase that must answer with its own refusal matches every cause
/// exactly rather than folding an open authorization kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryAdmissionLapse {
    /// The admitted request was cancelled.
    Cancelled,
    /// The admitted request reached its deadline.
    DeadlineExceeded,
    /// The admitted principal's authentication expired.
    AuthenticationExpired,
}

impl WorthQueryAdmissionLapse {
    pub(in crate::domain_computation) const fn denial_kind(
        self,
    ) -> WorthQueryOperationAuthorizationDenialKind {
        match self {
            Self::Cancelled => WorthQueryOperationAuthorizationDenialKind::Cancelled,
            Self::DeadlineExceeded => WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded,
            Self::AuthenticationExpired => {
                WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication
            }
        }
    }
}

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
    const fn from_denial_kind(kind: WorthQueryOperationAuthorizationDenialKind) -> Option<Self> {
        use WorthQueryOperationAuthorizationDenialKind as Denial;
        match kind {
            Denial::CapabilityGrantMissing
            | Denial::CapabilityAuthorizationMissing
            | Denial::CapabilityRequired => Some(Self::MissingCapability),
            Denial::ExplicitDenyRuleMatched => Some(Self::ExplicitPolicyDenial),
            Denial::ScopeMismatch => Some(Self::ScopeMismatch),
            Denial::PurposeMismatch => Some(Self::PurposeMismatch),
            Denial::ConflictRuleMatched => Some(Self::Conflict),
            Denial::SeparationOfDutyRuleMatched | Denial::DistinctActorRuleMatched => {
                Some(Self::SeparationOfDuty)
            }
            Denial::ElevationRequired => Some(Self::ElevationRequired),
            Denial::CapabilityExpired | Denial::ElevationExpired => Some(Self::ElevationExpired),
            Denial::ElevationProjectionRejected
            | Denial::ElevationInactive
            | Denial::ElevationSelfApproval
            | Denial::ElevationApproverConflict
            | Denial::ElevationTransitionRequired
            | Denial::ElevationLifecycleRoleMismatch
            | Denial::ElevationRequestRejected
            | Denial::ElevationApprovalRejected
            | Denial::ElevationCloseRejected
            | Denial::MandatoryReviewRejected
            | Denial::ElevationDurationExceeded => Some(Self::ElevationDenied),
            _ => None,
        }
    }
}

/// Process-local identity of one authorization denial, for correlation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryOperationAuthorizationDenialIdentity(WorthQueryApplicationOutcomeIdentity);

impl WorthQueryOperationAuthorizationDenialIdentity {
    fn mint() -> Option<Self> {
        WorthQueryApplicationOutcomeIdentity::mint().map(Self)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Refusal by authorization to let an operation or query proceed.
///
/// Decided during admission, before any effect. [`Self::kind`] is the first
/// cause and [`Self::causes`] lists every cause in order;
/// [`Self::explanation_cause`] gives a user-facing cause when one applies, and
/// [`Self::subject`] names what was being authorized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationAuthorizationDenial {
    identity: Option<WorthQueryOperationAuthorizationDenialIdentity>,
    kind: WorthQueryOperationAuthorizationDenialKind,
    causes: Vec<WorthQueryOperationAuthorizationDenialKind>,
    explanation_cause: Option<WorthQueryApplicationAuthorizationExplanationCause>,
    subject: String,
}

impl WorthQueryOperationAuthorizationDenial {
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryOperationAuthorizationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            identity: WorthQueryOperationAuthorizationDenialIdentity::mint(),
            kind,
            causes: vec![kind],
            explanation_cause: WorthQueryApplicationAuthorizationExplanationCause::from_denial_kind(
                kind,
            ),
            subject: subject.into(),
        }
    }

    pub(super) fn from_ordered_causes(
        causes: impl IntoIterator<Item = WorthQueryOperationAuthorizationDenialKind>,
        subject: impl Into<String>,
    ) -> Self {
        let mut causes = causes.into_iter().collect::<Vec<_>>();
        if causes.is_empty() {
            causes.push(WorthQueryOperationAuthorizationDenialKind::InconsistentDecision);
        }
        let kind = causes[0];
        Self {
            identity: WorthQueryOperationAuthorizationDenialIdentity::mint(),
            kind,
            causes,
            explanation_cause: WorthQueryApplicationAuthorizationExplanationCause::from_denial_kind(
                kind,
            ),
            subject: subject.into(),
        }
    }

    pub(in crate::domain_computation) fn inconsistent(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            subject,
        )
    }

    pub const fn kind(&self) -> WorthQueryOperationAuthorizationDenialKind {
        self.kind
    }

    pub const fn identity(&self) -> Option<WorthQueryOperationAuthorizationDenialIdentity> {
        self.identity
    }

    pub fn causes(&self) -> &[WorthQueryOperationAuthorizationDenialKind] {
        &self.causes
    }

    pub const fn explanation_cause(
        &self,
    ) -> Option<WorthQueryApplicationAuthorizationExplanationCause> {
        self.explanation_cause
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryOperationAuthorizationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "operation authorization denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOperationAuthorizationDenial {}

pub(super) fn product_security_basis_denial(
    denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(denial),
        subject,
    )
}
