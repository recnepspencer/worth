use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind as QueryKind,
    WorthQueryProductBranchAdmissionDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankAuthorizationDenialKind {
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
    ProductSecurityBasis(WorthQueryProductBranchAdmissionDenial),
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
pub struct BankAuthorizationDenial {
    kind: BankAuthorizationDenialKind,
    contributing_cause_count: usize,
}

impl BankAuthorizationDenial {
    pub const fn kind(self) -> BankAuthorizationDenialKind {
        self.kind
    }

    pub const fn code(self) -> &'static str {
        authorization_code(self.kind)
    }

    pub const fn contributing_cause_count(self) -> usize {
        self.contributing_cause_count
    }

    pub(crate) fn from_query(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self::from_kind(denial.kind(), denial.causes().len())
    }

    pub(crate) const fn from_kind(kind: QueryKind, contributing_cause_count: usize) -> Self {
        Self {
            kind: map_kind(kind),
            contributing_cause_count,
        }
    }
}

impl std::fmt::Display for BankAuthorizationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

const fn map_kind(kind: QueryKind) -> BankAuthorizationDenialKind {
    use BankAuthorizationDenialKind as Bank;
    match kind {
        QueryKind::Cancelled => Bank::Cancelled,
        QueryKind::DeadlineExceeded => Bank::DeadlineExceeded,
        QueryKind::ExpiredAuthentication => Bank::ExpiredAuthentication,
        QueryKind::ForeignRuntime => Bank::ForeignRuntime,
        QueryKind::StaleInstalledSchema => Bank::StaleInstalledSchema,
        QueryKind::StaleInstalledOperation => Bank::StaleInstalledOperation,
        QueryKind::StalePrincipal => Bank::StalePrincipal,
        QueryKind::StaleScope => Bank::StaleScope,
        QueryKind::MutationPreconditionRejected => Bank::MutationPreconditionRejected,
        QueryKind::CanonicalWorkDenied => Bank::CanonicalWorkDenied,
        QueryKind::TrustedTimeUnavailable => Bank::TrustedTimeUnavailable,
        QueryKind::InvalidOperationInput => Bank::InvalidOperationInput,
        QueryKind::CapabilityProjectionRejected => Bank::CapabilityProjectionRejected,
        QueryKind::CapabilityGrantMissing => Bank::CapabilityGrantMissing,
        QueryKind::CapabilityAuthorizationMissing => Bank::CapabilityAuthorizationMissing,
        QueryKind::PurposeMismatch => Bank::PurposeMismatch,
        QueryKind::ExplicitDenyRuleMatched => Bank::ExplicitDenyRuleMatched,
        QueryKind::ConflictRuleMatched => Bank::ConflictRuleMatched,
        QueryKind::SeparationOfDutyRuleMatched => Bank::SeparationOfDutyRuleMatched,
        QueryKind::DistinctActorRuleMatched => Bank::DistinctActorRuleMatched,
        QueryKind::CapabilityRequired => Bank::CapabilityRequired,
        QueryKind::CapabilityNotRequired => Bank::CapabilityNotRequired,
        QueryKind::CapabilityExpired => Bank::CapabilityExpired,
        QueryKind::ElevationRequired => Bank::ElevationRequired,
        QueryKind::ElevationNotApplicable => Bank::ElevationNotApplicable,
        QueryKind::ElevationProjectionRejected => Bank::ElevationProjectionRejected,
        QueryKind::ElevationExpired => Bank::ElevationExpired,
        QueryKind::ElevationInactive => Bank::ElevationInactive,
        QueryKind::ElevationSelfApproval => Bank::ElevationSelfApproval,
        QueryKind::ElevationApproverConflict => Bank::ElevationApproverConflict,
        QueryKind::ElevationTransitionRequired => Bank::ElevationTransitionRequired,
        QueryKind::ElevationLifecycleRoleMismatch => Bank::ElevationLifecycleRoleMismatch,
        QueryKind::ElevationRequestRejected => Bank::ElevationRequestRejected,
        QueryKind::ElevationApprovalRejected => Bank::ElevationApprovalRejected,
        QueryKind::ElevationCloseRejected => Bank::ElevationCloseRejected,
        QueryKind::MandatoryReviewRejected => Bank::MandatoryReviewRejected,
        QueryKind::ElevationDurationExceeded => Bank::ElevationDurationExceeded,
        QueryKind::DelegationRejected => Bank::DelegationRejected,
        QueryKind::DelegationTransitionRequired => Bank::DelegationTransitionRequired,
        QueryKind::DelegationDepthExceeded => Bank::DelegationDepthExceeded,
        QueryKind::DelegationCycle => Bank::DelegationCycle,
        QueryKind::DelegationLineageChanged => Bank::DelegationLineageChanged,
        QueryKind::StaleAuthorization => Bank::StaleAuthorization,
        QueryKind::ProductSecurityBasis(denial) => Bank::ProductSecurityBasis(denial),
        QueryKind::AdmissionIdentityExhausted => Bank::AdmissionIdentityExhausted,
        QueryKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Bank::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        QueryKind::SnapshotIdentityExhausted => Bank::SnapshotIdentityExhausted,
        QueryKind::RetentionCapacityExhausted => Bank::RetentionCapacityExhausted,
        QueryKind::RetentionIdentityExhausted => Bank::RetentionIdentityExhausted,
        QueryKind::GraphWorkAdmissionUnavailable => Bank::GraphWorkAdmissionUnavailable,
        QueryKind::ScopeMismatch => Bank::ScopeMismatch,
        QueryKind::PolicyNotInstalled => Bank::PolicyNotInstalled,
        QueryKind::InvalidInstalledPolicy => Bank::InvalidInstalledPolicy,
        QueryKind::RelationalObservationRejected => Bank::RelationalObservationRejected,
        QueryKind::GrantSelectionLimitExceeded => Bank::GrantSelectionLimitExceeded,
        QueryKind::BridgeEvaluationRejected => Bank::BridgeEvaluationRejected,
        QueryKind::InconsistentDecision => Bank::InconsistentDecision,
        QueryKind::PermissionDenied => Bank::PermissionDenied,
    }
}

