use crate::runtime::tests::graph_read_access::support;

use crate::facade::policy::{
    admit_relationship_proofs, RelationshipProofBudget, RelationshipProofDescriptor,
    RelationshipProofDescriptorSet,
};
use crate::runtime::{
    WorthQueryGraphReadAccessAuthorityDenialKind, WorthQueryGraphReadAccessAuthorityRequest,
    WorthQueryGraphReadAccessBasisScopeKind,
};

use support::graph_index_inventory::runtime_profiles::default_graph_support_workspace;
use support::graph_read_access::authority_assertions::assert_authority_denial_before_buffers;
use support::graph_read_access::authority_scenarios::{
    admitted_policy_tenant, canonical_query, policy_tenant_authority_request, session_label,
};
use support::graph_read_access::read_surface_declarations::graph_access_family;

#[test]
fn current_preview_and_branch_use_same_requirements_with_distinct_authority_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-fourteen.authority-parity",
    );
    let family = graph_access_family(&mut workspace, "phase-fourteen-authority-family");

    let current_authority =
        workspace
            .admit_graph_read_access_authority(
                WorthQueryGraphReadAccessAuthorityRequest::current_head(),
            )
            .expect("current-head authority should admit");
    let current = workspace
        .admit_graph_read_access_in_authority(&family, &current_authority)
        .expect("current-head graph access should admit");

    let preview_basis = {
        let preview = workspace
            .preview(request_execution, session_label("preview"))
            .expect("preview basis should admit");
        preview.basis_admission().clone()
    };
    let preview_authority = workspace
        .admit_graph_read_access_authority(WorthQueryGraphReadAccessAuthorityRequest::preview(
            &preview_basis,
        ))
        .expect("preview authority should admit");
    let preview = workspace
        .admit_graph_read_access_in_authority(&family, &preview_authority)
        .expect("preview graph access should admit");

    let branch_basis = {
        let branch = workspace
            .branch(request_execution, session_label("branch"))
            .expect("branch basis should admit");
        branch.basis_admission().clone()
    };
    let branch_authority = workspace
        .admit_graph_read_access_authority(WorthQueryGraphReadAccessAuthorityRequest::branch(
            &branch_basis,
        ))
        .expect("branch authority should admit");
    let branch = workspace
        .admit_graph_read_access_in_authority(&family, &branch_authority)
        .expect("branch graph access should admit");

    assert_eq!(
        current.requirement_set().rows(),
        preview.requirement_set().rows()
    );
    assert_eq!(
        current.requirement_set().rows(),
        branch.requirement_set().rows()
    );
    assert_ne!(
        current.requirement_set().digest(),
        preview.requirement_set().digest()
    );
    assert_ne!(
        current.requirement_set().digest(),
        branch.requirement_set().digest()
    );
    assert_eq!(
        preview.authority_receipt().basis_scope().kind(),
        WorthQueryGraphReadAccessBasisScopeKind::Preview
    );
    assert_eq!(
        branch.authority_receipt().basis_scope().kind(),
        WorthQueryGraphReadAccessBasisScopeKind::Branch
    );

    let preview_review = workspace
        .read_family_intent_in_graph_read_authority(&family, &preview_authority)
        .review()
        .expect("preview authority read review should admit");
    let preview_plan_digest = preview_review
        .graph_read_access_plan()
        .expect("preview authority review should expose plan")
        .digest()
        .to_string();
    let preview_result = preview_review
        .admit()
        .expect("preview authority read should admit")
        .execute(request_execution)
        .expect("preview authority read should execute");
    let branch_review = workspace
        .read_family_intent_in_graph_read_authority(&family, &branch_authority)
        .review()
        .expect("branch authority read review should admit");
    let branch_plan_digest = branch_review
        .graph_read_access_plan()
        .expect("branch authority review should expose plan")
        .digest()
        .to_string();
    let branch_result = branch_review
        .admit()
        .expect("branch authority read should admit")
        .execute(request_execution)
        .expect("branch authority read should execute");

    assert_eq!(
        preview_result
            .receipt()
            .graph_read_access_plan()
            .expect("preview receipt should carry authority-bound plan")
            .digest(),
        preview_plan_digest
    );
    assert_eq!(
        preview_result
            .receipt()
            .graph_read_access_summary()
            .expect("preview receipt should expose authority digest")
            .authority_receipt_digest(),
        preview_authority.receipt().digest()
    );
    assert_eq!(
        branch_result
            .receipt()
            .graph_read_access_plan()
            .expect("branch receipt should carry authority-bound plan")
            .digest(),
        branch_plan_digest
    );
    assert_eq!(
        branch_result
            .receipt()
            .graph_read_access_summary()
            .expect("branch receipt should expose authority digest")
            .authority_receipt_digest(),
        branch_authority.receipt().digest()
    );
}

