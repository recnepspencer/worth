//! R4 window (PR C plan line 401): an earlier plain support failure stays
//! visible despite a later valid supersession. Final support checks a
//! superseded read only over its window, from its own image to the image of
//! the last round before its recorded invalidator, a chain's superseded
//! rounds included. Since C4 qualified use compares task-owned selections and
//! claimed retentions at every append, so this check is reached only as
//! defense in depth; these courts drive its verdict directly.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::coupled::support::superseded_window_refuses_for_test as refuses;

/// A read at 1, superseded by the valid invalidator at 3. The plain change
/// the round at 2 made (a chain round later superseded by 5) lies inside
/// the window and stays visible: the edit refuses.
#[test]
fn an_earlier_plain_change_inside_the_window_refuses_despite_a_valid_supersession() {
    assert!(refuses(1, 3, &[2, 3, 5], 2));
}

/// The invalidator's own change is authenticated by the exact comparison
/// that superseded the read: the window ends before it, so it passes.
#[test]
fn the_invalidators_own_change_lies_outside_the_window() {
    assert!(!refuses(1, 3, &[2, 3, 5], 3));
}

/// The replacement round after the invalidator is checked on its own, never
/// through the superseded read's window.
#[test]
fn a_change_after_the_invalidator_lies_outside_the_window() {
    assert!(!refuses(1, 3, &[2, 3, 5], 5));
}

/// With no round between the read and its invalidator the window is the
/// read's own image: nothing inside it can change.
#[test]
fn an_empty_window_compares_the_read_with_itself() {
    assert!(!refuses(1, 2, &[2, 4], 2));
}
