use super::*;

pub(in crate::tests::resource_runtime) fn exercise_resource_async_hostile_suffix_on_active_branch(
    runtime: &mut TestRuntime,
    lifecycle_node: NodeId,
    cancel_node: NodeId,
    timeout_node: NodeId,
    malformed_node: NodeId,
    retained_denial_request_id: ResourceRequestId,
) -> (
    SignalSnapshotV1,
    Option<SignalSnapshotId>,
    ResourceReplayReconstructionReport,
    ReplaySlice,
    ResourceReplayReconstructionReport,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let digest = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(lifecycle_node))
        .expect("resource descriptor should exist")
        .payload_contract_digest()
        .clone();
    runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            retained_denial_request_id,
            ResourceGeneration::new(1),
            ResourceBranchEpoch::new(runtime.graph().current_branch().id, 0),
            ResourceAttemptId::ZERO,
            digest,
            32,
        ))
        .denied_completion()
        .expect("unknown request should produce retained denial evidence");

    let first_admission = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            lifecycle_node,
        )))
        .expect("first lifecycle request should admit");
    let stale_first = raw_completion(
        runtime,
        lifecycle_node,
        first_admission.admitted_request().handle(),
        first_admission.admitted_request().attempt(),
        64,
    );
    let second_admission = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            lifecycle_node,
        )))
        .expect("second lifecycle request should supersede first request");
    let second_request = second_admission.admitted_request();
    let tx_admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            runtime,
            lifecycle_node,
            second_request.handle(),
            second_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("current lifecycle completion should admit");
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    let tx_staging = tx
        .stage_admitted_resource_completion(tx_admitted_completion)
        .expect("transactional lifecycle completion should stage");
    tx.commit_staged_resource_completion(tx_staging.staged_effect())
        .expect("transactional lifecycle completion should mutate transaction-local state");
    tx.rollback()
        .expect("transaction rollback should restore the active lifecycle request");
    let rollback_completion = runtime
        .admit_resource_completion(raw_completion(
            runtime,
            lifecycle_node,
            second_request.handle(),
            second_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("restored lifecycle request should admit a second completion proof");
    let rollback_staging = runtime
        .stage_admitted_resource_completion(rollback_completion)
        .expect("runtime rollback completion should stage");
    runtime.rollback_staged_resource_completion(rollback_staging.staged_effect());
    let superseded_completion_report = runtime.admit_resource_completion(stale_first);
    assert_eq!(
        superseded_completion_report
            .denied_completion()
            .expect("late superseded completion should deny explicitly")
            .class(),
        CompletionDenialClass::Superseded
    );

    let cancel_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            cancel_node,
        )))
        .expect("cancel request should admit")
        .admitted_request();
    let late_cancelled = raw_completion(
        runtime,
        cancel_node,
        cancel_request.handle(),
        cancel_request.attempt(),
        64,
    );
    runtime
        .cancel_resource_request(
            cancel_request.handle(),
            ResourceCancellationReason::HostRequested,
        )
        .expect("cancellation should retire the active request");
    let cancelled_completion_report = runtime.admit_resource_completion(late_cancelled);
    assert_eq!(
        cancelled_completion_report
            .denied_completion()
            .expect("late cancelled completion should deny explicitly")
            .class(),
        CompletionDenialClass::Cancelled
    );

    let timeout_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            timeout_node,
        )))
        .expect("timeout request should admit")
        .admitted_request();
    let late_timed_out = raw_completion(
        runtime,
        timeout_node,
        timeout_request.handle(),
        timeout_request.attempt(),
        64,
    );
    let timeout_wake = runtime
        .in_flight_resource_request(timeout_request.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("timeout wake should attach");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .expect("authoritative clock should advance");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should become ready");
    runtime
        .admit_resource_timeout(timeout_request.handle(), ready_timeout)
        .expect("timeout admission should consume the wake");
    let timed_out_completion_report = runtime.admit_resource_completion(late_timed_out);
    assert_eq!(
        timed_out_completion_report
            .denied_completion()
            .expect("late timed out completion should deny explicitly")
            .class(),
        CompletionDenialClass::TimedOut
    );

    let malformed_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            malformed_node,
        )))
        .expect("malformed request should admit")
        .admitted_request();
    let malformed_completion_report =
        runtime.admit_resource_completion(RawCompletionEnvelope::new(
            malformed_request.handle().request_id(),
            malformed_request.handle().generation(),
            malformed_request.handle().branch_epoch(),
            malformed_request.attempt(),
            ResourcePayloadContractDigest::new("payload-contract:999:1024"),
            64,
        ));
    assert_eq!(
        malformed_completion_report
            .denied_completion()
            .expect("malformed completion should deny explicitly")
            .class(),
        CompletionDenialClass::Malformed
    );

    let snapshot = runtime
        .capture_branch_snapshot(runtime.observe().current_branch())
        .expect("branch snapshot should capture the hostile suffix checkpoint");
    let head_snapshot_before_restore = runtime
        .observe()
        .branch_head_snapshot_id(runtime.observe().current_branch().id);
    let replay_before_restore = runtime.reconstruct_resource_replay_summary();
    let replay_history_before_restore = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);
    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            lifecycle_node,
        )))
        .expect("post-snapshot request should drift branch state before restore");
    let replay_after_snapshot_drift = runtime.reconstruct_resource_replay_summary();
    assert_ne!(
        replay_after_snapshot_drift.replay_digest(),
        replay_before_restore.replay_digest(),
        "branch drift after the checkpoint must perturb replay truth before restore"
    );
    assert_eq!(
        runtime
            .observe()
            .branch_head_snapshot_id(runtime.observe().current_branch().id),
        head_snapshot_before_restore,
        "post-checkpoint branch drift must not rewrite the captured head snapshot"
    );

    (
        snapshot,
        head_snapshot_before_restore,
        replay_before_restore,
        replay_history_before_restore,
        replay_after_snapshot_drift,
    )
}
