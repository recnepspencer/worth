//! Every attempt contributes its work, including refusal, retry, and stop.

use super::*;

#[test]
fn successful_edits_failed_attempt_and_rebuild_are_all_reported() {
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    let first_depth = shape::Shape::depth(&shape::Shape::from_sorted(&keys), keys[0]);
    // First edit completes, then the second attempt enters a combine and
    // panics. The one-shot fault permits the fallback build to complete.
    let failed_entry = 2 * first_depth + 1;
    CALLS.set(0);
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
        next.report.metrics().combine_calls,
        u128::from(failed_entry + 64)
    );
    assert_eq!(
        next.report.metrics().recombined_nodes,
        u128::from(first_depth + 32)
    );
    assert_eq!(*next.outcome.unwrap().result(), 35);
}

#[test]
fn edit_memory_refusal_keeps_prior_work_and_releases_transient_hold_before_rebuild() {
    use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
    use std::num::NonZeroUsize;
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    let declared = 4096;
    let results_bytes = u64::try_from(keys.len() * size_of::<(Id, u64)>()).unwrap();
    let full_hold =
        Tree::checked_build_memory_bound(keys.len(), declared).unwrap() + 2 * results_bytes;
    // Reserve exactly the full build. A dense edit batch accumulates path
    // scratch beyond that bound, then must release it before rebuilding.
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            NonZeroUsize::new(1).unwrap(),
            full_hold,
        )),
        &request,
    );
    let mut memory = execution.reserve(results_bytes).unwrap();
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, counted);
    let plan = plan(&keys);
    let work = plan.checked_build_work();
    CALLS.set(0);
    let next = next_tree::<_, _, _, u32>(
        &retained,
        plan,
        work,
        keys.iter().map(|id| (*id, 2)).collect(),
        u64::MAX,
        declared,
        &reducer,
        &execution,
        &mut memory,
    );
    reconcile(next.report);
    assert!(matches!(
        next.report,
        TreeRun::Rebuilt(Rebuild::EditMemory, _)
    ));
    assert!(
        next.report.metrics().recombined_nodes > 32,
        "some edits ran before rebuilding"
    );
    assert_eq!(
        next.report.metrics().combine_calls,
        2 * next.report.metrics().recombined_nodes
    );
    assert_eq!(*next.outcome.unwrap().result(), 64);
}

#[test]
fn a_failed_edit_followed_by_a_stopped_rebuild_preserves_both_attempts() {
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    let mut retained = retained;
    retained.tree = Tree::try_from_declared(
        plan(&keys),
        keys.iter().map(|id| (*id, 1)).collect(),
        0,
        phase_panicking as fn(&u64, &u64) -> u64,
    )
    .unwrap()
    .0;
    CALLS.set(0);
    PANIC_AT.set(Some(1));
    // A reducer panic in each pass: the edit has one entered
    // combine and no completed node; the rebuilt tree then panics on its
    // first combine too. Neither terminal path may discard its work.
    let request = live_scope();
    let execution = QueryRequestExecution::open(serial_placement(), &request);
    let mut memory = execution
        .reserve(u64::try_from(size_of::<(Id, u64)>()).unwrap())
        .unwrap();
    let reducer = WorthQueryDeterministicReducer::canonical(|| 0, phase_panicking);
    let plan = plan(&keys);
    let work = plan.checked_build_work();
    let next = next_tree::<_, _, _, u32>(
        &retained,
        plan,
        work,
        BTreeMap::from([(keys[0], 2)]),
        u64::MAX,
        4096,
        &reducer,
        &execution,
        &mut memory,
    );
    reconcile(next.report);
    assert!(matches!(
        next.report,
        TreeRun::Rebuilt(Rebuild::ReducerPanicked, _)
    ));
    assert!(next.outcome.is_err());
    assert_eq!(next.report.metrics().combine_calls, 2);
    assert_eq!(next.report.metrics().recombined_nodes, 0);
    PANIC_AT.set(None);
}

fn phase_panicking(left: &u64, right: &u64) -> u64 {
    CALLS.set(CALLS.get() + 1);
    if PANIC_AT.get().is_some() {
        panic!("rebuild combine refused");
    }
    left + right
}

#[test]
fn a_work_stopped_rebuild_reports_the_work_it_entered() {
    let template = template();
    let keys: Vec<_> = (0..32).map(Id::new).collect();
    let retained = retained(&template, &keys);
    CALLS.set(0);
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
    assert!(next.report.metrics().combine_calls > 0);
}