#[test]
fn tenant_and_relationship_proofs_enter_shape_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-fourteen.policy-proof",
    );
    let family = graph_access_family(&mut workspace, "phase-fourteen-policy-proof-family");
    let canonical = canonical_query();
    let policy_tenant = admitted_policy_tenant(&canonical, "tenant-a");
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::tenant_membership(
            policy_tenant.bundle().tenant_schema_basis_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );
    let (relationship_proofs, proof_counters) =
        admit_relationship_proofs(canonical.query(), &policy_tenant, &descriptors)
            .expect("relationship proofs should admit");

    let plain_authority =
        workspace
            .admit_graph_read_access_authority(
                WorthQueryGraphReadAccessAuthorityRequest::current_head(),
            )
            .expect("plain authority should admit");
    let narrowed_authority = workspace
        .admit_graph_read_access_authority(
            WorthQueryGraphReadAccessAuthorityRequest::current_head()
                .with_policy_tenant(policy_tenant)
                .with_relationship_proofs(relationship_proofs),
        )
        .expect("narrowed authority should admit");

    let plain = workspace
        .admit_graph_read_access_in_authority(&family, &plain_authority)
        .expect("plain planning should admit");
    let narrowed = workspace
        .admit_graph_read_access_in_authority(&family, &narrowed_authority)
        .expect("narrowed planning should admit");

    assert_eq!(proof_counters.truth_touch_count(), 0);
    assert_ne!(
        plain.requirement_set().access_shape_digest(),
        narrowed.requirement_set().access_shape_digest()
    );
    assert!(narrowed
        .authority_receipt()
        .policy_tenant_digest()
        .is_some());
    assert!(narrowed
        .authority_receipt()
        .relationship_proof_digest()
        .is_some());

    let review = workspace
        .read_family_intent_in_graph_read_authority(&family, &narrowed_authority)
        .review()
        .expect("authority-bound read review should admit");
    let reviewed_plan = review
        .graph_read_access_plan()
        .expect("authority-bound review should expose admitted access plan");
    let reviewed_plan_digest = reviewed_plan.digest().to_string();
    let reviewed_admission_digest = reviewed_plan.admission().digest().to_string();
    let reviewed_authority_digest = reviewed_plan
        .admission()
        .authority_receipt()
        .digest()
        .to_string();
    let result = review
        .admit()
        .expect("authority-bound read should admit")
        .execute(request_execution)
        .expect("authority-bound read should execute with reviewed plan");
    let receipt = result.receipt();
    let receipt_plan = receipt
        .graph_read_access_plan()
        .expect("execution receipt should carry graph-read access plan");
    let receipt_summary = receipt
        .graph_read_access_summary()
        .expect("execution receipt should carry graph-read access summary");

    assert_eq!(receipt_plan.digest(), reviewed_plan_digest);
    assert_eq!(
        receipt_summary.admission_digest(),
        reviewed_admission_digest
    );
    assert_eq!(
        receipt_summary.authority_receipt_digest(),
        reviewed_authority_digest
    );
}

#[test]
fn policy_tenant_denial_stops_before_graph_access_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-fourteen.policy-denial",
    );
    let canonical = canonical_query();
    let request = policy_tenant_authority_request(&canonical, "tenant-policy-denial", false);

    let denial = workspace
        .admit_graph_read_access_authority_from_policy_tenant_request(request)
        .expect_err("policy-denied authority request must deny before graph access planning");

    assert_eq!(
        denial.kind(),
        WorthQueryGraphReadAccessAuthorityDenialKind::PolicyTenantDenied
    );
    assert_authority_denial_before_buffers(&denial);
}

#[test]
fn relationship_proof_without_policy_tenant_denies_before_buffers() {
    let canonical = canonical_query();
    let policy_tenant = admitted_policy_tenant(&canonical, "tenant-denial");
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::tenant_membership(
            policy_tenant.bundle().tenant_schema_basis_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );
    let (relationship_proofs, _) =
        admit_relationship_proofs(canonical.query(), &policy_tenant, &descriptors)
            .expect("relationship proof should admit before authority composition");

    let denial = crate::runtime::admit_graph_read_access_authority(
        WorthQueryGraphReadAccessAuthorityRequest::current_head()
            .with_relationship_proofs(relationship_proofs),
    )
    .expect_err("relationship proof without policy/tenant context must deny");

    assert_eq!(
        denial.kind(),
        WorthQueryGraphReadAccessAuthorityDenialKind::RelationshipProofRequiresPolicyTenantContext
    );
    assert_authority_denial_before_buffers(&denial);
}

#[test]
fn relationship_proof_bound_to_different_tenant_denies_before_buffers() {
    let canonical = canonical_query();
    let policy_tenant_a = admitted_policy_tenant(&canonical, "tenant-a");
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::tenant_membership(
            policy_tenant_a.bundle().tenant_schema_basis_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );
    let (relationship_proofs, _) =
        admit_relationship_proofs(canonical.query(), &policy_tenant_a, &descriptors)
            .expect("relationship proof should admit against tenant a");
    let policy_tenant_b = admitted_policy_tenant(&canonical, "tenant-b");

    let denial = crate::runtime::admit_graph_read_access_authority(
        WorthQueryGraphReadAccessAuthorityRequest::current_head()
            .with_policy_tenant(policy_tenant_b)
            .with_relationship_proofs(relationship_proofs),
    )
    .expect_err("relationship proof must not drift across policy/tenant contexts");

    assert_eq!(
        denial.kind(),
        WorthQueryGraphReadAccessAuthorityDenialKind::RelationshipProofPolicyTenantMismatch
    );
    assert_authority_denial_before_buffers(&denial);
}
