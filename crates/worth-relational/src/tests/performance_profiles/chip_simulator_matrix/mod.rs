use super::*;

mod branch_rollback;
mod checkpoint_recovery;
mod dense_fanout;
mod event_wave_churn;
mod event_wave_rich_diagnostics;
mod flat_entity_step_batch;
mod rich_fanout_diagnostics;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_chip_simulator_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    dense_fanout::certify_dense_fanout_wave(execution);
    rich_fanout_diagnostics::certify_dense_fanout_wave_rich_diagnostics(execution);
    checkpoint_recovery::certify_checkpoint_recovery_round_trip(execution);
    branch_rollback::certify_branch_rollback_step_window(execution);
    flat_entity_step_batch::certify_flat_entity_step_batch(execution);
    event_wave_churn::certify_event_wave_churn_window(execution);
    event_wave_rich_diagnostics::certify_event_wave_churn_rich_diagnostics(execution);
}
