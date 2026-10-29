use worth_query_declaration::facade::authoring::{
    AspectFieldKey, AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath,
    RawAuthoredQuery, RawAuthoredResultShape, RootEntityKey,
};
use worth_query_declaration::facade::canonicalization::CanonicalQueryBundle;

use super::super::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

use super::{
    admit_policy_tenant_context, runtime_backed_policy_tenant_admission_support_profile,
    BranchAccessGrant, PolicyAdmissionDisposition, PolicyAspectMask, PolicyCostPosture,
    PolicyEpoch, PolicyExecutionModeRequest, PolicyRuleSnapshot, PolicyTenantAdmissionFailureClass,
    PolicyTenantPhaseOneSurface, PolicyTenantSupportStatus, PolicyWorkBudget,
};

fn canonical_query() -> CanonicalQueryBundle {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .project(AspectFieldSelector::new("profile", "display_name").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .field(AuthoredResultShapeField::new("profile", "display_name", "display_name").unwrap())
        .build()
        .unwrap();

    GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap()
}

fn admitted_inputs() -> (
    PolicyRuleSnapshot,
    TenantBindingSnapshot,
    BranchAccessGrant,
    SchemaVariantSnapshot,
) {
    let policy = PolicyRuleSnapshot::synthetic_authority(
        "runtime-policy",
        "rules-v1",
        PolicyEpoch::Synthetic(7),
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "branch-a",
        "schema-a",
        TenantBasisEpoch::Synthetic(3),
    );
    let branch = BranchAccessGrant::synthetic_granted("branch-a", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "compatible");
    (policy, tenant, branch, schema)
}

#[test]
fn direct_policy_tenant_admission_freezes_all_required_bases() {
    let canonical = canonical_query();
    let (policy, tenant, branch, schema) = admitted_inputs();

    let admitted = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::CurrentRead,
    )
    .expect("direct tenant policy context should admit");

    assert_eq!(
        admitted.bundle().canonical_query_digest(),
        canonical.query().digest().as_str()
    );
    assert_eq!(
        admitted.policy_basis().disposition(),
        PolicyAdmissionDisposition::AdmittedUnchanged
    );
    assert_eq!(
        admitted.policy_basis().cost_posture(),
        PolicyCostPosture::ConstantProof
    );
    assert_eq!(
        admitted.bundle().policy_work_budget(),
        PolicyWorkBudget::bounded(1, 1, 1)
    );
    assert_eq!(admitted.tenant_truth_basis().tenant_identity(), "tenant-a");
    assert_eq!(admitted.tenant_truth_basis().branch_identity(), "branch-a");
    assert_eq!(admitted.tenant_schema_basis().schema_identity(), "schema-a");
    assert_eq!(admitted.bundle().counters().admission_bundle_count(), 1);
    assert_eq!(
        admitted
            .bundle()
            .counters()
            .tenant()
            .direct_tenant_binding_admitted_count(),
        1
    );
    assert!(!admitted.bundle().digest().as_str().is_empty());
}

#[test]
fn unknown_policy_cost_is_denied_before_tenant_truth_admission() {
    let canonical = canonical_query();
    let policy = PolicyRuleSnapshot::synthetic_authority_with_budget(
        "runtime-policy",
        "rules-v1",
        PolicyEpoch::Synthetic(7),
        true,
        PolicyCostPosture::UnknownCost,
        Some(PolicyWorkBudget::bounded(1, 1, 1)),
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "branch-a",
        "schema-a",
        TenantBasisEpoch::Synthetic(3),
    );
    let branch = BranchAccessGrant::synthetic_granted("branch-a", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "compatible");

    let error = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::CurrentRead,
    )
    .expect_err("unknown policy cost must deny during admission");

    assert_eq!(
        error.failure_class(),
        PolicyTenantAdmissionFailureClass::PolicyWorkBudgetDenied
    );
    assert_eq!(
        error.counters().policy().policy_work_budget_denial_count(),
        1
    );
    assert_eq!(
        error
            .counters()
            .tenant()
            .direct_tenant_binding_admitted_count(),
        0
    );
}

#[test]
fn policy_projection_narrowing_is_explicit_in_admitted_basis() {
    let canonical = canonical_query();
    let policy = PolicyRuleSnapshot::synthetic_authority_with_projection(
        "runtime-policy",
        "rules-v1",
        PolicyEpoch::Synthetic(7),
        PolicyAspectMask::allow_all()
            .with_masked(AspectFieldKey::from_authoring_parts("profile", "display_name").unwrap()),
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "branch-a",
        "schema-a",
        TenantBasisEpoch::Synthetic(3),
    );
    let branch = BranchAccessGrant::synthetic_granted("branch-a", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "compatible");

    let admitted = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::BranchRead,
    )
    .expect("narrowing policy should still admit with a typed disposition");

    assert_eq!(
        admitted.policy_basis().disposition(),
        PolicyAdmissionDisposition::AdmittedNarrowed
    );
}

