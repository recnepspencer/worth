use super::*;

mod branch_rollback_compile;
mod checkpoint_recover_compile;
mod dense_fanout_compile;
mod event_wave_churn;
mod event_wave_rich_diagnostics;
mod flat_entity_step_batch;
mod rich_fanout_diagnostics;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_chip_simulator_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "chip_simulator_matrix";

    dense_fanout_compile::certify_dense_fanout_compile_wave(execution, suite);
    rich_fanout_diagnostics::certify_dense_fanout_compile_wave_rich_diagnostics(execution, suite);
    checkpoint_recover_compile::certify_checkpoint_window_recover_compile_round_trip(
        execution, suite,
    );
    branch_rollback_compile::certify_branch_rollback_compile_step_window(execution, suite);
    flat_entity_step_batch::certify_flat_entity_step_batch_compile_window(execution, suite);
    event_wave_churn::certify_event_wave_compile_churn_window(execution, suite);
    event_wave_rich_diagnostics::certify_event_wave_compile_churn_rich_diagnostics(
        execution, suite,
    );
}
