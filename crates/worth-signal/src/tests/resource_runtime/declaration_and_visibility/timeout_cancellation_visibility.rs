use super::*;

#[test]
fn resource_timeout_reclassifies_hidden_pending_output_when_terminal_policy_preserves() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(
            hide_pending_output_resource_declaration(node).with_timeout_policy(
                ResourceTimeoutPolicyDeclaration::FixedTimeout {
                    timeout: TemporalDuration::temporal_duration(5).unwrap(),
                },
            ),
        )
        .expect("resource declaration should lower");

    let first = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request();
    let first_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            first.handle(),
            first.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("initial completion should admit");
    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staging = tx.stage_admitted_resource_completion(first_completion)?;
            tx.commit_staged_resource_completion(staging.staged_effect())?;
            Ok(())
        })
        .expect("completion transaction should commit");

    let pending_report = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("follow-up request should admit");
    let pending = pending_report.admitted_request();
    assert_eq!(
        pending_report.lifecycle().output_continuity(),
        ResourceOutputContinuity::OutputUnavailableByPolicy
    );

    let decisions_before_timeout = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;
    let timeout_wake = runtime
        .in_flight_resource_request(pending.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("pending request should have a timeout wake");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(8)),
        ))
        .expect("clock should advance past timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should promote after the clock advance");
    let timeout_report = runtime
        .admit_resource_timeout(pending.handle(), ready_timeout)
        .expect("timeout admission should succeed");

    assert_eq!(
        timeout_report
            .lifecycle()
            .expect("admitted timeout should report lifecycle truth")
            .output_continuity(),
        ResourceOutputContinuity::PriorOutputPreserved
    );
    assert_eq!(
        timeout_report
            .performance()
            .output_continuity_classification_width(),
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_timeout + 1
    );
}

#[test]
fn resource_timeout_without_prior_output_does_not_charge_terminal_visibility_classification() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");

    let pending = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("first request should admit")
        .admitted_request();
    let decisions_before_timeout = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;
    let timeout_wake = runtime
        .in_flight_resource_request(pending.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("pending request should retain timeout wake");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(8)),
        ))
        .expect("clock should advance past timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should promote");
    let timeout_report = runtime
        .admit_resource_timeout(pending.handle(), ready_timeout)
        .expect("timeout admission should succeed");

    assert_eq!(
        timeout_report
            .lifecycle()
            .expect("admitted timeout should report lifecycle truth")
            .output_continuity(),
        ResourceOutputContinuity::NoPriorOutput
    );
    assert_eq!(
        timeout_report
            .performance()
            .output_continuity_classification_width(),
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_timeout
    );
}

#[test]
fn resource_cancellation_reclassifies_hidden_pending_output_when_terminal_policy_preserves() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(hide_pending_output_resource_declaration(node))
        .expect("resource declaration should lower");

    let first = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request();
    let first_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            first.handle(),
            first.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("initial completion should admit");
    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staging = tx.stage_admitted_resource_completion(first_completion)?;
            tx.commit_staged_resource_completion(staging.staged_effect())?;
            Ok(())
        })
        .expect("completion transaction should commit");

    let pending_report = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("follow-up request should admit");
    let pending = pending_report.admitted_request();
    let decisions_before_cancellation = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;

    let cancellation_report = runtime
        .cancel_resource_request(pending.handle(), ResourceCancellationReason::HostRequested)
        .expect("cancellation should succeed");

    assert_eq!(
        cancellation_report
            .lifecycle()
            .expect("admitted cancellation should report lifecycle truth")
            .output_continuity(),
        ResourceOutputContinuity::PriorOutputPreserved
    );
    assert_eq!(
        cancellation_report
            .performance()
            .output_continuity_classification_width(),
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_cancellation + 1
    );
}

