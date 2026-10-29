use {crate::owned_async::ordinary_context::native_service::NativeServiceCourt};
use {worth_signal::facade::branch::SignalConditionalExecutionPort, worth_signal::facade::branch::SignalConditionalServiceExecutionDenial, worth_signal::facade::branch::SignalOwnedAsyncCompletionAdmission, worth_signal::facade::branch::SignalOwnedAsyncCompletionCommit, worth_signal::facade::branch::SignalOwnedAsyncQueueDenial};
use {worth_signal::facade::CompletionDenialClass, worth_signal::facade::ResourceLifecycleClass};

#[test]
fn local_selector_collisions_cannot_substitute_foreign_service_custody() {
    // This compiled signature excludes the public serializable report DTO.
    let _: fn(
        &SignalConditionalExecutionPort<(), (), ()>,
        &SignalOwnedAsyncCompletionAdmission,
    )
        -> Result<SignalOwnedAsyncCompletionCommit, SignalConditionalServiceExecutionDenial> =
        worth_signal::facade::branch::SignalConditionalExecutionPort::commit_owned_async_completion;
    let first = crate::owned_async::ordinary_context::native_service::NativeServiceCourt::new();
    let second = crate::owned_async::ordinary_context::native_service::NativeServiceCourt::new();
    assert_eq!(first.queue().request(), second.queue().request());
    assert_eq!(first.queue().attempt(), second.queue().attempt());
    assert_eq!(
        first.queue().queue_capacity(),
        second.queue().queue_capacity()
    );

    let oversized = first
        .port
        .admit_owned_async_completion(&first.source, first.envelope(65))
        .unwrap();
    assert_eq!(
        oversized.report().denied_completion().unwrap().class(),
        worth_signal::facade::CompletionDenialClass::Partial
    );
    assert!(first
        .port
        .commit_owned_async_completion(&oversized)
        .is_err());
    assert_eq!(
        first
            .port
            .active_owned_async_request_count(&first.source)
            .unwrap(),
        1
    );
    let admitted = first
        .port
        .admit_owned_async_completion(&first.source, first.envelope(64))
        .unwrap();
    assert!(admitted.report().admitted_completion().is_some());
    assert!(matches!(
        second.port.commit_owned_async_completion(&admitted),
        Err(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch)
    ));
    assert!(matches!(
        second.port.enqueue_owned_async_queue(first.queue(), 1),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert!(matches!(
        second.port.observe_owned_async_safe_point(first.queue()),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(second.queue())
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    assert_eq!(
        second
            .port
            .active_owned_async_request_count(&second.source)
            .unwrap(),
        1
    );

    first
        .port
        .enqueue_owned_async_queue(first.queue(), 1)
        .unwrap();
    first
        .port
        .dequeue_owned_async_queue(first.queue(), 1)
        .unwrap();
    let mut committed = first.port.commit_owned_async_completion(&admitted).unwrap();
    assert_eq!(
        committed.report().lifecycle().lifecycle(),
        worth_signal::facade::ResourceLifecycleClass::Fulfilled
    );
    assert!(committed.timeout_cleanup_complete());
    assert!(
        matches!(
            second
                .port
                .finish_owned_async_completion_timeout_cleanup(&mut committed),
            Err(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch)
        ),
        "already-complete cleanup still validates its actual originating service"
    );
    assert_eq!(
        second
            .port
            .active_owned_async_request_count(&second.source)
            .unwrap(),
        1
    );
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(second.queue())
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    first
        .port
        .finish_owned_async_completion_timeout_cleanup(&mut committed)
        .unwrap();
    assert_eq!(
        first
            .port
            .active_owned_async_request_count(&first.source)
            .unwrap(),
        0
    );

    second
        .port
        .enqueue_owned_async_queue(second.queue(), 1)
        .unwrap();
    assert!(matches!(
        first.port.dequeue_owned_async_queue(second.queue(), 1),
        Err(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch
        ))
    ));
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(second.queue())
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    let cancelled = second
        .port
        .cancel_owned_async_request(&second.source, second.queue().request())
        .unwrap();
    assert!(cancelled.cancelled_request().is_some());
    second
        .port
        .dequeue_owned_async_queue(second.queue(), 1)
        .unwrap();
    assert_eq!(
        second
            .port
            .observe_owned_async_safe_point(second.queue())
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
}
