use super::*;

#[test]
fn resource_observation_batch_report_can_include_denied_completion_without_applying_it() {
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
        .declare_resource_node(denied_completion_observation_resource_declaration(node))
        .expect("denied-completion observation declaration should lower");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();

    let denied = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            admitted.handle().request_id(),
            admitted.handle().generation(),
            admitted.handle().branch_epoch(),
            admitted.attempt(),
            ResourcePayloadContractDigest::new("payload-contract:999:1024"),
            64,
        ))
        .denied_completion()
        .expect("wrong payload contract should deny without apply");
    assert_eq!(denied.class(), CompletionDenialClass::Malformed);

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("resource observation should materialize denied completion evidence");
    let observed = &report.events()[0].matched_resource_nodes()[0];

    assert_eq!(observed.lifecycle(), ResourceLifecycleClass::Pending);
    assert_eq!(
        observed.output_continuity(),
        Some(ResourceOutputContinuity::NoPriorOutput)
    );
    assert_eq!(
        observed
            .denied_completion()
            .expect("policy should surface denied completion evidence")
            .class(),
        CompletionDenialClass::Malformed
    );
    assert!(observed.scheduled_retry().is_none());
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_denied_completion_observation_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_schedule_observation_count,
        0
    );

    let admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            admitted.handle(),
            admitted.attempt(),
            64,
        ))
        .admitted_completion();
    assert!(
        admitted_completion.is_some(),
        "denied-completion observation must not spend or poison real completion authority"
    );
}

#[test]
fn resource_observation_clears_stale_denied_completion_after_authoritative_progress() {
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
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(denied_completion_observation_resource_declaration(node))
        .expect("denied-completion observation declaration should lower");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();

    let denied = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            admitted.handle().request_id(),
            admitted.handle().generation(),
            admitted.handle().branch_epoch(),
            admitted.attempt(),
            ResourcePayloadContractDigest::new("payload-contract:999:1024"),
            64,
        ))
        .denied_completion()
        .expect("wrong payload contract should deny");
    assert_eq!(denied.class(), CompletionDenialClass::Malformed);

    let admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            admitted.handle(),
            admitted.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("valid completion should still admit afterward");

    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staging = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staging.staged_effect())?;
            Ok(())
        })
        .expect("valid completion should commit");

    mark_dirty(runtime.graph_mut(), source, Aspect::new(0))
        .expect("dependency invalidation should create a fresh observation boundary");
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("resource observation should materialize current state");
    let observed = &report.events()[0].matched_resource_nodes()[0];
    assert_eq!(observed.lifecycle(), ResourceLifecycleClass::Fulfilled);
    assert!(
        observed.denied_completion().is_none(),
        "fulfilled observation must not leak stale denied-completion evidence"
    );
}

#[test]
fn resource_observation_batch_report_can_include_retry_schedule_without_retry_apply() {
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
        .declare_resource_node(retry_schedule_observation_resource_declaration(node))
        .expect("retry-schedule observation declaration should lower");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );

    let scheduled = schedule_timed_out_retry(&mut runtime, node)
        .scheduled_retry()
        .cloned()
        .expect("timed out request should schedule retry");

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("resource observation should materialize scheduled retry evidence");
    let observed = &report.events()[0].matched_resource_nodes()[0];

    assert_eq!(observed.lifecycle(), ResourceLifecycleClass::TimedOut);
    assert_eq!(
        observed
            .scheduled_retry()
            .expect("policy should surface scheduled retry evidence")
            .retry_ordinal(),
        scheduled.retry_ordinal()
    );
    assert_eq!(
        observed
            .scheduled_retry()
            .expect("policy should retain scheduled retry reason")
            .reason(),
        ResourceRetryReason::TimedOut
    );
    assert!(observed.denied_completion().is_none());
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_denied_completion_observation_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_retry_schedule_observation_count,
        1
    );

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(
                runtime
                    .clock_basis()
                    .current_tick()
                    .get()
                    .saturating_add(scheduled.scheduled_delay().get()),
            ),
        ))
        .expect("clock should reach scheduled retry backoff after observation");
    let ready_retry = runtime
        .promote_temporal_wake_ready(scheduled.backoff_wake_id())
        .expect("observation must not consume scheduled retry wake");
    let retry_admission = runtime
        .admit_scheduled_resource_retry(scheduled.previous(), ready_retry)
        .expect("observation must not apply or block the scheduled retry");
    let admitted_retry = retry_admission
        .admitted_retry()
        .expect("ready retry should still admit after observation materialization");
    assert_eq!(
        admitted_retry.scheduled().retry_ordinal(),
        scheduled.retry_ordinal()
    );
}

#[test]
fn resource_observation_clears_superseded_retry_schedule_when_fresh_request_admits() {
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
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(retry_schedule_observation_resource_declaration(node))
        .expect("retry-schedule observation declaration should lower");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );

    let scheduled = schedule_timed_out_retry(&mut runtime, node)
        .scheduled_retry()
        .cloned()
        .expect("timed out request should schedule retry");

    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("fresh request should supersede the old timed-out lineage");
    assert!(
        runtime
            .promote_temporal_wake_ready(scheduled.backoff_wake_id())
            .is_err(),
        "superseded retry wake should be retired before it can promote"
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("resource observation should materialize current state");
    let observed = &report.events()[0].matched_resource_nodes()[0];

    assert_eq!(observed.lifecycle(), ResourceLifecycleClass::Pending);
    assert!(
        observed.scheduled_retry().is_none(),
        "fresh request observation must not leak superseded retry schedule"
    );
}
