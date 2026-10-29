use crate::authoring::{
    AspectFieldKey, AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath,
    RawAuthoredQuery, RawAuthoredResultShape, RootEntityKey,
};
use crate::authorized_projection::{PolicyAspectMask, PolicyInfluenceSet, PolicyMaskSnapshot};
use crate::policy_basis::{
    admit_policy_tenant_context, BranchAccessGrant, PolicyEpoch, PolicyExecutionModeRequest,
    PolicyRuleSnapshot,
};
use crate::policy_narrowing::narrow_policy_query;
use crate::relationship_proof::RelationshipProofDescriptorSet;
use crate::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

use super::{
    deny_durable_policy_artifact_reload_claim, deny_durable_policy_cursor_claim,
    deny_durable_policy_delivery_metadata_reload_claim, deny_policy_cross_tenant_fanout_claim,
    deny_policy_per_row_allocation_claim, deny_saved_query_policy_bypass_claim,
    deny_unsupported_policy_workflow_composition_claim,
    runtime_backed_policy_execution_seam_handoff_report,
    runtime_backed_policy_execution_seam_support_profile, PolicyAwareExecutionMode,
    PolicyAwareExecutionSeam, PolicyAwareExecutionSeamFailureClass, PolicyAwareSeamCounters,
    PolicyExecutionSeamSupportStatus, PolicyExecutionSeamSurface,
};

fn narrowed() -> crate::policy_narrowing::NarrowedPolicyQueryArtifact {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .project(AspectFieldSelector::new("secret", "salary").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let policy = PolicyRuleSnapshot::synthetic_authority_with_projection(
        "runtime-policy",
        "rules-v1",
        PolicyEpoch::Synthetic(7),
        PolicyAspectMask::allow_all()
            .with_masked(AspectFieldKey::from_authoring_parts("secret", "salary").unwrap()),
    );
    let admitted = admit_policy_tenant_context(
        canonical.query(),
        policy.clone(),
        TenantBindingSnapshot::synthetic_direct(
            "tenant-a",
            "branch-a",
            "schema-a",
            TenantBasisEpoch::Synthetic(3),
        ),
        BranchAccessGrant::synthetic_granted("branch-a", &policy),
        SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "compatible"),
        PolicyExecutionModeRequest::CurrentRead,
    )
    .unwrap();
    let mask = PolicyMaskSnapshot::synthetic_authority(
        admitted.bundle().policy_digest(),
        PolicyAspectMask::allow_all()
            .with_masked(AspectFieldKey::from_authoring_parts("secret", "salary").unwrap()),
    );
    narrow_policy_query(
        &canonical,
        admitted,
        mask,
        PolicyInfluenceSet::none(),
        RelationshipProofDescriptorSet::none(),
    )
    .unwrap()
}

#[test]
fn seam_identity_binds_narrowed_artifact_components() {
    let artifact = narrowed();
    let seam = PolicyAwareExecutionSeam::from_narrowed(
        &artifact,
        PolicyAwareExecutionMode::CurrentRead,
        PolicyAwareSeamCounters::admitted(1, 0, 1, 0, 12),
    );

    assert_eq!(seam.source_narrowed_artifact_digest(), artifact.digest());
    assert_eq!(seam.policy_digest(), artifact.policy_digest());
    assert_eq!(
        seam.tenant_schema_basis_digest(),
        artifact.tenant_schema_basis_digest()
    );
    assert_eq!(seam.mode(), PolicyAwareExecutionMode::CurrentRead);
    assert_eq!(seam.counters().executor_semantic_rediscovery_count(), 0);
    assert!(!seam.identity().as_str().is_empty());
}

#[test]
fn support_profile_and_handoff_report_keep_store_and_durable_debt_explicit() {
    let profile = runtime_backed_policy_execution_seam_support_profile();
    assert!(profile.surfaces().contains(&(
        PolicyExecutionSeamSurface::StoreBackedRetainedHistoricalExecution,
        PolicyExecutionSeamSupportStatus::LimitedAdmission
    )));
    assert!(profile.surfaces().contains(&(
        PolicyExecutionSeamSurface::DurablePolicyCursor,
        PolicyExecutionSeamSupportStatus::Deferred
    )));
    assert!(profile.surfaces().contains(&(
        PolicyExecutionSeamSurface::DurablePolicyArtifactReload,
        PolicyExecutionSeamSupportStatus::Deferred
    )));

    let handoff = runtime_backed_policy_execution_seam_handoff_report();
    assert_eq!(handoff.runtime_backed_verified_surface_count(), 7);
    assert_eq!(handoff.limited_admission_surface_count(), 1);
    assert_eq!(handoff.blocked_or_deferred_surface_count(), 2);
    assert!(handoff
        .milestone_ten_store_backed_handoff()
        .contains(&"store-backed diff execution parity"));
    assert!(handoff
        .milestone_eleven_durable_handoff()
        .contains(&"durable delivery cursors"));
    assert!(!handoff.handoff_digest().is_empty());
}

#[test]
fn durable_overclaims_are_typed_denials_with_exact_counters() {
    let cursor = deny_durable_policy_cursor_claim();
    let reload = deny_durable_policy_artifact_reload_claim();
    let delivery = deny_durable_policy_delivery_metadata_reload_claim();

    assert_eq!(
        cursor.failure_class(),
        PolicyAwareExecutionSeamFailureClass::DurablePolicyCursorDeferred
    );
    assert_eq!(cursor.counters().durable_cursor_deferred_count(), 1);
    assert_eq!(cursor.counters().durable_overclaim_denial_count(), 1);
    assert_eq!(
        reload.failure_class(),
        PolicyAwareExecutionSeamFailureClass::DurablePolicyArtifactReloadDeferred
    );
    assert_eq!(
        reload.counters().durable_artifact_reload_deferred_count(),
        1
    );
    assert_eq!(
        delivery.failure_class(),
        PolicyAwareExecutionSeamFailureClass::DurablePolicyDeliveryMetadataDeferred
    );
    assert_eq!(
        delivery
            .counters()
            .durable_delivery_metadata_deferred_count(),
        1
    );
}

#[test]
fn execution_shortcuts_are_typed_denials_with_exact_counters() {
    let allocation = deny_policy_per_row_allocation_claim();
    let fanout = deny_policy_cross_tenant_fanout_claim();
    let saved = deny_saved_query_policy_bypass_claim();
    let workflow = deny_unsupported_policy_workflow_composition_claim();

    assert_eq!(
        allocation.failure_class(),
        PolicyAwareExecutionSeamFailureClass::PerRowPolicyAllocationForbidden
    );
    assert_eq!(allocation.counters().per_row_allocation_denial_count(), 1);
    assert_eq!(
        fanout.failure_class(),
        PolicyAwareExecutionSeamFailureClass::CrossTenantPolicyFanoutForbidden
    );
    assert_eq!(fanout.counters().cross_tenant_fanout_denial_count(), 1);
    assert_eq!(
        saved.failure_class(),
        PolicyAwareExecutionSeamFailureClass::SavedQueryPolicyBypassForbidden
    );
    assert_eq!(saved.counters().saved_query_policy_bypass_denial_count(), 1);
    assert_eq!(
        workflow.failure_class(),
        PolicyAwareExecutionSeamFailureClass::UnsupportedPolicyWorkflowComposition
    );
    assert_eq!(
        workflow
            .counters()
            .unsupported_policy_workflow_composition_denial_count(),
        1
    );
}
