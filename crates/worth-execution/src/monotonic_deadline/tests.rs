use super::*;
use crate::{MonotonicTimeAdvanceDenial, MonotonicTimeController};

#[test]
fn repeated_capture_keeps_one_constraint_per_clock() {
    let parent = MonotonicTimeController::new().unwrap();
    let child = MonotonicTimeController::new().unwrap();
    let parent_now = parent.hold();
    let child_now = child.hold();
    let parent_deadline = parent
        .clock()
        .deadline_at(parent_now + Duration::from_secs(20));
    let child_deadline = child
        .clock()
        .deadline_at(child_now + Duration::from_secs(10));
    let mut retained = parent_deadline.clone().meet(child_deadline.clone());
    for _ in 0..1_000 {
        retained = retained
            .meet(parent_deadline.clone())
            .meet(child_deadline.clone());
    }
    assert_eq!(retained.constraints().len(), 2);
    assert!(!retained.is_elapsed());
    parent.advance(Duration::from_secs(21)).unwrap();
    assert!(!child_deadline.is_elapsed());
    assert!(retained.is_elapsed());
}

#[test]
fn a_child_clock_deadline_does_not_depend_on_its_parent_clock() {
    let parent = MonotonicTimeController::new().unwrap();
    let child = MonotonicTimeController::new().unwrap();
    let parent_now = parent.hold();
    let child_now = child.hold();
    let parent_deadline = parent
        .clock()
        .deadline_at(parent_now + Duration::from_secs(20));
    let child_deadline = child
        .clock()
        .deadline_at(child_now + Duration::from_secs(10));
    let retained = parent_deadline.clone().meet(child_deadline);
    child.advance(Duration::from_secs(11)).unwrap();
    assert!(!parent_deadline.is_elapsed());
    assert!(retained.is_elapsed());
    assert_eq!(retained.instant_on(&parent.clock()), None);
}

#[test]
fn holding_is_idempotent_and_failed_advance_preserves_the_sample() {
    let time = MonotonicTimeController::new().unwrap();
    assert_eq!(
        time.advance(Duration::ZERO),
        Err(MonotonicTimeAdvanceDenial::NotHeld)
    );
    let held = time.hold();
    assert_eq!(time.hold(), held);
    assert_eq!(
        time.advance(Duration::MAX),
        Err(MonotonicTimeAdvanceDenial::InstantOverflow)
    );
    assert_eq!(time.clock().now(), held);
    assert_eq!(
        time.advance(Duration::from_secs(1)).unwrap(),
        held + Duration::from_secs(1)
    );
}
