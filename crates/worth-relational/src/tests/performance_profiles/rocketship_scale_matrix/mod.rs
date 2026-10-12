use super::*;

mod flat_batch_wave;
mod larger_flat_batch_wave;
mod mixed_entity_relation_batch_wave;
mod pseudorealistic_narrow_round_trip;
mod pseudorealistic_propagation_wave;
mod rich_geometry_round_trip;
mod rich_propagation_wave;
mod varied_flat_batch_wave;
mod zero_diagnostics_round_trip;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_rocketship_scale_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "rocketship_scale_matrix";
    let node_count = rocketship_node_count();
    let query_target_count = rocketship_query_target_count(node_count);

    zero_diagnostics_round_trip::certify_hundred_k_nodes_zero_diagnostics_narrow_round_trip(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    rich_geometry_round_trip::certify_hundred_k_nodes_geometry_profile_narrow_round_trip(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    pseudorealistic_narrow_round_trip::certify_hundred_k_nodes_pseudorealistic_narrow_round_trip(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    pseudorealistic_propagation_wave::certify_hundred_k_nodes_pseudorealistic_propagation_wave(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    flat_batch_wave::certify_hundred_k_nodes_pseudorealistic_flat_batch_wave(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    varied_flat_batch_wave::certify_hundred_k_nodes_pseudorealistic_varied_flat_batch_wave(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    larger_flat_batch_wave::certify_hundred_k_nodes_pseudorealistic_larger_flat_batch_wave(
        execution,
        suite,
        node_count,
        query_target_count,
    );
    mixed_entity_relation_batch_wave::certify_hundred_k_nodes_pseudorealistic_mixed_entity_relation_batch_wave(execution, suite, node_count, query_target_count);
    rich_propagation_wave::certify_hundred_k_nodes_geometry_profile_propagation_wave(
        execution,
        suite,
        node_count,
        query_target_count,
    );
}