const fn authorization_code(kind: BankAuthorizationDenialKind) -> &'static str {
    use BankAuthorizationDenialKind as Bank;
    match kind {
        Bank::Cancelled => "cancelled",
        Bank::DeadlineExceeded => "deadline-exceeded",
        Bank::ExpiredAuthentication => "expired-authentication",
        Bank::ForeignRuntime => "foreign-runtime",
        Bank::StaleInstalledSchema => "stale-installed-schema",
        Bank::StaleInstalledOperation => "stale-installed-operation",
        Bank::StalePrincipal => "stale-principal",
        Bank::StaleScope => "stale-scope",
        Bank::MutationPreconditionRejected => "mutation-precondition-rejected",
        Bank::CanonicalWorkDenied => "canonical-work-denied",
        Bank::TrustedTimeUnavailable => "trusted-time-unavailable",
        Bank::InvalidOperationInput => "invalid-operation-input",
        Bank::CapabilityProjectionRejected => "capability-projection-rejected",
        Bank::CapabilityGrantMissing => "capability-grant-missing",
        Bank::CapabilityAuthorizationMissing => "capability-authorization-missing",
        Bank::PurposeMismatch => "purpose-mismatch",
        Bank::ExplicitDenyRuleMatched => "explicit-deny-rule-matched",
        Bank::ConflictRuleMatched => "conflict-rule-matched",
        Bank::SeparationOfDutyRuleMatched => "separation-of-duty-rule-matched",
        Bank::DistinctActorRuleMatched => "distinct-actor-rule-matched",
        Bank::CapabilityRequired => "capability-required",
        Bank::CapabilityNotRequired => "capability-not-required",
        Bank::CapabilityExpired => "capability-expired",
        Bank::ElevationRequired => "elevation-required",
        Bank::ElevationNotApplicable => "elevation-not-applicable",
        Bank::ElevationProjectionRejected => "elevation-projection-rejected",
        Bank::ElevationExpired => "elevation-expired",
        Bank::ElevationInactive => "elevation-inactive",
        Bank::ElevationSelfApproval => "elevation-self-approval",
        Bank::ElevationApproverConflict => "elevation-approver-conflict",
        Bank::ElevationTransitionRequired => "elevation-transition-required",
        Bank::ElevationLifecycleRoleMismatch => "elevation-lifecycle-role-mismatch",
        Bank::ElevationRequestRejected => "elevation-request-rejected",
        Bank::ElevationApprovalRejected => "elevation-approval-rejected",
        Bank::ElevationCloseRejected => "elevation-close-rejected",
        Bank::MandatoryReviewRejected => "mandatory-review-rejected",
        Bank::ElevationDurationExceeded => "elevation-duration-exceeded",
        Bank::DelegationRejected => "delegation-rejected",
        Bank::DelegationTransitionRequired => "delegation-transition-required",
        Bank::DelegationDepthExceeded => "delegation-depth-exceeded",
        Bank::DelegationCycle => "delegation-cycle",
        Bank::DelegationLineageChanged => "delegation-lineage-changed",
        Bank::StaleAuthorization => "stale-authorization",
        Bank::ProductSecurityBasis(_) => "product-security-basis",
        Bank::AdmissionIdentityExhausted => "admission-identity-exhausted",
        Bank::ActiveSnapshotCapacityExhausted { .. } => "active-snapshot-capacity-exhausted",
        Bank::SnapshotIdentityExhausted => "snapshot-identity-exhausted",
        Bank::RetentionCapacityExhausted => "retention-capacity-exhausted",
        Bank::RetentionIdentityExhausted => "retention-identity-exhausted",
        Bank::GraphWorkAdmissionUnavailable => "graph-work-admission-unavailable",
        Bank::ScopeMismatch => "scope-mismatch",
        Bank::PolicyNotInstalled => "policy-not-installed",
        Bank::InvalidInstalledPolicy => "invalid-installed-policy",
        Bank::RelationalObservationRejected => "relational-observation-rejected",
        Bank::GrantSelectionLimitExceeded => "grant-selection-limit-exceeded",
        Bank::BridgeEvaluationRejected => "bridge-evaluation-rejected",
        Bank::InconsistentDecision => "inconsistent-decision",
        Bank::PermissionDenied => "permission-denied",
    }
}
