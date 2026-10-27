//! Wire reasons are explicit native classifications, never Debug strings or authority.
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct WorthServerSourceDiagnostic {
    pub category: WorthServerSourceDiagnosticCategory,
    pub code: String,
    pub members: Vec<String>,
}
#[derive(serde::Serialize, schemars::JsonSchema)]
pub enum WorthServerSourceDiagnosticCategory {
    Requirement,
    Permission,
    Source,
    Input,
    Execution,
    Mutation,
    Recovery,
    Cleanup,
    History,
}
impl WorthServerSourceDiagnostic {
    pub fn new(category: WorthServerSourceDiagnosticCategory, code: impl Into<String>) -> Self {
        Self {
            category,
            code: code.into(),
            members: Vec::new(),
        }
    }
    pub fn body(cause: &WorthQueryOperationBodyDenial) -> Self {
        use WorthQueryOperationBodyDenial::*;
        let (category, code, members) = match cause {
            SourceMismatch {
                entity,
                aspect,
                field,
            } => (
                WorthServerSourceDiagnosticCategory::Source,
                "source_mismatch",
                vec![entity.clone(), aspect.clone(), field.clone()],
            ),
            InputMismatch { member } => (
                WorthServerSourceDiagnosticCategory::Input,
                "input_mismatch",
                vec![member.clone()],
            ),
            RequirementUnmet {
                entity,
                aspect,
                field,
            } => (
                WorthServerSourceDiagnosticCategory::Requirement,
                "requirement_unmet",
                vec![entity.clone(), aspect.clone(), field.clone()],
            ),
            DeclaredRequirementUnmet { identity } => (
                WorthServerSourceDiagnosticCategory::Requirement,
                "declared_requirement_unmet",
                vec![identity.clone()],
            ),
        };
        Self {
            category,
            code: code.into(),
            members,
        }
    }
}

pub(super) fn mutation(
    cause: &WorthQueryApplicationRequestMutationDenial,
) -> WorthServerSourceDiagnostic {
    use WorthQueryApplicationRequestMutationDenial::*;
    let code = match cause {
        Authorization(cause) => return authorization(cause),
        ExecutionRequest(_) => "execution_request",
        InputCapture(_) => "input_capture",
        BindingInstallation(_) => "binding_installation",
        CapabilityInstallation(_) => "capability_installation",
        ProductSelection(_) => "product_selection",
        PrincipalResolution(_) => "principal_resolution",
        ScopeResolution(_) => "scope_resolution",
        Idempotency(_) => "idempotency",
        Identity(_) => "identity",
        InputCapture(_) => "input_capture",
        Handler(_) => "handler",
        SourceExpectation(cause) => return source(cause),
        ProgramSelection(_) => "program_selection",
        ApplicationProgramRequired => "application_program_required",
        ApplicationProgramMismatch => "application_program_mismatch",
        RequiresWorkflowTransition => "requires_workflow_transition",
        WorkflowAuthoritySpent => "workflow_authority_spent",
        WorkflowCapturedInput(_) => "workflow_captured_input",
        PreparationSpent => "preparation_spent",
        WorkflowTransitionCurrentness(_) => "workflow_transition_currentness",
        SignedCreation(_) => "signed_creation",
    };
    WorthServerSourceDiagnostic::new(WorthServerSourceDiagnosticCategory::Mutation, code)
}
fn source(cause: &WorthQuerySourceExpectationDenial) -> WorthServerSourceDiagnostic {
    use WorthQuerySourceExpectationDenialKind::*;
    let code = match cause.kind() {
        MissingExpectation => "missing_expectation",
        ForeignApplication => "foreign_application",
        ForeignInstallation => "foreign_installation",
        ForeignSchema => "foreign_schema",
        ForeignModel => "foreign_model",
        ForeignBranch => "foreign_branch",
        ForeignOperation => "foreign_operation",
        SourceChanged => "source_changed",
        IncompleteFootprint => "incomplete_footprint",
        SourceContractMismatch => "source_contract_mismatch",
        SourceParametersMismatch => "source_parameters_mismatch",
        WorkBudgetExceeded => "source_work_denied",
        PreparationMemoryExceeded => "source_memory_denied",
        RetainedSourceStorageDenied => "source_storage_denied",
    };
    WorthServerSourceDiagnostic::new(WorthServerSourceDiagnosticCategory::Source, code)
}
fn authorization(cause: &WorthQueryOperationAuthorizationDenial) -> WorthServerSourceDiagnostic {
    let code = match cause.kind() {
        WorthQueryOperationAuthorizationDenialKind::Stopped(_) => "request_stopped",
        WorthQueryOperationAuthorizationDenialKind::Refused(reason) => authorization_code(reason),
    };
    WorthServerSourceDiagnostic::new(WorthServerSourceDiagnosticCategory::Permission, code)
}
pub(super) fn refusal(cause: &WorthQuerySourceOnlyDenial) -> WorthServerSourceDiagnostic {
    use WorthQuerySourceOnlyDenial::*;
    let (category, code) = match cause {
        Mutation(cause) => return mutation(cause),
        Execution(_) => (WorthServerSourceDiagnosticCategory::Execution, "execution"),
        Recovery(_) => (WorthServerSourceDiagnosticCategory::Recovery, "recovery"),
        Cleanup(cause) => return cleanup(cause),
        History(_) => (WorthServerSourceDiagnosticCategory::History, "history"),
    };
    WorthServerSourceDiagnostic::new(category, code)
}
pub(super) fn cleanup(cause: &WorthQueryOperationCloseDenial) -> WorthServerSourceDiagnostic {
    use WorthQueryOperationCloseDenial::*;
    let code = match cause {
        FreshRequestMismatch => "fresh_request_mismatch",
        ManagedCleanup(_) => "managed_cleanup",
        CleanupCapacity => "cleanup_capacity",
        NativeObligationRetained => "native_obligation_retained",
        Outputs(_) => "output_cleanup",
        Execution(_) => "execution",
        Recovery(_) => "recovery",
        Owner(_) => "owner_cleanup",
    };
    WorthServerSourceDiagnostic::new(WorthServerSourceDiagnosticCategory::Cleanup, code)
}

