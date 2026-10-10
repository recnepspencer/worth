use crate::runtime::tests::graph_read_access::support;

use crate::facade::policy::PolicyExecutionModeRequest;
use crate::runtime::{
    WorthQueryGraphReadAccessAuthorityDenialKind, WorthQueryGraphReadAccessAuthorityRequest,
    WorthQueryGraphReadAccessBasisScopeKind,
};

use support::graph_index_inventory::runtime_profiles::default_graph_support_workspace;
use support::graph_read_access::authority_assertions::assert_authority_denial_before_buffers;
use support::graph_read_access::authority_scenarios::{
    admitted_policy_tenant, admitted_policy_tenant_for_mode, canonical_query, session_label,
};
use support::graph_read_access::read_surface_declarations::graph_access_family;

#[test]
fn branch_basis_rejects_current_read_policy_tenant_before_buffers() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-fourteen.mode-mismatch",
    );
    let branch_basis = {
        let branch = workspace
            .branch(request_execution, session_label("branch-mode-mismatch"))
            .expect("branch basis should admit");
        branch.basis_admission().clone()
    };
    let canonical = canonical_query();
    let current_policy_tenant = admitted_policy_tenant(&canonical, "tenant-branch-mismatch");

    let denial = crate::runtime::admit_graph_read_access_authority(
        WorthQueryGraphReadAccessAuthorityRequest::branch(&branch_basis)
            .with_policy_tenant(current_policy_tenant),
    )
    .expect_err("branch basis must require branch-read policy/tenant admission");

    assert_eq!(
        denial.kind(),
        WorthQueryGraphReadAccessAuthorityDenialKind::PolicyTenantBasisScopeMismatch
    );
    assert_authority_denial_before_buffers(&denial);
}

#[test]
fn branch_basis_accepts_branch_read_policy_tenant_and_executes_reviewed_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-fourteen.branch-positive",
    );
    let family = graph_access_family(&mut workspace, "phase-fourteen-branch-positive-family");
    let branch_basis = {
        let branch = workspace
            .branch(request_execution, session_label("branch-positive"))
            .expect("branch basis should admit");
        branch.basis_admission().clone()
    };
    let canonical = canonical_query();
    let branch_policy_tenant = admitted_policy_tenant_for_mode(
        &canonical,
        "tenant-branch-positive",
        PolicyExecutionModeRequest::BranchRead,
    );
    let branch_authority = workspace
        .admit_graph_read_access_authority(
            WorthQueryGraphReadAccessAuthorityRequest::branch(&branch_basis)
                .with_policy_tenant(branch_policy_tenant),
        )
        .expect("branch-read policy/tenant authority should admit");
    let review = workspace
        .read_family_intent_in_graph_read_authority(&family, &branch_authority)
        .review()
        .expect("branch authority read review should admit");
    let reviewed_plan = review
        .graph_read_access_plan()
        .expect("branch authority review should expose plan");
    let reviewed_plan_digest = reviewed_plan.digest().to_string();
    let result = review
        .admit()
        .expect("branch authority read should admit")
        .execute(request_execution)
        .expect("branch authority read should execute");

    assert_eq!(
        reviewed_plan
            .admission()
            .authority_receipt()
            .basis_scope()
            .kind(),
        WorthQueryGraphReadAccessBasisScopeKind::Branch
    );
    assert_eq!(
        result
            .receipt()
            .graph_read_access_plan()
            .expect("branch execution receipt should carry reviewed plan")
            .digest(),
        reviewed_plan_digest
    );
    assert_eq!(
        result
            .receipt()
            .graph_read_access_summary()
            .expect("branch execution receipt should carry access summary")
            .authority_receipt_digest(),
        branch_authority.receipt().digest()
    );
}
