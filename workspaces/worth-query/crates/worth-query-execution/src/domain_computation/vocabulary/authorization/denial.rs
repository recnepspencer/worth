//! Moved whole from `domain_computation::authorization::denial`: the items below and every impl they own.

/// The specific reason authorization refused an operation or query.
///
/// Families: cancellation and deadline; stale installed meaning, principal,
/// scope, or authorization; capability, policy, and rule outcomes; elevation
/// and delegation lifecycle; basis, capacity, and identity exhaustion; and
/// internal consistency failures. Every kind is decided at admission, before
/// any effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationAuthorizationDenialKind {
    Stopped(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    Refused(WorthQueryOperationAuthorizationRefusalKind),
}

/// An authorization refusal decided from principal, scope and installed policy facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationAuthorizationRefusalKind {
    /// The supplied phase belongs to another installed runtime.
    ForeignPhase,
    /// The principal's authentication has expired.
    ExpiredAuthentication,
    /// The authorization material belongs to a different runtime.
    ForeignRuntime,
    /// The installed schema binding changed since the material was admitted.
    StaleInstalledSchema,
    /// The installed operation changed or is no longer installed.
    StaleInstalledOperation,
    /// The admission brand carries a different input type than installation.
    InputNotAdmitted,
    /// The principal no longer resolves as it did.
    StalePrincipal,
    /// The request scope no longer resolves as it did.
    StaleScope,
    /// A declared mutation precondition did not hold.
    MutationPreconditionRejected,
    /// The bounded canonical work for the decision was refused.
    CanonicalWorkDenied,
    /// A structural failure in the caller's execution retains its native cause.
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
    WorkflowActorContextRequired,
    WorkflowActorAbsent,
    WorkflowActorComparisonMismatch,
    WorkflowActorSourceInvalid,
    WorkflowActorEvidenceUnavailable,
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
    ProductSecurityBasis(crate::basis::WorthQueryProductBranchAdmissionRefusalKind),
    /// The runtime ran out of admission identities.
    AdmissionIdentityExhausted,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
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

impl From<crate::basis::WorthQueryProductBranchAdmissionDenial>
    for WorthQueryOperationAuthorizationDenialKind
{
    fn from(denial: crate::basis::WorthQueryProductBranchAdmissionDenial) -> Self {
        match denial {
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop) => {
                Self::Stopped(stop)
            }
            crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(refusal) => {
                Self::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::ProductSecurityBasis(refusal),
                )
            }
        }
    }
}
