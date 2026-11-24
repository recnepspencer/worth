//! Both original lease tasks survive contention and cancellation.
use super::*;
use worth_relational::facade::{
    change_source::RelationalRetainedObservationRetirementProgress as Progress,
    durability::RelationalRecoveredPublicationRetirement,
};

#[test]
fn recorded_original_retirement_preserves_both_leases_across_busy_and_stop() {
    let stop = worth_execution::CancellationSource::new();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request()
        .with_cancellation(stop.token());
    let request = &ExecutionRequest::serial(&host);
    let (runtime, proof) = readmit_original(request, false);
    let basis = proof.custody().predecessor().clone();
    let before = runtime.external_retention_terminal_counts_for_test();
    let mut task = proof.consume(|original| original.into_retirement());
    assert_recovered_cut_and_lane_busy(&runtime, &mut task, &basis, request);
    drop(basis);
    stop.cancel();
    assert!(matches!(task.advance(request), Progress::Stopped(_)));
    let terminal = runtime.external_retention_terminal_counts_for_test();
    assert_eq!(terminal.explicit_releases, before.explicit_releases + 1);
    assert_eq!(terminal.dropped_releases, before.dropped_releases);
    let mut task = task.into_retired().unwrap_err();
    let retry = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let retry = ExecutionRequest::serial(&retry);
    let retired = loop {
        match task.advance(&retry) {
            Progress::Advanced => continue,
            Progress::Complete => break task.into_retired().unwrap(),
            other => panic!("original recovered custody retirement: {other:?}"),
        }
    };
    let terminal = runtime.external_retention_terminal_counts_for_test();
    assert_eq!(terminal.explicit_releases, before.explicit_releases + 2);
    assert_eq!(terminal.dropped_releases, before.dropped_releases);
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
}

/// The aggregate must preserve its two original lease tasks on either gate.
fn assert_recovered_cut_and_lane_busy(
    runtime: &RelationalRuntime,
    task: &mut RelationalRecoveredPublicationRetirement,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let [cut, lane] = runtime
        .recovered_retirement_busy_trials_for_test(task, basis, request)
        .unwrap();
    assert_eq!(cut.progress, Progress::Busy);
    assert_eq!(cut.after.counters, cut.before.counters);
    assert_eq!(lane.progress, Progress::Busy);
    assert!(lane.before.root_obligations.is_some());
    assert_eq!(lane.after.lane_obligations, lane.before.lane_obligations);
    assert_eq!(lane.after.root_obligations, lane.before.root_obligations);
    assert_eq!(task.advance(request), Progress::Advanced);
    assert_eq!(
        runtime.recovered_retirement_lane_releases_for_test(basis),
        Some(cut.before.counters.external_pin_releases + 1)
    );
}
