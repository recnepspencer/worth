use crate::runtime::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadCheckpointInterval,
    WorthQueryGraphReadMaterializationPolicy, WorthQueryGraphReadMaterializationRequest,
    WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support::graph_read_access::async_materialization::{
    async_materialization_workspace, async_required_graph_read_family,
};

#[test]
fn materialization_checkpoint_intervals_emit_exact_sequences() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = async_materialization_workspace(
        request_execution,
        "graph-read-access.async-materialization.checkpoints",
    );
    let family = async_required_graph_read_family(&mut workspace, "async-checkpoints");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("large read should be reviewable")
        .graph_read_access_admission()
        .expect("graph read admission should exist")
        .clone();

    assert_checkpoint_interval_sequence(request_execution, &mut workspace, &admission, 1);
    assert_checkpoint_interval_sequence(request_execution, &mut workspace, &admission, 2);
    assert_checkpoint_interval_sequence(request_execution, &mut workspace, &admission, 256);
}

fn assert_checkpoint_interval_sequence(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    admission: &WorthQueryGraphReadAccessAdmission,
    checkpoint_interval: usize,
) {
    let policy = WorthQueryGraphReadMaterializationPolicy::bounded().with_checkpoint_interval(
        WorthQueryGraphReadCheckpointInterval::frontier_pages(checkpoint_interval),
    );
    let request =
        WorthQueryGraphReadMaterializationRequest::from_required_admission(admission, policy)
            .expect("request should derive");
    let expected_checkpoint_count =
        expected_checkpoint_count_for_request(&request, checkpoint_interval.max(1));
    let mut job = workspace
        .graph_read_materializations()
        .admit(request_execution, request)
        .expect("request should admit")
        .start()
        .expect("job should start");
    let mut checkpoint = job.checkpoint().clone();
    while checkpoint.sequence() < expected_checkpoint_count {
        checkpoint = job.advance_to_next_checkpoint();
    }

    assert_eq!(checkpoint.sequence(), expected_checkpoint_count);
    assert_eq!(job.checkpoints().len(), expected_checkpoint_count + 1);
    assert_eq!(job.complete().checkpoint_count(), expected_checkpoint_count);
}

fn expected_checkpoint_count_for_request(
    request: &WorthQueryGraphReadMaterializationRequest,
    checkpoint_interval: usize,
) -> usize {
    request
        .estimated_touched_edges()
        .div_ceil(64)
        .max(1)
        .div_ceil(checkpoint_interval)
        .max(1)
}
