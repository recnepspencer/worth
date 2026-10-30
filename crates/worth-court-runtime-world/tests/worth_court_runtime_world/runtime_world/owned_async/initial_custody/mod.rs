use crate::owned_async::ordinary_context::native_service::NativeServiceCourt;
use worth_signal::facade::branch::SignalOwnedAsyncManagedReport as Report;
use {
    crate::owned_async::ordinary_context::fixture::completion,
    crate::owned_async::ordinary_context::fixture::selected_fork,
    crate::owned_async::ordinary_context::fixture::AsyncSourceCourt,
};

#[test]
fn unqueued_initial_supersession_preserves_witness_and_exact_old_cleanup() {
    let court = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::new();
    let selected = selected_fork(&court);
    let source = court.lease(&selected);
    let mut prior = court
        .world
        .admit_unqueued_owned_async_request(&court.bridge, &court.declaration, &selected, &source)
        .unwrap();
    assert!(prior.refusal().is_none());
    let original = prior.retained_request().unwrap();
    let original_handle = original.request().request_handle();
    let (before, frontier) = court
        .bridge
        .owned_async_wake_frontier_for_test(&original)
        .unwrap();
    assert_eq!(before.scheduled_count(), 1);
    assert_eq!(before.ready_count(), 0);
    let old_wake = frontier.next_due_wake_id().unwrap();
    let successor_wake = before.next_wake_id();
    let mut successor = court
        .world
        .admit_unqueued_owned_async_request(&court.bridge, &court.declaration, &selected, &source)
        .unwrap();
    assert!(successor.refusal().is_none());
    assert!(successor.admitted_current_occurrence());
    let replacement = successor.retained_request().unwrap();
    assert_ne!(successor.request_handle(), original_handle);
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Admission(report) =
        successor.native_report()
    else {
        panic!("actual initial report")
    };
    assert_eq!(report.superseded_request().unwrap(), original_handle);
    let (after, successor_frontier) = court
        .bridge
        .owned_async_wake_frontier_for_test(&replacement)
        .unwrap();
    assert_eq!(
        after.scheduled_count(),
        1,
        "prior timeout was retired before this handoff"
    );
    assert_eq!(after.ready_count(), 0);
    assert_eq!(after.retired_count(), before.retired_count() + 1);
    assert_eq!(after.next_wake_id().get(), successor_wake.get() + 1);
    assert_eq!(successor_frontier.next_due_wake_id(), Some(successor_wake));
    assert_ne!(Some(old_wake), successor_frontier.next_due_wake_id());

    // Old entry cleanup must neither select nor retire the successor's timeout.
    court
        .bridge
        .finish_unqueued_owned_async_entry_cleanup(&mut prior)
        .unwrap();
    court
        .bridge
        .finish_unqueued_owned_async_entry_cleanup(&mut prior)
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_wake_frontier_for_test(&replacement)
            .unwrap(),
        (after, successor_frontier),
        "old cleanup cannot double-retire or change the successor's wake"
    );
    assert_eq!(original.request().request_handle(), original_handle);
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&replacement)
            .unwrap(),
        1
    );
    let raw = court
        .bridge
        .validate_owned_async_completion_envelope(&original, completion(&original))
        .unwrap();
    let obsolete = court
        .bridge
        .admit_owned_async_completion(&original, &raw)
        .unwrap();
    assert!(obsolete.report().admitted_completion().is_none());
    assert!(obsolete.report().denied_completion().is_some());
    let timeout = court
        .bridge
        .advance_managed_owned_async_request_to_timeout(&mut successor, 5)
        .unwrap();
    assert!(
        timeout.report().timed_out_request().is_some(),
        "successor retains its real timeout"
    );
    assert_eq!(
        original.request().request_handle(),
        original_handle,
        "retained old receipt is unchanged"
    );
    court
        .bridge
        .finish_unqueued_owned_async_entry_cleanup(&mut successor)
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&replacement)
            .unwrap(),
        0
    );
}

#[test]
fn native_initial_coalescence_keeps_winner_and_retires_only_new_unused_timeout() {
    let court =
        crate::owned_async::ordinary_context::native_service::NativeServiceCourt::coalescing();
    let winner = court.owner.admitted_request().handle();
    let (before, frontier) = court
        .port
        .owned_async_wake_frontier_for_test(&court.source)
        .unwrap();
    assert_eq!(before.scheduled_count(), 1);
    assert_eq!(before.ready_count(), 0);
    let original_wake = frontier.next_due_wake_id().unwrap();
    let unused_wake = before.next_wake_id();
    let mut joined = court
        .port
        .admit_unqueued_owned_async_request(&court.source)
        .unwrap();
    assert!(joined.refusal().is_none());
    assert_eq!(joined.admitted_request().handle(), winner);
    assert!(!joined.admitted_current_occurrence());
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Admission(report) =
        joined.report()
    else {
        panic!("actual initial coalescence report")
    };
    let coalescing = report.intent_equivalence_coalescing().unwrap();
    assert_eq!(coalescing.winner(), winner);
    assert_ne!(
        coalescing.coalesced_request().handle(),
        winner,
        "the report retains its real terminal side occurrence"
    );
    let (after, after_frontier) = court
        .port
        .owned_async_wake_frontier_for_test(&court.source)
        .unwrap();
    assert_eq!(
        after.scheduled_count(),
        1,
        "the new unused timeout must not remain scheduled"
    );
    assert_eq!(after.ready_count(), 0);
    assert_eq!(after.retired_count(), before.retired_count() + 1);
    assert_eq!(after.next_wake_id().get(), unused_wake.get() + 1);
    assert_eq!(after_frontier.next_due_wake_id(), Some(original_wake));
    assert_ne!(after_frontier.next_due_wake_id(), Some(unused_wake));
    court
        .port
        .finish_managed_owned_async_wake_cleanup(&mut joined)
        .unwrap();
    assert_eq!(
        court
            .port
            .owned_async_wake_frontier_for_test(&court.source)
            .unwrap(),
        (after, after_frontier)
    );
    assert_eq!(
        court
            .port
            .active_owned_async_request_count(&court.source)
            .unwrap(),
        1
    );
    let progress = court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut joined, 5)
        .unwrap();
    assert_eq!(
        progress,
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::TimedOut
    );
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Timeout(report) =
        joined.report()
    else {
        panic!("real winner timeout")
    };
    assert!(report.timed_out_request().is_some());
    assert_eq!(joined.admitted_request().handle(), winner);
    assert_eq!(
        court
            .port
            .active_owned_async_request_count(&court.source)
            .unwrap(),
        0
    );
    let (terminal, terminal_frontier) = court
        .port
        .owned_async_wake_frontier_for_test(&court.source)
        .unwrap();
    assert_eq!(terminal.scheduled_count(), 0);
    assert_eq!(terminal.ready_count(), 0);
    assert_eq!(terminal.retired_count(), after.retired_count() + 1);
    assert!(terminal_frontier.next_due_wake_id().is_none());
}