#[test]
fn resource_cancellation_without_prior_output_does_not_charge_terminal_visibility_classification() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retain_all_transitions_resource_declaration(node))
        .expect("resource declaration should lower");

    let pending = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("first request should admit")
        .admitted_request();
    let decisions_before_cancellation = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;

    let cancellation_report = runtime
        .cancel_resource_request(pending.handle(), ResourceCancellationReason::HostRequested)
        .expect("cancellation should succeed");

    assert_eq!(
        cancellation_report
            .lifecycle()
            .expect("admitted cancellation should report lifecycle truth")
            .output_continuity(),
        ResourceOutputContinuity::NoPriorOutput
    );
    assert_eq!(
        cancellation_report
            .performance()
            .output_continuity_classification_width(),
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_cancellation
    );
}

#[test]
fn resource_timeout_visibility_hide_and_preserve_share_lifecycle_but_not_visibility_digest() {
    fn drive_timeout_visibility(
        hide_after_timeout: bool,
    ) -> (
        ResourceTimeoutReport,
        ResourceReplayReconstructionReport,
        TestRuntime,
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

        let mut graph = SignalGraph::new();
        let node = graph.node().build();
        let mut runtime = TestRuntime::build(graph);
        let declaration = if hide_after_timeout {
            hide_after_timeout_resource_declaration(node)
        } else {
            resource_declaration(node)
        }
        .with_timeout_policy(ResourceTimeoutPolicyDeclaration::FixedTimeout {
            timeout: TemporalDuration::temporal_duration(5).unwrap(),
        });
        runtime
            .declare_resource_node(declaration)
            .expect("resource declaration should lower");

        let first = runtime
            .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
            .expect("initial request should admit")
            .admitted_request();
        let first_completion = runtime
            .admit_resource_completion(raw_completion(
                &runtime,
                node,
                first.handle(),
                first.attempt(),
                64,
            ))
            .admitted_completion()
            .expect("initial completion should admit");
        let mut ctx = ();
        runtime
            .transaction(request_execution, &mut ctx, |tx| {
                let staging = tx.stage_admitted_resource_completion(first_completion)?;
                tx.commit_staged_resource_completion(staging.staged_effect())?;
                Ok(())
            })
            .expect("completion transaction should commit");

        let pending = runtime
            .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
            .expect("follow-up request should admit")
            .admitted_request();
        let timeout_wake = runtime
            .in_flight_resource_request(pending.handle())
            .and_then(|in_flight| in_flight.timeout_wake_id())
            .expect("pending request should retain timeout wake");
        runtime
            .advance_clock(ClockAdvanceRequest::new(
                ClockDomain::MonotonicExecution,
                ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(8)),
            ))
            .expect("clock should advance past timeout");
        let ready_timeout = runtime
            .promote_temporal_wake_ready(timeout_wake)
            .expect("timeout wake should promote");
        let timeout_report = runtime
            .admit_resource_timeout(pending.handle(), ready_timeout)
            .expect("timeout admission should succeed");
        let replay = runtime.reconstruct_resource_replay_summary();
        (timeout_report, replay, runtime)
    }

    let (preserve_report, preserve_replay, _) = drive_timeout_visibility(false);
    let (hide_report, hide_replay, hide_runtime) = drive_timeout_visibility(true);

    assert_eq!(
        preserve_report
            .lifecycle()
            .expect("preserve timeout should admit")
            .lifecycle(),
        ResourceLifecycleClass::TimedOut
    );
    assert_eq!(
        hide_report
            .lifecycle()
            .expect("hide timeout should admit")
            .lifecycle(),
        ResourceLifecycleClass::TimedOut
    );
    assert_eq!(
        preserve_replay.lifecycle_digest(),
        hide_replay.lifecycle_digest()
    );
    assert_ne!(
        preserve_replay.output_continuity_digest(),
        hide_replay.output_continuity_digest()
    );
    assert_eq!(
        hide_report
            .lifecycle()
            .expect("hide timeout should retain lifecycle")
            .output_continuity(),
        ResourceOutputContinuity::OutputUnavailableByPolicy
    );
    assert_eq!(
        hide_runtime
            .telemetry()
            .resource
            .resource_previous_output_hidden_count,
        1
    );
}
