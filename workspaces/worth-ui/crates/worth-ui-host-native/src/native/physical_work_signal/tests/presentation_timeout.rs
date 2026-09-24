use super::super::{
    UiNativePhysicalPresentationBasis, UiNativePhysicalSignalOwner,
    UiNativePhysicalSignalSettlement, UiNativePhysicalSignalStatus,
};

#[test]
fn timed_out_presentation_recovery_hands_off_to_the_retained_host_owner() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let identity = owner
        .admit_presentation(UiNativePhysicalPresentationBasis::test())
        .expect("presentation admission");
    let retained = owner
        .take_initial_presentation(identity)
        .expect("the host retains the first physical token");
    assert_eq!(
        owner.reconcile(retained.observe(UiNativePhysicalSignalStatus::Pending)),
        UiNativePhysicalSignalSettlement::Pending
    );

    for _ in 0..80 {
        if owner.observation().counters.recovery_schedules != 0 {
            break;
        }
        let due = owner
            .next_due_tick()
            .expect("bounded work has a due transition");
        owner.advance_clock_to(due).expect("advance physical clock");
    }

    assert_eq!(owner.observation().counters.recovery_schedules, 1);
    assert_eq!(owner.observation().pending_wakes, 1);
    let successor = owner
        .take_ready_presentation(identity, retained)
        .expect("the original host token owns its timed-out recovery successor")
        .current();
    assert_ne!(successor, retained);
    assert!(owner.token_uses_recovery(successor));
    assert_eq!(owner.observation().pending_wakes, 0);
    assert_eq!(
        owner.reconcile(successor.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
}
