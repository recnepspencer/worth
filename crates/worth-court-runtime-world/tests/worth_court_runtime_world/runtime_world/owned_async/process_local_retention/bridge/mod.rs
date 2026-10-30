use worth_signal::facade::ResourceLifecycleClass;
use {
    crate::owned_async::ordinary_context::fixture::admit_managed,
    crate::owned_async::ordinary_context::fixture::completion,
    crate::owned_async::ordinary_context::fixture::selected_fork,
    crate::owned_async::ordinary_context::fixture::AsyncSourceCourt,
};
use {
    worth_signal::facade::branch::SignalOwnedAsyncManagedReport as Report,
    worth_signal::facade::branch::SignalOwnedAsyncQueueDenial as Denial,
};

#[test]
fn installed_process_local_retry_coalescence_and_commit_preserve_exact_release_custody() {
    let court = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::process_local(
        std::num::NonZeroU64::MIN,
    );
    let selected = selected_fork(&court);
    let mut owner = admit_managed(&court, &selected);
    let original = owner.retained_request().unwrap();
    let timeout = court
        .bridge
        .advance_managed_owned_async_request_to_timeout(&mut owner, 5)
        .unwrap();
    let schedule = court
        .bridge
        .schedule_managed_owned_async_retry(&mut owner, &timeout)
        .unwrap();
    court
        .bridge
        .advance_managed_owned_async_retry(&mut owner, &schedule, 7)
        .unwrap();
    let retried = owner.request_handle();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let newer = court.publish(execution, &selected);
    court
        .world
        .revalidate_managed_owned_async_request(
            &court.bridge,
            &mut owner,
            &newer,
            &court.lease(&newer),
        )
        .unwrap();
    assert_ne!(owner.request_handle(), retried);
    let winner = owner.request_handle();
    let newest = court.publish(execution, &newer);
    court
        .world
        .revalidate_managed_owned_async_request(
            &court.bridge,
            &mut owner,
            &newest,
            &court.lease(&newest),
        )
        .unwrap();
    assert_eq!(owner.request_handle(), winner);
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Revalidation(report) =
        owner.native_report()
    else {
        panic!("genuine revalidation")
    };
    assert!(report
        .admitted_revalidation()
        .unwrap()
        .coalescing()
        .is_some());
    court
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap();
    let released = court
        .bridge
        .release_process_local_managed_request(owner, None)
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    assert_eq!(released.native().terminal().handle(), winner);
    assert!(released.native().released_predecessor().is_some());
    let (empty, wakes) = court
        .bridge
        .owned_async_retention_storage_for_test(&original)
        .unwrap();
    assert_eq!(empty.in_flight, 0);
    assert_eq!(empty.terminal, 0);
    assert_eq!(empty.retained_retry, 0);
    assert_eq!(empty.pruned_retry, 0);
    assert_eq!(empty.pruned_lifecycle, 0);
    assert_eq!(empty.retired_wakes, 0);
    assert_eq!(wakes.retired_count(), 5);
    assert_eq!(wakes.scheduled_count(), 0);
    assert_eq!(wakes.ready_count(), 0);

    let mut owner = admit_managed(&court, &selected);
    let witness = owner.retained_request().unwrap();
    let handle = owner.request_handle();
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let envelope = court
        .bridge
        .validate_owned_async_completion_envelope(&witness, completion(&witness))
        .unwrap();
    let admitted = court
        .bridge
        .admit_owned_async_completion(&witness, &envelope)
        .unwrap();
    let committed = court
        .bridge
        .commit_owned_async_completion(&witness, &admitted)
        .unwrap();
    assert!(committed.timeout_cleanup_complete());
    let before = court
        .bridge
        .owned_async_retention_storage_for_test(&witness)
        .unwrap();
    let refusal = court
        .bridge
        .release_process_local_managed_request(owner, Some(committed))
        .err()
        .unwrap();
    assert!(matches!(
        refusal.cause(),
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
            worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::QueueNotDrained { depth: 1 }
        )
    ));
    let (returned, committed, _) = refusal.into_parts();
    owner = returned;
    assert_eq!(owner.request_handle(), handle);
    let committed = committed.expect("the genuine commit owner survives refusal");
    assert_eq!(
        committed.report().lifecycle().lifecycle(),
        worth_signal::facade::ResourceLifecycleClass::Fulfilled
    );
    assert_eq!(
        court
            .bridge
            .owned_async_retention_storage_for_test(&witness)
            .unwrap(),
        before
    );
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let before = court
        .bridge
        .owned_async_retention_storage_for_test(&witness)
        .unwrap();
    assert!(matches!(
        court
            .world
            .revalidate_managed_owned_async_request(
                &court.bridge,
                &mut owner,
                &newer,
                &court.lease(&newer),
            ),
        Err(worth_runtime_world::facade::RuntimeWorldOwnedAsyncManagedRevalidationDenial::Bridge(
            worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::ReleaseRequired)
        ))),
        "fulfilled ProcessLocal custody requires its actual completion release first"
    );
    assert_eq!(
        court
            .bridge
            .owned_async_retention_storage_for_test(&witness)
            .unwrap(),
        before
    );
    let refusal = court
        .bridge
        .release_process_local_managed_request(owner, None)
        .err()
        .unwrap();
    assert!(matches!(refusal.cause(), worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
        worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::DefinitionMismatch)
    )));
    let (returned, absent, _) = refusal.into_parts();
    assert!(absent.is_none());
    assert_eq!(returned.request_handle(), handle);
    assert_eq!(
        court
            .bridge
            .owned_async_retention_storage_for_test(&witness)
            .unwrap(),
        before
    );
    let released = court
        .bridge
        .release_process_local_managed_request(returned, Some(committed))
        .unwrap_or_else(|refusal| panic!("{:?}", refusal.cause()));
    assert!(released.native().completion_report().is_some());
    let (after, wakes) = court
        .bridge
        .owned_async_retention_storage_for_test(&witness)
        .unwrap();
    assert_eq!(after, empty);
    assert_eq!(after.retired_wakes, 0);
    assert_eq!(wakes.retired_count(), 6);
    assert_eq!(wakes.scheduled_count(), 0);
}
