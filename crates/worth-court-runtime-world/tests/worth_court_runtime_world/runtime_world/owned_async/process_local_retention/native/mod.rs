use crate::owned_async::ordinary_context::native_service::NativeServiceCourt;
use worth_signal::facade::ResourceLifecycleClass;
use {
    worth_signal::facade::branch::SignalOwnedAsyncManagedProgress as Progress,
    worth_signal::facade::branch::SignalOwnedAsyncQueueDenial as Denial,
};

#[test]
fn exact_process_local_release_bounds_repeated_storage_without_expiring_a_rich_producer() {
    let (court, other_node) = crate::owned_async::ordinary_context::native_service::NativeServiceCourt::process_local_two_nodes();
    let observe = || {
        court
            .port
            .owned_async_retention_storage_for_test(&court.source)
            .unwrap()
    };
    let other_source = court
        .port
        .bind_owned_async_source(court.port.issuance_basis(), other_node)
        .unwrap();
    let rich = court
        .port
        .admit_managed_owned_async_request(&other_source, 1)
        .unwrap();
    court
        .port
        .cancel_owned_async_request(&other_source, rich.admitted_request().handle())
        .unwrap();
    let rich_queue = rich.queue().unwrap();
    let before = observe();
    assert!(matches!(court.port.admit_process_local_managed_owned_async_request(&other_source, 1),
        Err(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch)));
    assert_eq!(
        observe(),
        before,
        "the free Rich node refuses ProcessLocal admission before creating a wake or occurrence"
    );
    let mut live = Some(court.owner);
    let mut baseline = None;
    for iteration in 0..20 {
        let owner = live.take().unwrap();
        let denied = court
            .port
            .admit_owned_async_completion(
                &court.source,
                crate::owned_async::ordinary_context::native_service::NativeServiceCourt::envelope_for(&owner, &court.payload, 65),
            )
            .unwrap();
        assert!(denied.report().denied_completion().is_some());
        let (stored, _) = observe();
        assert_eq!(stored.denied, 1);
        assert_eq!(stored.latest_denied, 1);
        court
            .port
            .release_process_local_completion_denial(&denied)
            .unwrap();
        assert_eq!(
            court
                .port
                .active_owned_async_request_count(&court.source)
                .unwrap(),
            1
        );
        court
            .port
            .cancel_owned_async_request(&court.source, owner.admitted_request().handle())
            .unwrap();
        let released = court
            .port
            .release_process_local_managed_request(owner, None)
            .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
        assert_eq!(
            released.terminal().lifecycle(),
            worth_signal::facade::ResourceLifecycleClass::Cancelled
        );
        let (storage, wakes) = observe();
        assert_eq!(
            storage.in_flight, 1,
            "only the independently retained Rich terminal remains"
        );
        assert_eq!(storage.terminal, 1);
        assert_eq!(storage.retained_lifecycle, 0);
        assert_eq!(storage.pruned_lifecycle, 0);
        assert_eq!(storage.denied, 0);
        assert_eq!(storage.pruned_denied, 0);
        assert_eq!(storage.latest_denied, 0);
        assert_eq!(storage.retained_retry, 0);
        assert_eq!(storage.pruned_retry, 0);
        assert_eq!(storage.pending_retry_requests, 0);
        assert_eq!(storage.pending_retry_wakes, 0);
        assert_eq!(storage.pending_retry_nodes, 0);
        assert_eq!(storage.retry_budget_generations, 0);
        assert_eq!(wakes.scheduled_count(), 0);
        assert_eq!(wakes.ready_count(), 0);
        assert_eq!(
            storage.retired_wakes, 1,
            "the Rich producer's wake was not expired"
        );
        assert_eq!(
            wakes.retired_count(),
            iteration + 2,
            "logical expiry remains in the existing omission summary"
        );
        if let Some(expected) = baseline {
            assert_eq!(storage, expected);
        } else {
            baseline = Some(storage);
        }
        assert_eq!(
            court
                .port
                .observe_owned_async_safe_point(rich_queue)
                .unwrap()
                .pressure()
                .queue_depth(),
            0
        );
        live = Some(
            court
                .port
                .admit_process_local_managed_owned_async_request(&court.source, 1)
                .unwrap(),
        );
    }

    // A real newer occurrence denies the old acquired backoff. Cleanup refusal
    // must return the old custodian, then clear only that exact old retry/wake.
    let mut old = live.take().unwrap();
    let first = old.admitted_request().handle();
    court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut old, 5)
        .unwrap();
    let first_schedule = court
        .port
        .schedule_managed_owned_async_retry(&mut old)
        .unwrap();
    assert_eq!(
        court
            .port
            .advance_managed_owned_async_retry(&court.source, &mut old, &first_schedule, 7)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Replaced
    );
    assert_ne!(old.admitted_request().handle(), first);
    assert_eq!(
        old.admitted_request().handle().generation(),
        first.generation()
    );
    court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut old, 12)
        .unwrap();
    assert_eq!(
        observe().0.retry_budget_generations,
        1,
        "exact predecessor release preserves its current retry custodian's charged generation"
    );
    let schedule = court
        .port
        .schedule_managed_owned_async_retry(&mut old)
        .unwrap();
    assert_eq!(
        schedule
            .report()
            .scheduled_retry()
            .unwrap()
            .retry_budget_usage(),
        Some(2)
    );
    let mut successor = court
        .port
        .admit_process_local_managed_owned_async_request(&court.source, 1)
        .unwrap();
    let successor_handle = successor.admitted_request().handle();
    court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut successor, 17)
        .unwrap();
    let successor_schedule = court
        .port
        .schedule_managed_owned_async_retry(&mut successor)
        .unwrap();
    let newest = court
        .port
        .admit_process_local_managed_owned_async_request(&court.source, 1)
        .unwrap();
    let newest_handle = newest.admitted_request().handle();
    assert_eq!(observe().0.pending_retry_requests, 2);
    assert_eq!(observe().0.pending_retry_nodes, 1);
    assert_eq!(observe().0.retry_budget_generations, 2);
    assert_eq!(
        court
            .port
            .advance_managed_owned_async_retry(&court.source, &mut old, &schedule, 17)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Denied
    );
    assert!(old.wake_cleanup_pending());
    let before = observe();
    let refusal = court
        .port
        .release_process_local_managed_request(old, None)
        .err()
        .unwrap();
    assert!(matches!(
        refusal.cause(),
        worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::CleanupPending
    ));
    let (mut old, completion, _) = refusal.into_parts();
    assert!(completion.is_none());
    assert_eq!(observe(), before);
    court
        .port
        .finish_managed_owned_async_wake_cleanup(&mut old)
        .unwrap();
    court
        .port
        .release_process_local_managed_request(old, None)
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    assert_eq!(
        court
            .port
            .active_owned_async_request_count(&court.source)
            .unwrap(),
        1
    );
    assert_eq!(
        observe().0.pending_retry_requests,
        1,
        "old cleanup preserves the newer same-node pending request"
    );
    assert_eq!(observe().0.pending_retry_nodes, 1);
    assert_eq!(observe().0.pending_retry_wakes, 1);
    assert_eq!(
        observe().0.retry_budget_generations,
        1,
        "old release preserves the actual successor's charged generation"
    );
    assert_eq!(successor.queue().unwrap().request(), successor_handle);
    assert_eq!(
        court
            .port
            .observe_owned_async_safe_point(successor.queue().unwrap())
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    assert_eq!(newest.queue().unwrap().request(), newest_handle);
    court
        .port
        .cancel_owned_async_request(&court.source, newest_handle)
        .unwrap();
    court
        .port
        .release_process_local_managed_request(newest, None)
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    court
        .port
        .abort_process_local_owned_async_retry(&mut successor)
        .unwrap();
    assert_eq!(
        successor.released_retry().unwrap().scheduled(),
        successor_schedule.report().scheduled_retry().unwrap()
    );
    court
        .port
        .release_process_local_managed_request(successor, None)
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    let (storage, wakes) = observe();
    assert_eq!(Some(storage), baseline);
    assert_eq!(storage.retired_wakes, 1);
    assert_eq!(wakes.retired_count(), 28);
    let mut closing = court
        .port
        .admit_process_local_managed_owned_async_request(&court.source, 1)
        .unwrap();
    court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut closing, 24)
        .unwrap();
    let scheduled = court
        .port
        .schedule_managed_owned_async_retry(&mut closing)
        .unwrap();
    let backoff = scheduled
        .report()
        .scheduled_retry()
        .unwrap()
        .backoff_wake_id();
    assert_eq!(
        court
            .port
            .advance_managed_owned_async_retry(&court.source, &mut closing, &scheduled, 25)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::TemporalDeferred
    );
    drop(scheduled);
    assert!(matches!(
        closing.report(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedReport::TemporalDeferred(_)
    ));
    let before_abort = observe();
    assert_eq!(before_abort.0.pending_retry_requests, 1);
    court
        .port
        .abort_process_local_owned_async_retry(&mut closing)
        .unwrap();
    let aborted = closing
        .released_retry()
        .expect("the genuine schedule survives report overwrite");
    assert_eq!(aborted.scheduled().backoff_wake_id(), backoff);
    assert_eq!(aborted.retired().id(), backoff);
    assert_eq!(
        aborted.retired().retired_tick().get(),
        25,
        "abort does not advance time to backoff due26"
    );
    assert_eq!(
        aborted.retired().reason(),
        worth_signal::facade::TemporalWakeRetirementReason::Disposed
    );
    assert_eq!(
        aborted.scheduled().previous(),
        closing.admitted_request().handle()
    );
    let released = court
        .port
        .release_process_local_managed_request(closing, None)
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    assert_eq!(released.released_retry().unwrap().retired().id(), backoff);
    let (storage, wakes) = observe();
    assert_eq!(Some(storage), baseline);
    assert_eq!(wakes.retired_count(), 30);
    assert_eq!(
        court
            .port
            .observe_owned_async_safe_point(rich_queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
}
