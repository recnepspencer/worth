//! Typed authorization denial topology.

use crate::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationAuthorizationDenialKind {
    Cancelled,
    DeadlineExceeded,
    ExpiredAuthentication,
    ForeignRuntime,
    StaleInstalledSchema,
    StaleInstalledOperation,
    StalePrincipal,
    StaleScope,
    MutationPreconditionRejected,
    CanonicalWorkDenied,
    TrustedTimeUnavailable,
    InvalidOperationInput,
    CapabilityProjectionRejected,
    CapabilityGrantMissing,
    CapabilityAuthorizationMissing,
    PurposeMismatch,
    ExplicitDenyRuleMatched,
    ConflictRuleMatched,
    SeparationOfDutyRuleMatched,
    DistinctActorRuleMatched,
    CapabilityRequired,
    CapabilityNotRequired,
    CapabilityExpired,
    ElevationRequired,
    ElevationNotApplicable,
    ElevationProjectionRejected,
    ElevationExpired,
    ElevationInactive,
    ElevationSelfApproval,
    ElevationApproverConflict,
    ElevationTransitionRequired,
    ElevationLifecycleRoleMismatch,
    ElevationRequestRejected,
    ElevationApprovalRejected,
    ElevationCloseRejected,
    MandatoryReviewRejected,
    ElevationDurationExceeded,
    DelegationRejected,
    DelegationTransitionRequired,
    DelegationDepthExceeded,
    DelegationCycle,
    DelegationLineageChanged,
    StaleAuthorization,
    ProductSecurityBasis(crate::basis::WorthQueryProductBranchAdmissionDenial),
    AdmissionIdentityExhausted,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    GraphWorkAdmissionUnavailable,
    ScopeMismatch,
    PolicyNotInstalled,
    InvalidInstalledPolicy,
    RelationalObservationRejected,
    GrantSelectionLimitExceeded,
    BridgeEvaluationRejected,
    InconsistentDecision,
    PermissionDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationAuthorizationExplanationCause {
    MissingCapability,
    ExplicitPolicyDenial,
    ScopeMismatch,
    PurposeMismatch,
    Conflict,
    SeparationOfDuty,
    ElevationRequired,
    ElevationDenied,
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
