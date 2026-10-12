//! Placement arms restore only their live owner.
use super::*;
use crate::domain_computation::execution_observation::WorthQueryExecutionObservation as Observation;
#[test]
fn unwinding_clears_placement_and_budget_before_the_thread_is_reused() {
    assert!(std::panic::catch_unwind(|| {
        let _guard = Observation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Serial,
        );
        bound_advancement_requests_on_this_thread_for_test(Some(ExecutionBudget::new(
            NonZeroUsize::MIN,
            7,
            9,
        )));
        panic!("exercise placement cleanup");
    })
    .is_err());
    assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
    assert!(REQUEST_BUDGET.get().is_none());
}
#[test]
fn dropping_an_older_guard_cannot_clear_a_newer_guards_placement() {
    let outer = Observation::install_on_this_thread();
    place_managed_computations_on_this_thread_for_test(WorthQueryExecutionPlacementForTest::Serial);
    let inner = Observation::install_on_this_thread();
    let leased = WorthQueryExecutionPlacementForTest::Leased(NonZeroUsize::MIN);
    place_managed_computations_on_this_thread_for_test(leased);
    drop(outer);
    assert_eq!(PLACEMENT.get(), leased);
    drop(inner);
    assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
}
#[test]
fn an_inner_guard_restores_only_a_still_owned_outer_placement() {
    let outer = Observation::install_on_this_thread();
    place_managed_computations_on_this_thread_for_test(WorthQueryExecutionPlacementForTest::Serial);
    let inner = Observation::install_on_this_thread();
    place_managed_computations_on_this_thread_for_test(
        WorthQueryExecutionPlacementForTest::Leased(NonZeroUsize::MIN),
    );
    drop(inner);
    assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::Serial);
    drop(outer);
    assert_eq!(PLACEMENT.get(), WorthQueryExecutionPlacementForTest::World);
}
