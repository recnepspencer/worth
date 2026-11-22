use super::*;

mod connectivity_wave;
mod identity_survival;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_geometry_kernel_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    identity_survival::certify_topology_identity_survival_recovery_round_trip(execution);
    connectivity_wave::certify_topology_bridge_connectivity_wave(execution);
}
