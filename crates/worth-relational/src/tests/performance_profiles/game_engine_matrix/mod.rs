use super::*;

mod flat_batch_wave;
mod local_scene_wave;
mod mixed_frame_churn;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_game_engine_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "game_engine_matrix";

    local_scene_wave::certify_local_scene_graph_propagation_wave(execution, suite);
    flat_batch_wave::certify_flat_entity_batch_region_wave(execution, suite);
    mixed_frame_churn::certify_mixed_read_write_frame_churn_window(execution, suite);
}
