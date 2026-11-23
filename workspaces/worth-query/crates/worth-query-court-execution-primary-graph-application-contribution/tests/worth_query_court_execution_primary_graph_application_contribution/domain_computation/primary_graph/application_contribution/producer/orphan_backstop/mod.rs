//! The orphan backstop (PR C C4 r5, plan §1.6): at the final drive's
//! quiescence a task that derived in this edit, done or reopened since, must
//! still be reached from a root through current prerequisites; one that is
//! not refuses `SupersededSelectionDropped`. These drive its walk over
//! hand-built task graphs (indices are canonical tasks).
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::first_orphan_for_test as orphan;

/// Root 0 closes on 1, which closes on 2: every derived task is reached.
#[test]
fn a_derived_task_a_root_still_reaches_is_no_orphan() {
    let prerequisites = vec![vec![1], vec![2], vec![]];
    assert_eq!(orphan(&[0], &[true, true, true], &prerequisites), None);
}

/// Root 0 re-closed on 2 alone: task 1, derived (done), is unreached.
#[test]
fn a_done_task_no_root_reaches_is_an_orphan() {
    let prerequisites = vec![vec![2], vec![], vec![]];
    assert_eq!(orphan(&[0], &[true, true, true], &prerequisites), Some(1));
}

/// A reopened task (derived, being derived again) that no root selects is an
/// orphan: being seeded to derive again does not make it selected.
#[test]
fn a_reopened_task_no_root_reaches_is_an_orphan() {
    let prerequisites = vec![vec![], vec![]];
    assert_eq!(orphan(&[0], &[false, true], &prerequisites), Some(1));
}

/// Reachability passes through a never-derived intermediate: root 0 still
/// closes on 1 (not derived), which closes on derived 2.
#[test]
fn reachability_passes_through_an_underived_intermediate() {
    let prerequisites = vec![vec![1], vec![2], vec![]];
    assert_eq!(orphan(&[0], &[false, false, true], &prerequisites), None);
}

/// An unreached task that never derived is not an orphan, and a cycle
/// among prerequisites terminates.
#[test]
fn an_unreached_underived_task_and_a_cycle_are_fine() {
    let prerequisites = vec![vec![1], vec![0], vec![]];
    assert_eq!(orphan(&[0], &[true, true, false], &prerequisites), None);
}
