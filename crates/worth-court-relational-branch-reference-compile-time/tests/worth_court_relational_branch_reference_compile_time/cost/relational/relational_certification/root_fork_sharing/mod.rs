#[allow(unused_imports)]
pub(crate) use {crate::relational_certification::ordinary_context::root_fork_sharing::fitted_slope, crate::relational_certification::ordinary_context::root_fork_sharing::standard_fanout_samples};

#[test]
#[ignore = "scheduled maximum-scale fork slope"]
fn phase5_standard_fork_copy_slope_is_flat_through_4096_forks() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let samples = standard_fanout_samples(execution, &[1, 64, 4_096]);
    assert_eq!(
        samples
            .iter()
            .map(|sample| sample.fork_materialized_authoritative_bytes)
            .collect::<Vec<_>>(),
        vec![0; 3],
        "the exact zero-copy counter remains flat at every declared fan-out"
    );
    assert_eq!(
        fitted_slope(&samples),
        0.0,
        "least-squares fork-copy slope over log2 fan-out remains exactly flat"
    );
}
