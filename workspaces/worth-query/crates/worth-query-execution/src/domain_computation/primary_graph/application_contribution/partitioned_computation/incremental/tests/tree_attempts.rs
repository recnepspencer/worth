//! Every attempt contributes its work, including refusal, retry, and stop.

use super::*;

#[test]
fn successful_edits_failed_attempt_and_rebuild_are_all_reported() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    let first_depth = shape::Shape::depth(&shape::Shape::from_sorted(&keys), keys[0]);
    // First edit completes, then the second attempt enters a combine and
    // panics. The one-shot fault permits the fallback build to complete.
    let failed_entry = 2 * first_depth + 1;
    reset_counts();
    PANIC_AT.set(Some(failed_entry));
    let next = run(
        &retained,
        &keys,
        BTreeMap::from([(keys[0], 2), (keys[1], 3)]),
        u64::MAX,
    );
    reconcile(next.report);
    assert!(matches!(
        next.report,
        TreeRun::Rebuilt(Rebuild::ReducerPanicked, _)
    ));
    assert_eq!(
        next.report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls,
        u128::from(failed_entry + 64)
    );
    assert_eq!(
        next.report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .recombined_nodes,
        u128::from(first_depth + 32)
    );
    assert_eq!(*next.outcome.unwrap().result(), 35);
}

#[test]
fn a_failed_edit_followed_by_a_stopped_rebuild_preserves_both_attempts() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    let mut retained = retained;
    let request = live_scope();
    let execution = QueryRequestExecution::open(serial_placement(), &request);
    retained.tree = ReductionTree::<u64, fn(&u64, &u64) -> u64>::try_from_declared_native(
        plan(&keys),
        keys.iter().map(|id| (*id, 1)).collect(),
        0,
        phase_panicking as fn(&u64, &u64) -> u64,
        Some(execution.dispatch().unwrap().native_lease_for_test()),
        || execution.checkpoint(),
    )
    .unwrap()
    .0;
    reset_counts();
    PANIC_AT.set(Some(1));
    // A reducer panic in each pass: the edit has one entered
    // combine and no completed node; the rebuilt tree then panics on its
    // first combine too. Neither terminal path may discard its work.
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, phase_panicking);
    let plan = plan(&keys);
    let work = plan.checked_build_work();
    let next = next_tree::<_, _, _, u32>(
        &retained,
        plan,
        work,
        BTreeMap::from([(keys[0], 2)]),
        None,
        &reducer,
        &execution,
    );
    reconcile(next.report);
    assert!(matches!(
        next.report,
        TreeRun::Rebuilt(Rebuild::ReducerPanicked, _)
    ));
    assert!(next.outcome.is_err());
    assert_eq!(
        next.report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls,
        2
    );
    assert_eq!(
        next.report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .recombined_nodes,
        0
    );
    PANIC_AT.set(None);
}

fn phase_panicking(left: &u64, right: &u64) -> u64 {
    CALLS.set(CALLS.get() + 1);
    if PANIC_AT.get().is_some() {
        PAIR.set(false);
        panic!("rebuild combine refused");
    }
    let value = left + right;
    if PAIR.replace(!PAIR.get()) {
        NODES.set(NODES.get() + 1);
    }
    value
}

#[test]
fn a_work_stopped_rebuild_reports_the_work_it_entered() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    reset_counts();
    let next = run(
        &retained,
        &keys,
        BTreeMap::from([(keys[0], 2)]),
        plan(&keys).checked_build_work().unwrap() - 1,
    );
    reconcile(next.report);
    assert!(matches!(
        next.report,
        TreeRun::Rebuilt(Rebuild::WorkCeiling, _)
    ));
    assert!(next.outcome.is_err());
    assert!(
        next.report
            .metrics_for_test()
            .accounted_metrics()
            .expect("explicit accounted fixture")
            .combine_calls
            > 0
    );
}
