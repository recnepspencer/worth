use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadMaterializationJobState,
    WorthQueryGraphReadMaterializationPolicy, WorthQueryGraphReadMaterializationRequest,
    WorthQueryRuntimeError,
};

use crate::runtime::tests::graph_read_access::support::graph_read_access::async_materialization::{
    async_materialization_workspace, async_required_graph_read_family,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_assertions::{
    assert_pre_execution_graph_access_denial, read_composition_denial,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_declarations::dense_over_budget_family;
use crate::runtime::tests::graph_read_access::support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn closeout_dense_broad_read_denies_before_executor_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("graph-read-access.closeout.memory.dense")
        .expect("closeout workspace should open");
    let family = dense_over_budget_family(&mut workspace, "closeout-dense-denial");
    let denial = read_composition_denial(
        workspace
            .execute_read_family(request_execution, &family)
            .expect_err("dense broad read should deny before execution"),
    );
    let admission = assert_pre_execution_graph_access_denial(&denial);
    let budget_denial = admission
        .denial()
        .and_then(|denial| denial.budget_exceeded())
        .expect("dense denial should carry budget envelope");

    assert!(budget_denial.estimated_index_bytes() > budget_denial.max_inline_index_bytes());
    assert_eq!(
        admission.denial().map(|denial| denial.suggested_posture()),
        Some(&WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired)
    );
}

#[test]
fn closeout_async_materialization_stops_when_observed_work_exceeds_declared_budget() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.closeout.memory.async-artifact",
    );
    let family = async_required_graph_read_family(&mut workspace, "closeout-async-artifact");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("async-required read should review")
        .graph_read_access_admission()
        .expect("async-required read should admit to required materialization");
    let policy = WorthQueryGraphReadMaterializationPolicy::bounded()
        .with_max_touched_edges(1)
        .with_max_resident_bytes(1);
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        policy.clone(),
    )
    .expect("required admission should derive materialization request");
    let estimated_resident_bytes = request.estimated_resident_bytes();
    let resource_limit = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("materialization request should admit")
        .start()
        .expect("materialization job should start")
        .stop_for_resource_limit()
        .expect("over-budget materialization must stop before completion");

    assert!(estimated_resident_bytes > policy.max_resident_bytes());
    assert_eq!(
        resource_limit.estimated_resident_bytes(),
        estimated_resident_bytes
    );
    assert_eq!(
        resource_limit.final_job_state(),
        &WorthQueryGraphReadMaterializationJobState::Indeterminate
    );
}

#[test]
fn closeout_async_materialization_artifact_reports_observed_memory_envelope() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.closeout.memory.async-artifact",
    );
    let family = async_required_graph_read_family(&mut workspace, "closeout-async-artifact");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("async-required read should review")
        .graph_read_access_admission()
        .expect("async-required read should admit to required materialization");
    let policy = WorthQueryGraphReadMaterializationPolicy::bounded()
        .with_max_touched_edges(1_000_000)
        .with_max_resident_bytes(1_000_000);
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        policy.clone(),
    )
    .expect("required admission should derive materialization request");
    let expected_resident_bytes = request.estimated_resident_bytes();
    let expected_touched_edges = request.estimated_touched_edges();
    let artifact = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("materialization request should admit")
        .start()
        .expect("materialization job should start")
        .complete_to_artifact();
    let receipt = artifact.receipt();

    assert_eq!(
        receipt.max_resident_bytes_observed(),
        expected_resident_bytes
    );
    assert_eq!(receipt.touched_edges(), expected_touched_edges);
    assert!(receipt.max_resident_bytes_observed() <= policy.max_resident_bytes());
    assert!(receipt.touched_edges() <= policy.max_touched_edges());
    assert_eq!(artifact.row_count(), receipt.emitted_rows());
    assert!(artifact
        .row_proofs()
        .iter()
        .all(|row| { row.materialization_digest() == artifact.materialization_digest() }));
}

#[test]
fn closeout_wrong_access_plan_denies_before_executor_counters_exist() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut source = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("graph-read-access.closeout.memory.wrong-plan-source")
        .expect("source workspace should open");
    let source_family = dense_over_budget_family(&mut source, "closeout-source-denied");
    let source_error = source
        .read_family_intent(&source_family)
        .review()
        .expect("source review should exist")
        .graph_read_access_plan()
        .expect_err("denied source should not produce a plan");

    assert!(matches!(
        source_error,
        WorthQueryRuntimeError::ReadCompositionDenied(_)
    ));
}
