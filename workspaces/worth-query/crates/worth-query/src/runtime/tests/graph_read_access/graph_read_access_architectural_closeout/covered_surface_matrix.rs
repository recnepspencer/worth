use crate::facade::policy::{
    admit_relationship_proofs, PolicyExecutionModeRequest, RelationshipProofBudget,
    RelationshipProofDescriptor, RelationshipProofDescriptorSet,
};
use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessAuthorityRequest,
    WorthQueryGraphReadAccessBasisScopeKind, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryReadFamily, WorthQueryReadResult, WorthQueryUnrefinedLiveShape, WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support::graph_index_inventory::runtime_profiles::{
    default_graph_support_workspace, profile_with_ephemeral_graph_support,
    workspace_with_graph_support,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::authority_scenarios::{
    admitted_policy_tenant, admitted_policy_tenant_for_mode, canonical_query, session_label,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::hostile_graph_fixture::seed_hostile_frontier_graph;
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_declarations::graph_access_family;
use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    frontier_search_family, projection_only_family, workspace,
};

#[test]
fn closeout_local_anchored_frontier_and_reusable_reads_emit_consumed_access_plan_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_consumed_access_plan_receipt_for_family(
        request_execution,
        "local",
        "graph-read-access.closeout.surface.local",
        |workspace| projection_only_family(workspace, "closeout-local"),
    );
    assert_consumed_access_plan_receipt_for_family(
        request_execution,
        "anchored",
        "graph-read-access.closeout.surface.anchored",
        |workspace| graph_access_family(workspace, "closeout-anchored"),
    );

    let mut frontier_workspace = workspace(
        request_execution,
        "graph-read-access.closeout.surface.frontier",
    );
    let hostile_graph = seed_hostile_frontier_graph(
        request_execution,
        &mut frontier_workspace,
        "closeout-frontier",
    );
    assert!(hostile_graph.active_user_count() > 32);
    assert!(hostile_graph.relation_edge_count() > hostile_graph.user_count());
    assert!(hostile_graph.branching_factor() > 1);
    let frontier = frontier_search_family(&mut frontier_workspace, "closeout-frontier");
    let frontier_result = frontier_workspace
        .read_family_intent(&frontier)
        .execute(request_execution)
        .expect("frontier read should execute through admitted access plan");
    let frontier_summary = assert_consumed_access_plan_receipt("frontier", &frontier_result);
    // With no inline byte forecast, a hostile frontier executes inline.
    assert_eq!(
        &frontier_summary.admission_posture,
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
    assert!(frontier_result
        .receipt()
        .graph_read_streaming_receipt()
        .is_none());

    let mut reusable_workspace = workspace(
        request_execution,
        "graph-read-access.closeout.surface.reusable",
    );
    let reusable = graph_access_family(&mut reusable_workspace, "closeout-reusable");
    let reviewed_plan = reusable_workspace
        .read_family_intent(&reusable)
        .review()
        .expect("reusable read should review")
        .graph_read_access_plan()
        .expect("reusable read should expose explicit access plan");
    let reviewed_plan_digest = reviewed_plan.digest().to_string();
    let reusable_result = reusable_workspace
        .execute_read_family_with_access_plan(request_execution, &reusable, reviewed_plan)
        .expect("reusable read should execute with explicit reviewed plan");
    assert_eq!(
        assert_consumed_access_plan_receipt("reusable", &reusable_result).plan_digest,
        reviewed_plan_digest
    );
}

#[test]
fn closeout_policy_tenant_relationship_surface_executes_with_authority_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut authority_workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.closeout.surface.authority",
    );
    let authority_family = graph_access_family(&mut authority_workspace, "closeout-authority");
    let canonical = canonical_query();
    let policy_tenant = admitted_policy_tenant(&canonical, "closeout-tenant");
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::tenant_membership(
            policy_tenant.bundle().tenant_schema_basis_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );
    let (relationship_proofs, proof_counters) =
        admit_relationship_proofs(canonical.query(), &policy_tenant, &descriptors)
            .expect("relationship proof should admit before access authority");
    assert_eq!(proof_counters.truth_touch_count(), 0);
    let narrowed_authority = authority_workspace
        .admit_graph_read_access_authority(
            WorthQueryGraphReadAccessAuthorityRequest::current_head()
                .with_policy_tenant(policy_tenant)
                .with_relationship_proofs(relationship_proofs),
        )
        .expect("policy/tenant relationship authority should admit");
    let narrowed_result = authority_workspace
        .read_family_intent_in_graph_read_authority(&authority_family, &narrowed_authority)
        .execute(request_execution)
        .expect("policy/tenant relationship read should execute through access plan");
    assert_eq!(
        assert_consumed_access_plan_receipt("policy-relationship", &narrowed_result)
            .authority_receipt_digest,
        narrowed_authority.receipt().digest()
    );
}

#[test]
fn closeout_preview_surface_executes_with_preview_authority_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut authority_workspace, authority_family) =
        closeout_authority_workspace_and_family(request_execution, "preview");
    let preview_authority = {
        let preview_basis = authority_workspace
            .preview(request_execution, session_label("closeout-preview"))
            .expect("preview basis should admit")
            .basis_admission()
            .clone();
        authority_workspace
            .admit_graph_read_access_authority(WorthQueryGraphReadAccessAuthorityRequest::preview(
                &preview_basis,
            ))
            .expect("preview authority should admit")
    };
    let preview_result = authority_workspace
        .read_family_intent_in_graph_read_authority(&authority_family, &preview_authority)
        .execute(request_execution)
        .expect("preview read should execute through access plan");
    assert_eq!(
        preview_authority.receipt().basis_scope().kind(),
        WorthQueryGraphReadAccessBasisScopeKind::Preview
    );
    assert_eq!(
        assert_consumed_access_plan_receipt("preview", &preview_result).authority_receipt_digest,
        preview_authority.receipt().digest()
    );
}

