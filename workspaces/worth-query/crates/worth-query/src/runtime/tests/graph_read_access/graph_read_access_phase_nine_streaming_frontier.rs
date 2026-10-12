use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadBudgetClassKind,
    WorthQueryGraphReadStreamingCursorDenialKind, WorthQueryGraphReadStreamingReceipt,
};

use crate::runtime::tests::graph_read_access::support;

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    dense_traversal_family, frontier_search_family, projection_only_family, workspace,
};
use support::aspect_touch as touch;

#[test]
fn frontier_read_admits_streaming_plan_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.plan-visible",
    );
    let family = frontier_search_family(&mut workspace, "phase-nine-plan-visible");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("streaming frontier read should be reviewable");
    let admission = review
        .graph_read_access_admission()
        .expect("streaming admission evidence should exist");
    let plan = review
        .graph_read_access_plan()
        .expect("streaming frontier should lower to an admitted plan");
    let streaming_plan = plan
        .streaming_plan()
        .expect("streaming posture should carry a streaming plan");

    assert!(admission.is_admitted());
    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::AdmittedPagedStreaming
    );
    assert_eq!(
        admission.budget_check().class().kind(),
        &WorthQueryGraphReadBudgetClassKind::ExceedsInlineEphemeralBudget
    );
    assert_eq!(
        plan.execution_strategy(),
        "paged-streaming-frontier-read-execution"
    );
    assert_eq!(streaming_plan.admission_digest(), admission.digest());
    assert_eq!(
        streaming_plan.requirement_set_digest(),
        admission.requirement_set().digest().render_support_hex()
    );
    assert!(streaming_plan.page_budget().max_page_width() > 0);
}

#[test]
fn streaming_execution_receipt_pages_result_with_cursor_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.receipt-pages",
    );
    let identities = seed_active_users(request_execution, &mut workspace, "phase-nine-receipt", 3);
    seed_frontier_edges(
        request_execution,
        &mut workspace,
        "phase-nine-receipt",
        &identities,
    );
    let family = frontier_search_family(&mut workspace, "phase-nine-receipt-pages");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("streaming frontier read should execute");
    let access_plan = result
        .receipt()
        .graph_read_access_plan()
        .expect("execution receipt should include access plan");
    let streaming_plan = access_plan
        .streaming_plan()
        .expect("execution access plan should include streaming plan");
    let streaming_receipt = result
        .receipt()
        .graph_read_streaming_receipt()
        .expect("streaming execution should attach streaming receipt");
    let first_cursor = streaming_receipt
        .first_page_receipt()
        .and_then(|page| page.next_cursor())
        .expect("multi-row streaming execution should expose an opaque cursor");

    assert_eq!(
        access_plan.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::AdmittedPagedStreaming
    );
    assert_eq!(
        streaming_receipt.streaming_plan_digest(),
        streaming_plan.digest()
    );
    assert_eq!(
        streaming_receipt.convergence_result_digest(),
        result.receipt().result_digest()
    );
    assert_eq!(
        streaming_receipt.counters().emitted_row_count(),
        result.rows().len()
    );
    assert!(streaming_receipt.counters().page_count() >= 2);
    assert!(
        streaming_receipt
            .counters()
            .max_resident_frontier_observed()
            <= streaming_plan.page_budget().max_resident_frontier()
    );
    assert!(
        streaming_receipt.counters().max_resident_visited_observed()
            <= streaming_plan.page_budget().max_resident_visited()
    );
    assert_eq!(
        first_cursor.streaming_plan_digest(),
        streaming_plan.digest()
    );
    assert_eq!(
        first_cursor.snapshot_identity_digest(),
        streaming_receipt.snapshot_identity_digest()
    );
}

