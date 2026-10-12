use crate::runtime::{
    WorthQueryEphemeralGraphIndexScopeKind, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessRequirementKind,
};

use crate::runtime::tests::graph_read_access::support;

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    dense_traversal_family, workspace,
};
use support::aspect_touch as touch;
use support::graph_index_inventory::read_families::traversal_collection_family;
use support::graph_index_inventory::runtime_profiles::{
    profile_with_ephemeral_graph_support, workspace_with_graph_support,
};

#[test]
fn oversized_read_rejects_before_ephemeral_allocation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-eight.budget-rejects-first",
    );
    let family = dense_traversal_family(&mut workspace, "phase-eight-budget-rejects-first");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("budget-denied read should still be reviewable");
    let admission = review
        .graph_read_access_admission()
        .expect("admission evidence should exist before execution");

    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::Denied
    );
    assert_eq!(admission.cost_estimate().counters().edge_scan_count(), 0);
    assert_eq!(
        admission
            .cost_estimate()
            .counters()
            .access_buffer_allocation_count(),
        0
    );
    assert!(review.graph_read_access_plan().is_err());
}

#[test]
fn bounded_ephemeral_index_plan_is_visible_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.plan-visible",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = traversal_collection_family(&mut workspace, "phase-eight-plan-visible");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("read review should admit ephemeral graph access");
    let plan = review
        .graph_read_access_plan()
        .expect("bounded ephemeral access should lower to an admitted plan");
    let ephemeral_plan = plan
        .ephemeral_index_plan()
        .expect("bounded ephemeral admission should carry an ephemeral index plan");

    assert_eq!(
        plan.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex
    );
    assert_eq!(ephemeral_plan.required_allocations().len(), 1);
    assert!(
        ephemeral_plan.estimated_index_bytes() <= ephemeral_plan.admitted_byte_budget(),
        "ephemeral provisioning must be budget-admitted before execution"
    );
}

#[test]
fn bounded_ephemeral_index_receipt_proves_cleanup_after_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.cleanup",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = traversal_collection_family(&mut workspace, "phase-eight-cleanup");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("bounded ephemeral read should execute through admitted plan");
    let receipt = result
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("bounded ephemeral execution should attach provisioning receipt");

    assert_eq!(
        receipt.scope_kind(),
        &WorthQueryEphemeralGraphIndexScopeKind::ReadExecution
    );
    assert_eq!(receipt.active_resource_count_after_scope(), 0);
    assert_eq!(receipt.orphan_resource_count(), 0);
    assert_eq!(receipt.counters().allocation_attempt_count(), 1);
    assert_eq!(receipt.counters().successful_allocation_count(), 1);
    assert_eq!(
        receipt.counters().release_count(),
        receipt.counters().successful_allocation_count()
    );
    assert_eq!(receipt.counters().rejected_before_allocation_count(), 0);
    assert!(receipt.actual_allocated_bytes() <= receipt.admitted_byte_budget());
}

#[test]
fn bounded_ephemeral_index_rebuild_is_stable_for_same_snapshot_and_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut first_workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.equivalence",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let first_family = traversal_collection_family(&mut first_workspace, "phase-eight-equivalence");
    let first_receipt = first_workspace
        .read_family_intent(&first_family)
        .execute(request_execution)
        .expect("first bounded ephemeral read should execute")
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("first receipt should include ephemeral provisioning")
        .clone();

    let second_receipt = first_workspace
        .read_family_intent(&first_family)
        .execute(request_execution)
        .expect("second bounded ephemeral read should execute")
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("second receipt should include ephemeral provisioning")
        .clone();

    assert_eq!(first_receipt.digest(), second_receipt.digest());
    assert_eq!(first_receipt.index_digest(), second_receipt.index_digest());
    assert_eq!(
        first_receipt.actual_allocated_bytes(),
        second_receipt.actual_allocated_bytes()
    );
    assert_eq!(first_receipt.counters(), second_receipt.counters());
}

#[test]
fn bounded_ephemeral_receipt_changes_when_snapshot_scope_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut first_workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.scope-a",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let first_family =
        traversal_collection_family(&mut first_workspace, "phase-eight-scope-sensitive");
    let first_receipt = first_workspace
        .read_family_intent(&first_family)
        .execute(request_execution)
        .expect("first bounded ephemeral read should execute")
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("first receipt should include ephemeral provisioning")
        .clone();

    let mut second_workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.scope-b",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let second_family =
        traversal_collection_family(&mut second_workspace, "phase-eight-scope-sensitive");
    second_workspace
        .insert(request_execution, "user", |user| {
            user.set_aspect(
                touch("identity.id"),
                authored_text("phase-eight-scope-sensitive-user"),
            )
            .set_aspect(touch("status.value"), authored_text("active"))
        })
        .expect("state change should create a distinct snapshot scope");
    let second_receipt = second_workspace
        .read_family_intent(&second_family)
        .execute(request_execution)
        .expect("second bounded ephemeral read should execute")
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("second receipt should include ephemeral provisioning")
        .clone();

    assert_ne!(first_receipt.scope_digest(), second_receipt.scope_digest());
    assert_ne!(first_receipt.index_digest(), second_receipt.index_digest());
    assert_ne!(first_receipt.digest(), second_receipt.digest());
}

#[test]
fn bounded_ephemeral_receipt_is_bound_to_admitted_ephemeral_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-eight.receipt-plan-binding",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = traversal_collection_family(&mut workspace, "phase-eight-receipt-plan-binding");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("bounded ephemeral read should execute");
    let access_plan = result
        .receipt()
        .graph_read_access_plan()
        .expect("bounded ephemeral execution should attach access plan");
    let ephemeral_plan = access_plan
        .ephemeral_index_plan()
        .expect("bounded ephemeral access plan should carry ephemeral plan");
    let receipt = result
        .receipt()
        .ephemeral_graph_index_receipt()
        .expect("bounded ephemeral execution should attach receipt");

    assert_eq!(receipt.plan_digest(), ephemeral_plan.digest());
    assert_eq!(
        receipt.counters().touched_node_count(),
        ephemeral_plan.estimated_touched_nodes()
    );
    assert_eq!(
        receipt.counters().touched_edge_count(),
        ephemeral_plan.estimated_touched_edges()
    );
    assert!(!receipt.index_digest().is_empty());
}

#[test]
fn ordinary_inline_indexed_read_does_not_emit_ephemeral_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        support::graph_index_inventory::runtime_profiles::default_graph_support_workspace(
            request_execution,
            "graph-read-access.phase-eight.inline-no-ephemeral",
        );
    let family = traversal_collection_family(&mut workspace, "phase-eight-inline");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("inline indexed read should execute");

    assert!(result.receipt().ephemeral_graph_index_receipt().is_none());
    assert_eq!(
        result
            .receipt()
            .graph_read_access_plan()
            .expect("inline read should still attach access plan")
            .posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
}

fn authored_text(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}