#[test]
fn unsupported_execution_seams_are_deferred_before_policy_and_truth_touch() {
    let canonical = canonical_query();
    let (policy, tenant, branch, schema) = admitted_inputs();

    let error = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::LiveSubscription,
    )
    .expect_err("live subscription must remain deferred in Phase 1");

    assert_eq!(
        error.failure_class(),
        PolicyTenantAdmissionFailureClass::UnsupportedExecutionMode
    );
    assert_eq!(
        error
            .counters()
            .policy()
            .unsupported_execution_mode_denial_count(),
        1
    );
    assert_eq!(
        error
            .counters()
            .tenant()
            .direct_tenant_binding_admitted_count(),
        0
    );
}

#[test]
fn branch_denial_blocks_before_tenant_basis_is_admitted() {
    let canonical = canonical_query();
    let (policy, tenant, _, schema) = admitted_inputs();
    let branch = BranchAccessGrant::synthetic_denied("branch-a", "no_relationship_path", &policy);

    let error = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::CurrentRead,
    )
    .expect_err("branch denial must block before tenant truth basis");

    assert_eq!(
        error.failure_class(),
        PolicyTenantAdmissionFailureClass::BranchAccessDenied
    );
    assert_eq!(error.counters().policy().branch_access_denial_count(), 1);
    assert_eq!(
        error
            .counters()
            .tenant()
            .direct_tenant_binding_admitted_count(),
        0
    );
}

#[test]
fn hidden_tenant_filter_is_a_typed_denial_not_a_predicate_shortcut() {
    let canonical = canonical_query();
    let (policy, _, branch, schema) = admitted_inputs();
    let tenant = TenantBindingSnapshot::synthetic_hidden_filter(
        "tenant-a",
        "branch-a",
        "schema-a",
        TenantBasisEpoch::Synthetic(3),
    );

    let error = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::CurrentRead,
    )
    .expect_err("hidden tenant filters must not sneak into query predicates");

    assert_eq!(
        error.failure_class(),
        PolicyTenantAdmissionFailureClass::TenantAdmissionDenied
    );
    assert_eq!(
        error
            .counters()
            .tenant()
            .hidden_tenant_filter_denial_count(),
        1
    );
}

#[test]
fn schema_global_fallback_is_denied_for_tenant_scoped_admission() {
    let canonical = canonical_query();
    let (policy, tenant, branch, _) = admitted_inputs();
    let schema = SchemaVariantSnapshot::synthetic_global_fallback("tenant-a", "schema-a");

    let error = admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::HistoricalRead,
    )
    .expect_err("global schema fallback must be denied");

    assert_eq!(
        error.failure_class(),
        PolicyTenantAdmissionFailureClass::TenantAdmissionDenied
    );
    assert_eq!(
        error
            .counters()
            .tenant()
            .global_schema_fallback_denial_count(),
        1
    );
}

#[test]
fn support_profile_names_deferred_execution_surfaces() {
    let profile = runtime_backed_policy_tenant_admission_support_profile();

    assert!(profile
        .admitted_execution_modes()
        .contains(&PolicyExecutionModeRequest::CurrentRead));
    assert!(profile
        .deferred_execution_modes()
        .contains(&PolicyExecutionModeRequest::HistoricalDiff));
    assert!(profile
        .deferred_execution_modes()
        .contains(&PolicyExecutionModeRequest::LiveSubscription));
    assert!(profile.surfaces().contains(&(
        PolicyTenantPhaseOneSurface::BranchAccessGrant,
        PolicyTenantSupportStatus::Verified
    )));
    assert!(profile.surfaces().contains(&(
        PolicyTenantPhaseOneSurface::PolicyWorkBudget,
        PolicyTenantSupportStatus::Verified
    )));
    assert!(profile.surfaces().contains(&(
        PolicyTenantPhaseOneSurface::RelationshipProofLowering,
        PolicyTenantSupportStatus::Deferred
    )));
    assert!(profile.surfaces().contains(&(
        PolicyTenantPhaseOneSurface::DurableStoreBackedArtifacts,
        PolicyTenantSupportStatus::Deferred
    )));
    assert!(!profile.profile_digest().is_empty());
}
