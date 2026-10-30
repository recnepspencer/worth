use crate::owned_async::ordinary_context::native_service::NativeServiceCourt;
use {
    worth_signal::facade::branch::SignalConditionalServiceExecutionDenial,
    worth_signal::facade::branch::SignalOwnedAsyncManagedProgress as Progress,
    worth_signal::facade::branch::SignalOwnedAsyncManagedReport as Report,
    worth_signal::facade::branch::SignalOwnedAsyncQueueDenial as Denial,
};
use {
    worth_signal::facade::ResourceRetryDenialClass,
    worth_signal::facade::ResourceRevalidationIntent,
};

#[test]
fn reissued_service_transfers_actual_queue_origin_without_rebinding_a_coalesced_queue() {
    let mut court = crate::owned_async::ordinary_context::native_service::NativeServiceCourt::new();
    let reissued = court
        .port
        .reissue_for_successor_basis(court.port.issuance_basis())
        .unwrap();
    let source = reissued
        .bind_owned_async_source(reissued.issuance_basis(), court.node)
        .unwrap();
    let prior = court.owner.admitted_request().handle();
    assert_eq!(
        reissued
            .revalidate_managed_owned_async_request(
                &source,
                &mut court.owner,
                worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
                    court.node, prior
                ),
            )
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Replaced
    );
    let winner = court.owner.admitted_request().handle();
    assert_ne!(winner, prior);
    assert_eq!(
        reissued
            .revalidate_managed_owned_async_request(
                &source,
                &mut court.owner,
                worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
                    court.node, winner
                ),
            )
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Coalesced
    );
    assert_eq!(court.owner.admitted_request().handle(), winner);
    assert!(court.owner.refusal().is_none());
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Revalidation(report) =
        court.owner.report()
    else {
        panic!("native report")
    };
    assert!(report
        .admitted_revalidation()
        .unwrap()
        .coalescing()
        .is_some());
    assert!(matches!(
        court.port.enqueue_owned_async_queue(court.queue(), 1),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    reissued
        .enqueue_owned_async_queue(court.queue(), 1)
        .unwrap();
    reissued
        .dequeue_owned_async_queue(court.queue(), 1)
        .unwrap();
    reissued
        .cancel_owned_async_request(&source, winner)
        .unwrap();
    let independent = reissued
        .admit_managed_owned_async_request(&source, 1)
        .unwrap();
    let mut older = std::mem::replace(&mut court.owner, independent);
    let successor_before = court.owner.admitted_request().handle();
    assert_eq!(
        reissued
            .revalidate_managed_owned_async_request(
                &source,
                &mut court.owner,
                worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
                    court.node,
                    successor_before
                ),
            )
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Replaced
    );
    let successor = court.owner.admitted_request().handle();
    let old_queue = older.queue().unwrap().request();
    assert!(matches!(
        reissued.revalidate_managed_owned_async_request(
            &source,
            &mut older,
            worth_signal::facade::ResourceRevalidationIntent::with_expected_active(court.node, winner),
        ),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(older.admitted_request().handle(), winner);
    assert_eq!(older.queue().unwrap().request(), old_queue);
    assert_eq!(court.owner.admitted_request().handle(), successor);
    assert_eq!(court.queue().request(), successor);
    assert_eq!(
        reissued.active_owned_async_request_count(&source).unwrap(),
        1
    );
    assert_eq!(
        reissued
            .observe_owned_async_safe_point(court.queue())
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    reissued
        .cancel_owned_async_request(&source, successor)
        .unwrap();
}

#[test]
fn foreign_schedule_cannot_advance_clock_and_refused_ready_retry_cleans_only_its_occurrence() {
    let mut first = crate::owned_async::ordinary_context::native_service::NativeServiceCourt::new();
    let mut second =
        crate::owned_async::ordinary_context::native_service::NativeServiceCourt::new();
    first
        .port
        .advance_managed_owned_async_request_to_timeout(&mut first.owner, 5)
        .unwrap();
    second
        .port
        .advance_managed_owned_async_request_to_timeout(&mut second.owner, 5)
        .unwrap();
    let foreign = first
        .port
        .schedule_managed_owned_async_retry(&mut first.owner)
        .unwrap();
    let own = second
        .port
        .schedule_managed_owned_async_retry(&mut second.owner)
        .unwrap();
    assert_eq!(
        foreign.report().scheduled_retry().unwrap().previous(),
        own.report().scheduled_retry().unwrap().previous()
    );
    assert_eq!(
        foreign
            .report()
            .scheduled_retry()
            .unwrap()
            .backoff_wake_id(),
        own.report().scheduled_retry().unwrap().backoff_wake_id()
    );
    assert!(matches!(
        second.port.advance_managed_owned_async_retry(
            &second.source,
            &mut second.owner,
            &foreign,
            7,
        ),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(
        second
            .port
            .advance_managed_owned_async_retry(&second.source, &mut second.owner, &own, 6,)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::TemporalDeferred
    );
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::TemporalDeferred(clock) =
        second.owner.report()
    else {
        panic!("actual native clock")
    };
    assert_eq!(
        clock.current_tick().get(),
        6,
        "a foreign schedule did not advance the native clock to seven"
    );
    let successor = second
        .port
        .admit_managed_owned_async_request(&second.source, 1)
        .unwrap();
    let successor_handle = successor.admitted_request().handle();
    assert_ne!(successor_handle, second.owner.admitted_request().handle());
    assert_eq!(
        second
            .port
            .advance_managed_owned_async_retry(&second.source, &mut second.owner, &own, 7,)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Denied
    );
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Retry(denied) =
        second.owner.report()
    else {
        panic!("genuine denied retry")
    };
    assert_eq!(
        denied.denied_retry().unwrap().class(),
        worth_signal::facade::ResourceRetryDenialClass::SupersededByNewerRequest
    );
    assert!(second.owner.wake_cleanup_pending());
    assert!(matches!(
        second
            .port
            .advance_managed_owned_async_retry(&second.source, &mut second.owner, &own, 7,),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::CleanupPending)
    ));
    second
        .port
        .finish_managed_owned_async_wake_cleanup(&mut second.owner)
        .unwrap();
    assert!(!second.owner.wake_cleanup_pending());
    assert_eq!(
        second
            .port
            .active_owned_async_request_count(&second.source)
            .unwrap(),
        1
    );
    let queue = successor.queue().unwrap();
    second.port.enqueue_owned_async_queue(queue, 1).unwrap();
    second.port.dequeue_owned_async_queue(queue, 1).unwrap();
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(queue)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    assert!(second
        .port
        .cancel_owned_async_request(&second.source, successor_handle)
        .unwrap()
        .cancelled_request()
        .is_some());
    assert_eq!(
        first
            .port
            .advance_managed_owned_async_retry(&first.source, &mut first.owner, &foreign, 7,)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Replaced
    );
    first
        .port
        .cancel_owned_async_request(&first.source, first.owner.admitted_request().handle())
        .unwrap();
}

#[test]
fn same_port_other_resource_cannot_advance_the_custodied_retry() {
    let (mut court, other_node) =
        crate::owned_async::ordinary_context::native_service::NativeServiceCourt::two_nodes();
    let other = court
        .port
        .bind_owned_async_source(court.port.issuance_basis(), other_node)
        .unwrap();
    court
        .port
        .advance_managed_owned_async_request_to_timeout(&mut court.owner, 5)
        .unwrap();
    let original = court.owner.admitted_request().handle();
    assert_eq!(
        court.port.active_owned_async_request_count(&other).unwrap(),
        0
    );
    let schedule = court
        .port
        .schedule_managed_owned_async_retry(&mut court.owner)
        .unwrap();
    assert!(matches!(
        court
            .port
            .advance_managed_owned_async_retry(&other, &mut court.owner, &schedule, 7),
        Err(
            worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
                worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
            )
        )
    ));
    assert_eq!(court.owner.admitted_request().handle(), original);
    assert_eq!(
        court
            .port
            .advance_managed_owned_async_retry(&court.source, &mut court.owner, &schedule, 6)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::TemporalDeferred
    );
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::TemporalDeferred(clock) =
        court.owner.report()
    else {
        panic!("actual clock")
    };
    assert_eq!(clock.current_tick().get(), 6);
    assert_eq!(
        court.port.active_owned_async_request_count(&other).unwrap(),
        0
    );
    assert_eq!(
        court
            .port
            .advance_managed_owned_async_retry(&court.source, &mut court.owner, &schedule, 7)
            .unwrap(),
        worth_signal::facade::branch::SignalOwnedAsyncManagedProgress::Replaced
    );
    assert_ne!(court.owner.admitted_request().handle(), original);
    assert_eq!(
        court.queue().request(),
        court.owner.admitted_request().handle()
    );
    let other_owner = court
        .port
        .admit_managed_owned_async_request(&other, 1)
        .unwrap();
    assert!(other_owner.refusal().is_none());
    assert_eq!(
        court.port.active_owned_async_request_count(&other).unwrap(),
        1
    );
    court
        .port
        .cancel_owned_async_request(&other, other_owner.admitted_request().handle())
        .unwrap();
    court
        .port
        .cancel_owned_async_request(&court.source, court.owner.admitted_request().handle())
        .unwrap();
}
