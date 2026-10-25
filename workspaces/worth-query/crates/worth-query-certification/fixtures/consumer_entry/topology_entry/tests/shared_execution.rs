//! Observe the ordinary consumer on the same model thread that advances it.
#[path = "../examples/shared_execution.rs"]
mod shared;

#[test]
fn installed_roots_and_reopen_record_real_unaccounted_owner_work() {
    use worth_query_host::facade::primary_graph::{
        advancement_observations_on_this_thread_for_test, WorthQueryExecutionObservation,
    };
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    shared::run();
    let observations = advancement_observations_on_this_thread_for_test();
    assert!(
        !observations.is_empty(),
        "the model thread must record actual advancement"
    );
    assert!(observations.iter().all(|entry| entry.refusal.is_none()));
    assert!(observations
        .iter()
        .all(|entry| entry.report.accounted_cost().is_none()));
    assert!(observations.iter().any(|entry| {
        entry.workers.is_some() && entry.report.physical().active_workers_high_watermark() >= 1
    }));
    assert!(
        observations.iter().any(|entry| !entry.applied.is_empty()),
        "a physical report alone cannot stand in for actual owner application"
    );
}
