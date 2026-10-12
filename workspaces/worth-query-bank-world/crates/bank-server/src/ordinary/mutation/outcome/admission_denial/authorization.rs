use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind as QueryKind,
    WorthQueryProductBranchAdmissionDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankAuthorizationDenialKind {
    ExecutionRequest(worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind),
    ForeignPhase,
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

/// Every Query authorization kind keeps its own Bank kind, so a request-level
/// refusal carries the same cause as a Bank-level one.
impl From<QueryKind> for BankAuthorizationDenialKind {
    fn from(kind: QueryKind) -> Self {
        map_kind(kind)
    }
}

const fn map_kind(kind: QueryKind) -> BankAuthorizationDenialKind {
    use BankAuthorizationDenialKind as Bank;
    match kind {
        QueryKind::Stopped(Ok(cause)) => Bank::ExecutionRequest(cause),
        QueryKind::Refused(AuthorizationRefusal::ForeignPhase) => Bank::ForeignPhase,
        QueryKind::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled)) => Bank::Cancelled,
        QueryKind::Stopped(Err(worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut)) => Bank::DeadlineExceeded,
        QueryKind::Refused(AuthorizationRefusal::ExpiredAuthentication) => Bank::ExpiredAuthentication,
        QueryKind::Refused(AuthorizationRefusal::ForeignRuntime) => Bank::ForeignRuntime,
        QueryKind::Refused(AuthorizationRefusal::StaleInstalledSchema) => Bank::StaleInstalledSchema,
        QueryKind::Refused(AuthorizationRefusal::StaleInstalledOperation) => Bank::StaleInstalledOperation,
        QueryKind::Refused(AuthorizationRefusal::StalePrincipal) => Bank::StalePrincipal,
        QueryKind::Refused(AuthorizationRefusal::StaleScope) => Bank::StaleScope,
        QueryKind::Refused(AuthorizationRefusal::MutationPreconditionRejected) => Bank::MutationPreconditionRejected,
        QueryKind::Refused(AuthorizationRefusal::CanonicalWorkDenied) => Bank::CanonicalWorkDenied,
        QueryKind::Refused(AuthorizationRefusal::TrustedTimeUnavailable) => Bank::TrustedTimeUnavailable,
        QueryKind::Refused(AuthorizationRefusal::InvalidOperationInput) => Bank::InvalidOperationInput,
        QueryKind::Refused(AuthorizationRefusal::CapabilityProjectionRejected) => Bank::CapabilityProjectionRejected,
        QueryKind::Refused(AuthorizationRefusal::CapabilityGrantMissing) => Bank::CapabilityGrantMissing,
        QueryKind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing) => Bank::CapabilityAuthorizationMissing,
        QueryKind::Refused(AuthorizationRefusal::PurposeMismatch) => Bank::PurposeMismatch,
        QueryKind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched) => Bank::ExplicitDenyRuleMatched,
        QueryKind::Refused(AuthorizationRefusal::ConflictRuleMatched) => Bank::ConflictRuleMatched,
        QueryKind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched) => Bank::SeparationOfDutyRuleMatched,
        QueryKind::Refused(AuthorizationRefusal::DistinctActorRuleMatched) => Bank::DistinctActorRuleMatched,
        QueryKind::Refused(AuthorizationRefusal::CapabilityRequired) => Bank::CapabilityRequired,
        QueryKind::Refused(AuthorizationRefusal::CapabilityNotRequired) => Bank::CapabilityNotRequired,
        QueryKind::Refused(AuthorizationRefusal::CapabilityExpired) => Bank::CapabilityExpired,
        QueryKind::Refused(AuthorizationRefusal::ElevationRequired) => Bank::ElevationRequired,
        QueryKind::Refused(AuthorizationRefusal::ElevationNotApplicable) => Bank::ElevationNotApplicable,
        QueryKind::Refused(AuthorizationRefusal::ElevationProjectionRejected) => Bank::ElevationProjectionRejected,
        QueryKind::Refused(AuthorizationRefusal::ElevationExpired) => Bank::ElevationExpired,
        QueryKind::Refused(AuthorizationRefusal::ElevationInactive) => Bank::ElevationInactive,
        QueryKind::Refused(AuthorizationRefusal::ElevationSelfApproval) => Bank::ElevationSelfApproval,
        QueryKind::Refused(AuthorizationRefusal::ElevationApproverConflict) => Bank::ElevationApproverConflict,
        QueryKind::Refused(AuthorizationRefusal::ElevationTransitionRequired) => Bank::ElevationTransitionRequired,
        QueryKind::Refused(AuthorizationRefusal::ElevationLifecycleRoleMismatch) => Bank::ElevationLifecycleRoleMismatch,
        QueryKind::Refused(AuthorizationRefusal::ElevationRequestRejected) => Bank::ElevationRequestRejected,
        QueryKind::Refused(AuthorizationRefusal::ElevationApprovalRejected) => Bank::ElevationApprovalRejected,
        QueryKind::Refused(AuthorizationRefusal::ElevationCloseRejected) => Bank::ElevationCloseRejected,
        QueryKind::Refused(AuthorizationRefusal::MandatoryReviewRejected) => Bank::MandatoryReviewRejected,
        QueryKind::Refused(AuthorizationRefusal::ElevationDurationExceeded) => Bank::ElevationDurationExceeded,
        QueryKind::Refused(AuthorizationRefusal::DelegationRejected) => Bank::DelegationRejected,
        QueryKind::Refused(AuthorizationRefusal::DelegationTransitionRequired) => Bank::DelegationTransitionRequired,
        QueryKind::Refused(AuthorizationRefusal::DelegationDepthExceeded) => Bank::DelegationDepthExceeded,
        QueryKind::Refused(AuthorizationRefusal::DelegationCycle) => Bank::DelegationCycle,
        QueryKind::Refused(AuthorizationRefusal::DelegationLineageChanged) => Bank::DelegationLineageChanged,
        QueryKind::Refused(AuthorizationRefusal::StaleAuthorization) => Bank::StaleAuthorization,
        QueryKind::Refused(AuthorizationRefusal::ProductSecurityBasis(refusal)) => Bank::ProductSecurityBasis(worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::Refused(refusal)),
        QueryKind::Refused(AuthorizationRefusal::AdmissionIdentityExhausted) => Bank::AdmissionIdentityExhausted,
        QueryKind::Refused(AuthorizationRefusal::ActiveSnapshotCapacityExhausted{
            maximum_active_snapshots,
        }) => Bank::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        QueryKind::Refused(AuthorizationRefusal::SnapshotIdentityExhausted) => Bank::SnapshotIdentityExhausted,
        QueryKind::Refused(AuthorizationRefusal::RetentionCapacityExhausted) => Bank::RetentionCapacityExhausted,
        QueryKind::Refused(AuthorizationRefusal::RetentionIdentityExhausted) => Bank::RetentionIdentityExhausted,
        QueryKind::Refused(AuthorizationRefusal::GraphWorkAdmissionUnavailable) => Bank::GraphWorkAdmissionUnavailable,
        QueryKind::Refused(AuthorizationRefusal::ScopeMismatch) => Bank::ScopeMismatch,
        QueryKind::Refused(AuthorizationRefusal::PolicyNotInstalled) => Bank::PolicyNotInstalled,
        QueryKind::Refused(AuthorizationRefusal::InvalidInstalledPolicy) => Bank::InvalidInstalledPolicy,
        QueryKind::Refused(AuthorizationRefusal::RelationalObservationRejected) => Bank::RelationalObservationRejected,
        QueryKind::Refused(AuthorizationRefusal::GrantSelectionLimitExceeded) => Bank::GrantSelectionLimitExceeded,
        QueryKind::Refused(AuthorizationRefusal::BridgeEvaluationRejected) => Bank::BridgeEvaluationRejected,
        QueryKind::Refused(AuthorizationRefusal::InconsistentDecision) => Bank::InconsistentDecision,
        QueryKind::Refused(AuthorizationRefusal::PermissionDenied) => Bank::PermissionDenied,
    }
}

const fn authorization_code(kind: BankAuthorizationDenialKind) -> &'static str {
    use BankAuthorizationDenialKind as Bank;
    match kind {
        Bank::ExecutionRequest(_) => "execution-request",
        Bank::ForeignPhase => "foreign-phase",
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