#[test]
fn streaming_cursor_session_denies_replay_and_skipped_sequence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let receipt = seeded_streaming_receipt(
        request_execution,
        "graph-read-access.phase-nine.cursor-session",
    );
    let first_cursor = receipt
        .page_receipts()
        .first()
        .and_then(|page| page.next_cursor())
        .expect("first streaming page should emit cursor");
    let skipped_cursor = receipt
        .page_receipts()
        .get(1)
        .and_then(|page| page.next_cursor())
        .expect("second streaming page should emit cursor");
    let mut session = receipt.open_cursor_session();

    let resumed_page = session
        .resume(first_cursor)
        .expect("first cursor should resume the next page");
    let replay_denial = session
        .resume(first_cursor)
        .expect_err("replaying the same cursor should be denied");
    let mut skipped_session = receipt.open_cursor_session();
    let skipped_denial = skipped_session
        .resume(skipped_cursor)
        .expect_err("skipping the first cursor should be denied");

    assert_eq!(
        resumed_page.page_ordinal(),
        first_cursor.next_page_ordinal()
    );
    assert_eq!(session.consumed_cursor_count(), 1);
    assert_eq!(
        replay_denial.kind(),
        &WorthQueryGraphReadStreamingCursorDenialKind::CursorReplayDenied
    );
    assert_eq!(
        skipped_denial.kind(),
        &WorthQueryGraphReadStreamingCursorDenialKind::CursorSequenceSkipped
    );
}

#[test]
fn dense_broad_read_remains_denied_instead_of_streaming_magic() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.dense-denied",
    );
    let family = dense_traversal_family(&mut workspace, "phase-nine-dense-denied");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("dense broad read review should be inspectable");
    let admission = review
        .graph_read_access_admission()
        .expect("dense broad admission evidence should exist");

    assert!(!admission.is_admitted());
    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::Denied
    );
    assert!(review.graph_read_access_plan().is_err());
}

#[test]
fn ordinary_inline_read_does_not_emit_streaming_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.inline-no-streaming",
    );
    let family = projection_only_family(&mut workspace, "phase-nine-inline-no-streaming");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("inline read should execute");

    assert!(result.receipt().graph_read_streaming_receipt().is_none());
    assert_eq!(
        result
            .receipt()
            .graph_read_access_plan()
            .expect("inline execution should attach access plan")
            .posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
}

fn seeded_streaming_receipt(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
) -> WorthQueryGraphReadStreamingReceipt {
    let mut workspace = workspace(request_execution, workspace_name);
    let identities = seed_active_users(request_execution, &mut workspace, workspace_name, 3);
    seed_frontier_edges(
        request_execution,
        &mut workspace,
        workspace_name,
        &identities,
    );
    let family = frontier_search_family(&mut workspace, workspace_name);
    workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("streaming frontier read should execute")
        .receipt()
        .graph_read_streaming_receipt()
        .expect("streaming execution should attach streaming receipt")
        .clone()
}

fn seed_active_users(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    prefix: &str,
    count: usize,
) -> Vec<crate::memory_workspace::WorthQueryEntityIdentity> {
    let mut identities = Vec::with_capacity(count);
    for index in 0..count {
        let receipt = workspace
            .insert(request_execution, "user", |user| {
                user.set_aspect(
                    touch("identity.id"),
                    authored_text(format!("{prefix}-{index}")),
                )
                .set_aspect(touch("status.value"), authored_text("active"))
                .set_aspect(
                    touch("profile.display_name"),
                    authored_text(format!("User {index}")),
                )
            })
            .expect("seed user should insert through runtime");
        identities.push(receipt.deltas()[0].entity_identity().clone());
    }
    identities
}

fn seed_frontier_edges(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    prefix: &str,
    identities: &[crate::memory_workspace::WorthQueryEntityIdentity],
) {
    for relation in ["manager", "mentor"] {
        for index in 0..2 {
            workspace
                .insert(request_execution, relation, |edge| {
                    edge.set_aspect(
                        touch("identity.id"),
                        authored_text(format!("{prefix}-{relation}-{index}")),
                    )
                    .set_aspect(touch("source.id"), authored_entity_ref(&identities[index]))
                    .set_aspect(
                        touch("target.id"),
                        authored_entity_ref(&identities[index + 1]),
                    )
                })
                .expect("seed frontier relation should insert through runtime");
        }
    }
}

fn authored_text(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn authored_entity_ref(
    identity: &crate::memory_workspace::WorthQueryEntityIdentity,
) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::native(
        crate::runtime::tests::support::test_native_entity_ref_value(identity),
    )
}
