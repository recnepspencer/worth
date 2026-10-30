use {
    crate::owned_async::ordinary_context::fixture::admit_managed,
    crate::owned_async::ordinary_context::fixture::completion,
    crate::owned_async::ordinary_context::fixture::selected_fork,
    crate::owned_async::ordinary_context::fixture::source_commit,
};

#[test]
fn managed_retry_retains_queued_refusal_and_continues_after_real_before_due_progress() {
    let court = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::managed(
        std::num::NonZeroU64::MIN,
    );
    let selected = selected_fork(&court);
    let mut owner = admit_managed(&court, &selected);
    let prior = owner.request_handle();
    let prior_attempt = owner.request().unwrap().request().attempt();
    let obsolete = completion(owner.request().unwrap());
    let original_source = source_commit(owner.request().unwrap()).clone();
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let timeout = court
        .bridge
        .advance_managed_owned_async_request_to_timeout(&mut owner, 5)
        .unwrap();
    let schedule = court
        .bridge
        .schedule_managed_owned_async_retry(&mut owner, &timeout)
        .unwrap();
    assert!(matches!(
        court
            .bridge
            .advance_managed_owned_async_retry(&mut owner, &schedule, 7),
        Err(
            worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
                worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::QueueNotDrained {
                    depth: 1
                }
            )
        )
    ));
    assert_eq!(owner.request_handle(), prior);
    assert_eq!(
        court
            .bridge
            .observe_managed_owned_async_safe_point(&owner)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    assert!(matches!(
        court
            .bridge
            .advance_managed_owned_async_retry(&mut owner, &schedule, 6),
        Err(worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Service(
            worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::SlotAdmission(_)
        )))
    ));
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::TemporalDeferred(clock) =
        owner.native_report()
    else {
        panic!("the actual partial clock advance is retained without a retry report");
    };
    assert_eq!(clock.current_tick().get(), 6);
    assert!(!owner.wake_cleanup_pending());
    court
        .bridge
        .advance_managed_owned_async_retry(&mut owner, &schedule, 7)
        .unwrap();
    assert_ne!(owner.request_handle().request_id(), prior.request_id());
    assert_eq!(owner.request_handle().generation(), prior.generation());
    assert_eq!(
        owner.request().unwrap().request().attempt().get(),
        prior_attempt.get() + 1
    );
    assert_eq!(source_commit(owner.request().unwrap()), &original_source);
    let stale = court
        .bridge
        .validate_owned_async_completion_envelope(owner.request().unwrap(), obsolete)
        .unwrap_err();
    assert_eq!(
        stale.kind(),
        worth_runtime_bridge::facade::BridgeAsyncCompletionRejectionKind::EnvelopeHandleMismatch
    );
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(owner.request().unwrap())
            .unwrap(),
        1
    );
    let request = owner.request().unwrap();
    let envelope = court
        .bridge
        .validate_owned_async_completion_envelope(request, completion(request))
        .unwrap();
    let admitted = court
        .bridge
        .admit_owned_async_completion(request, &envelope)
        .unwrap();
    court
        .bridge
        .commit_owned_async_completion(request, &admitted)
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        0
    );
}

#[test]
fn managed_revalidation_preserves_source_and_queue_when_native_coalesces() {
    let court = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::managed(
        std::num::NonZeroU64::MIN,
    );
    let selected = selected_fork(&court);
    let mut owner = admit_managed(&court, &selected);
    let prior = owner.request_handle();
    let original_source = source_commit(owner.request().unwrap()).clone();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let newer = court.publish(execution, &selected);
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    assert!(matches!(
        court.world.revalidate_managed_owned_async_request(
            &court.bridge,
            &mut owner,
            &newer,
            &court.lease(&newer)
        ),
        Err(
            worth_runtime_world::facade::RuntimeWorldOwnedAsyncManagedRevalidationDenial::Bridge(
                worth_runtime_bridge::facade::BridgeOwnedAsyncManagedDenial::Signal(
                    worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::QueueNotDrained {
                        depth: 1
                    }
                )
            )
        )
    ));
    assert_eq!(owner.request_handle(), prior);
    assert_eq!(source_commit(owner.request().unwrap()), &original_source);
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    assert!(matches!(
        court
            .world
            .revalidate_managed_owned_async_request(
                &court.bridge,
                &mut owner,
                &newer,
                &court.lease(&newer)
            )
            .unwrap(),
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRevalidation::Replaced(_)
    ));
    assert_ne!(owner.request_handle(), prior);
    let commit = newer
        .basis()
        .relational_basis()
        .observation()
        .commit_id()
        .unwrap();
    let admitted_source =
        worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(commit.0);
    assert_eq!(source_commit(owner.request().unwrap()), &admitted_source);
    let winner = owner.request_handle();
    let newest = court.publish(execution, &newer);
    assert!(matches!(
        court
            .world
            .revalidate_managed_owned_async_request(
                &court.bridge,
                &mut owner,
                &newest,
                &court.lease(&newest)
            )
            .unwrap(),
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRevalidation::Coalesced
    ));
    assert_eq!(owner.request_handle(), winner);
    assert_eq!(
        source_commit(owner.request().unwrap()),
        &admitted_source,
        "coalescing retains the original business source and performs no new read"
    );
    let worth_signal::facade::branch::SignalOwnedAsyncManagedReport::Revalidation(report) =
        owner.native_report()
    else {
        panic!("native revalidation report")
    };
    assert!(report
        .admitted_revalidation()
        .unwrap()
        .coalescing()
        .is_some());
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(owner.request().unwrap())
            .unwrap(),
        1
    );
    court
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap();
}