fn authorization_code(reason: WorthQueryOperationAuthorizationRefusalKind) -> &'static str {
    use WorthQueryOperationAuthorizationRefusalKind::*;
    match reason {
        ForeignPhase => "foreign_phase",
        ExpiredAuthentication => "expired_authentication",
        ForeignRuntime => "foreign_runtime",
        StaleInstalledSchema => "stale_installed_schema",
        StaleInstalledOperation => "stale_installed_operation",
        StalePrincipal => "stale_principal",
        StaleScope => "stale_scope",
        MutationPreconditionRejected => "mutation_precondition_rejected",
        CanonicalWorkDenied => "canonical_work_denied",
        TrustedTimeUnavailable => "trusted_time_unavailable",
        InvalidOperationInput => "invalid_operation_input",
        CapabilityProjectionRejected => "capability_projection_rejected",
        CapabilityGrantMissing => "capability_grant_missing",
        CapabilityAuthorizationMissing => "capability_authorization_missing",
        WorkflowActorContextRequired => "workflow_actor_context_required",
        WorkflowActorAbsent => "workflow_actor_absent",
        WorkflowActorComparisonMismatch => "workflow_actor_comparison_mismatch",
        WorkflowActorSourceInvalid => "workflow_actor_source_invalid",
        WorkflowActorEvidenceUnavailable => "workflow_actor_evidence_unavailable",
        PurposeMismatch => "purpose_mismatch",
        ExplicitDenyRuleMatched => "explicit_deny_rule_matched",
        ConflictRuleMatched => "conflict_rule_matched",
        SeparationOfDutyRuleMatched => "separation_of_duty_rule_matched",
        DistinctActorRuleMatched => "distinct_actor_rule_matched",
        CapabilityRequired => "capability_required",
        CapabilityNotRequired => "capability_not_required",
        CapabilityExpired => "capability_expired",
        ElevationRequired => "elevation_required",
        ElevationNotApplicable => "elevation_not_applicable",
        ElevationProjectionRejected => "elevation_projection_rejected",
        ElevationExpired => "elevation_expired",
        ElevationInactive => "elevation_inactive",
        ElevationSelfApproval => "elevation_self_approval",
        ElevationApproverConflict => "elevation_approver_conflict",
        ElevationTransitionRequired => "elevation_transition_required",
        ElevationLifecycleRoleMismatch => "elevation_lifecycle_role_mismatch",
        ElevationRequestRejected => "elevation_request_rejected",
        ElevationApprovalRejected => "elevation_approval_rejected",
        ElevationCloseRejected => "elevation_close_rejected",
        MandatoryReviewRejected => "mandatory_review_rejected",
        ElevationDurationExceeded => "elevation_duration_exceeded",
        DelegationRejected => "delegation_rejected",
        DelegationTransitionRequired => "delegation_transition_required",
        DelegationDepthExceeded => "delegation_depth_exceeded",
        DelegationCycle => "delegation_cycle",
        DelegationLineageChanged => "delegation_lineage_changed",
        StaleAuthorization => "stale_authorization",
        ProductSecurityBasis(_) => "product_security_basis",
        AdmissionIdentityExhausted => "admission_identity_exhausted",
        ActiveSnapshotCapacityExhausted { .. } => "active_snapshot_capacity_exhausted",
        SnapshotIdentityExhausted => "snapshot_identity_exhausted",
        RetentionCapacityExhausted => "retention_capacity_exhausted",
        RetentionIdentityExhausted => "retention_identity_exhausted",
        GraphWorkAdmissionUnavailable => "graph_work_admission_unavailable",
        ScopeMismatch => "scope_mismatch",
        PolicyNotInstalled => "policy_not_installed",
        InvalidInstalledPolicy => "invalid_installed_policy",
        RelationalObservationRejected => "relational_observation_rejected",
        GrantSelectionLimitExceeded => "grant_selection_limit_exceeded",
        BridgeEvaluationRejected => "bridge_evaluation_rejected",
        InconsistentDecision => "inconsistent_decision",
        PermissionDenied => "permission_denied",
    }
}
