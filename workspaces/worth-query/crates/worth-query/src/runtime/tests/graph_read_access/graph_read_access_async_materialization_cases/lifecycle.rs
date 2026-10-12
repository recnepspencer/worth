use crate::runtime::{
    WorthQueryGraphReadMaterializationJobState, WorthQueryGraphReadMaterializationPolicy,
    WorthQueryGraphReadMaterializationRequest,
};

use crate::runtime::tests::graph_read_access::support::graph_read_access::async_materialization::{
    async_materialization_workspace, async_required_graph_read_family,
};

#[test]
fn materialization_job_exposes_progress_and_replay_stable_completion_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.complete",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-complete");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();
    let policy = WorthQueryGraphReadMaterializationPolicy::bounded()
        .with_max_touched_edges(1_000_000)
        .with_max_resident_bytes(1_000_000);
    let request =
        WorthQueryGraphReadMaterializationRequest::from_required_admission(&admission, policy)
            .expect("request should derive");
    let expected_touched_edges = request.estimated_touched_edges();
    let expected_allocated_bytes = request.estimated_resident_bytes();
    let repeated_request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded()
            .with_max_touched_edges(1_000_000)
            .with_max_resident_bytes(1_000_000),
    )
    .expect("same request should derive");

    let job = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start");
    assert_eq!(job.progress().checkpoint_count(), 0);
    let receipt = job.complete();
    let repeated_receipt = workspace
        .graph_read_materializations()
        .admit(request_execution, repeated_request)
        .expect("repeated request should admit")
        .start()
        .expect("repeated job should start")
        .complete();
    let divergent_policy_receipt = workspace
        .graph_read_materializations()
        .admit(
            request_execution,
            WorthQueryGraphReadMaterializationRequest::from_required_admission(
                &admission,
                WorthQueryGraphReadMaterializationPolicy::bounded()
                    .with_max_touched_edges(2_000_000)
                    .with_max_resident_bytes(1_000_000),
            )
            .expect("divergent policy request should derive"),
        )
        .expect("divergent request should admit")
        .start()
        .expect("divergent job should start")
        .complete();

    assert_eq!(receipt.touched_edges(), expected_touched_edges);
    assert_eq!(
        receipt.max_resident_bytes_observed(),
        expected_allocated_bytes
    );
    assert_eq!(
        receipt.emitted_rows(),
        admission
            .cost_estimate()
            .intrinsic()
            .intermediate_set_size()
    );
    assert_eq!(receipt.admission_digest(), admission.digest());
    assert_eq!(
        receipt.materialization_digest(),
        repeated_receipt.materialization_digest()
    );
    assert_ne!(
        receipt.materialization_digest(),
        divergent_policy_receipt.materialization_digest()
    );
}

#[test]
fn completed_materialization_exposes_artifact_rows_bound_to_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.artifact",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-artifact");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded()
            .with_max_touched_edges(1_000_000)
            .with_max_resident_bytes(1_000_000),
    )
    .expect("request should derive");
    let artifact = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start")
        .complete_to_artifact();
    let receipt = artifact.receipt();

    assert_eq!(artifact.request_digest(), receipt.request_digest());
    assert_eq!(
        artifact.materialization_digest(),
        receipt.materialization_digest()
    );
    assert_eq!(artifact.row_count(), receipt.emitted_rows());
    assert_eq!(
        artifact.final_checkpoint_digest(),
        receipt.final_checkpoint_digest()
    );
    assert!(artifact.row_count() > 0);
    for (ordinal, row) in artifact.row_proofs().iter().enumerate() {
        assert_eq!(row.row_ordinal(), ordinal);
        assert_eq!(
            row.materialization_digest(),
            artifact.materialization_digest()
        );
        assert!(!row.digest().is_empty());
    }
}

#[test]
fn cancelling_materialization_releases_allocated_frontier_resources() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.cancel",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-cancel");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded().with_cancellation_scope("operator:17"),
    )
    .expect("request should derive");
    let mut job = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start");
    let checkpoint = job.advance_to_next_checkpoint();
    let progress = job.progress().clone();
    let cancellation = job.cancel_after_checkpoint();

    assert_eq!(cancellation.request_digest(), progress.request_digest());
    assert_eq!(cancellation.progress_digest(), progress.digest());
    assert_eq!(cancellation.last_checkpoint_digest(), checkpoint.digest());
    assert_eq!(
        cancellation.released_frontier_pages(),
        progress.frontier_pages()
    );
    assert_eq!(
        cancellation.released_allocated_bytes(),
        progress.allocated_bytes()
    );
    assert_eq!(cancellation.cancellation_scope(), "operator:17");
    assert_eq!(
        cancellation.cancellation_poll_count(),
        progress.cancellation_poll_count()
    );
    assert_eq!(
        cancellation.final_job_state(),
        &WorthQueryGraphReadMaterializationJobState::Cancelled
    );
}

#[test]
fn indeterminate_materialization_exposes_recovery_handle_from_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.recovery",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-recovery");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded(),
    )
    .expect("request should derive");
    let mut job = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start");
    let checkpoint = job.advance_to_next_checkpoint();
    let progress = job.progress().clone();
    let recovery = job.stop_indeterminate_after_checkpoint("checkpoint-observation-missing");

    assert_eq!(recovery.request_digest(), checkpoint.request_digest());
    assert_eq!(recovery.last_checkpoint_digest(), checkpoint.digest());
    assert_eq!(recovery.progress_digest(), progress.digest());
    assert_eq!(recovery.recovery_reason(), "checkpoint-observation-missing");
}

#[test]
fn materialization_resource_limit_stop_is_typed_and_checkpoint_bound() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.resource-limit",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-resource-limit");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();
    let request = WorthQueryGraphReadMaterializationRequest::from_required_admission(
        &admission,
        WorthQueryGraphReadMaterializationPolicy::bounded().with_max_resident_bytes(1),
    )
    .expect("request should derive");
    let estimated_resident_bytes = request.estimated_resident_bytes();
    let limit = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start")
        .stop_for_resource_limit()
        .expect("resident estimate above cap should stop with limit receipt");

    assert!(estimated_resident_bytes > limit.max_resident_bytes());
    assert_eq!(limit.estimated_resident_bytes(), estimated_resident_bytes);
    assert_eq!(
        limit.final_job_state(),
        &WorthQueryGraphReadMaterializationJobState::Indeterminate
    );
    assert!(!limit.last_checkpoint_digest().is_empty());
    assert!(!limit.progress_digest().is_empty());
}