#[test]
fn closeout_branch_surface_executes_with_branch_authority_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut authority_workspace, authority_family) =
        closeout_authority_workspace_and_family(request_execution, "branch");
    let canonical = canonical_query();
    let branch_authority = {
        let branch_basis = authority_workspace
            .branch(request_execution, session_label("closeout-branch"))
            .expect("branch basis should admit")
            .basis_admission()
            .clone();
        let branch_policy_tenant = admitted_policy_tenant_for_mode(
            &canonical,
            "closeout-branch-tenant",
            PolicyExecutionModeRequest::BranchRead,
        );
        authority_workspace
            .admit_graph_read_access_authority(
                WorthQueryGraphReadAccessAuthorityRequest::branch(&branch_basis)
                    .with_policy_tenant(branch_policy_tenant),
            )
            .expect("branch authority should admit")
    };
    let branch_result = authority_workspace
        .read_family_intent_in_graph_read_authority(&authority_family, &branch_authority)
        .execute(request_execution)
        .expect("branch read should execute through access plan");
    assert_eq!(
        branch_authority.receipt().basis_scope().kind(),
        WorthQueryGraphReadAccessBasisScopeKind::Branch
    );
    assert_eq!(
        assert_consumed_access_plan_receipt("branch", &branch_result).authority_receipt_digest,
        branch_authority.receipt().digest()
    );
}

#[test]
fn closeout_live_promoted_surface_emits_live_access_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_live_read_receipt_proves_no_caller_owned_n_plus_one(request_execution);
}

#[test]
fn closeout_ephemeral_covered_read_receipt_proves_bounded_runtime_index_use() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.closeout.surface.ephemeral",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = graph_access_family(&mut workspace, "closeout-ephemeral-surface");
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("ephemeral read should execute through admitted access plan");
    let counters = result
        .receipt()
        .graph_read_access_complexity_counters()
        .expect("ephemeral receipt should expose access counters");

    assert_consumed_access_plan_receipt("ephemeral", &result);
    assert_eq!(counters.per_result_neighbor_lookup_count(), 0);
    assert_eq!(counters.persistent_artifact_bypass_count(), 0);
}

fn assert_consumed_access_plan_receipt_for_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
    workspace_name: &str,
    family: impl FnOnce(&mut WorthQueryWorkspace) -> crate::runtime::WorthQueryReadFamily,
) {
    let mut workspace = workspace(request_execution, workspace_name);
    let family = family(&mut workspace);
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("covered read should execute through admitted access plan");

    assert_consumed_access_plan_receipt(label, &result);
}

fn closeout_authority_workspace_and_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> (WorthQueryWorkspace, WorthQueryReadFamily) {
    let mut workspace = default_graph_support_workspace(
        request_execution,
        &format!("graph-read-access.closeout.surface.{label}"),
    );
    let family = graph_access_family(&mut workspace, &format!("closeout-{label}-authority"));
    (workspace, family)
}

fn assert_consumed_access_plan_receipt(
    label: &str,
    result: &WorthQueryReadResult,
) -> CloseoutAccessReceiptDigestReadout {
    let receipt = result.receipt();
    let plan = receipt
        .graph_read_access_plan()
        .unwrap_or_else(|| panic!("{label} receipt should carry access plan"));
    let summary = receipt
        .graph_read_access_summary()
        .unwrap_or_else(|| panic!("{label} receipt should carry access summary"));
    let consumption = receipt
        .graph_read_access_plan_consumption()
        .unwrap_or_else(|| panic!("{label} receipt should carry plan consumption"));

    assert_eq!(summary.plan_digest(), plan.digest());
    assert_eq!(consumption.admitted_plan_digest(), plan.digest());
    assert_eq!(consumption.admission_digest(), summary.admission_digest());
    assert_eq!(consumption.execution_counters().executor_entry_count(), 1);
    assert_eq!(
        consumption.execution_counters().strategy_recompute_count(),
        0
    );
    assert_eq!(
        consumption
            .execution_counters()
            .per_result_neighbor_lookup_count(),
        0
    );
    CloseoutAccessReceiptDigestReadout {
        plan_digest: summary.plan_digest().to_string(),
        authority_receipt_digest: summary.authority_receipt_digest().to_string(),
        admission_posture: summary.admission_posture().clone(),
    }
}

struct CloseoutAccessReceiptDigestReadout {
    plan_digest: String,
    authority_receipt_digest: String,
    admission_posture: WorthQueryGraphReadAccessAdmissionPosture,
}

fn assert_live_read_receipt_proves_no_caller_owned_n_plus_one(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let mut workspace = workspace(request_execution, "graph-read-access.closeout.surface.live");
    let live_view = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.closeout.table",
            |query| {
                query
                    .from("Task")
                    .select([
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("graph-read-access-closeout-live")
            },
        )
        .expect("live view should declare");
    let result = workspace
        .read_live_result(request_execution, &live_view)
        .expect("live read should execute");
    let receipt = result
        .receipt()
        .live_graph_read_access()
        .expect("live read receipt should expose graph access proof");

    assert!(!receipt.live_access_plan_digest().is_empty());
    assert!(receipt.proves_no_caller_owned_n_plus_one());
    assert_eq!(
        receipt
            .maintenance_counters()
            .per_result_neighbor_lookup_count(),
        0
    );
}
