use crate::facade::*;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

fn drive_interior_gate_timeout_visibility(
    hide_after_timeout: bool,
) -> (
    AsyncNodeGateStateReport,
    ResourceReplayReconstructionReport,
    TestRuntime,
) {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let gate = graph.node().build();
    let sink = graph.node().build();
    graph
        .append_dependency(gate, source, ASPECT_A)
        .expect("source should feed gate");
    graph
        .append_dependency(sink, gate, ASPECT_A)
        .expect("gate should feed sink");

    let mut source_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut gate_eval = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0)).with_output_identity("gate-v3"))
    };
    let mut sink_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(3, 0));
    evaluate(&mut graph, source, &mut source_eval).expect("source should evaluate");
    evaluate(&mut graph, gate, &mut gate_eval).expect("gate should evaluate");
    evaluate(&mut graph, sink, &mut sink_eval).expect("sink should evaluate");

    let mut runtime = TestRuntime::build(graph);
    let declaration = async_node_capability_declaration(gate)
        .with_timeout_policy(ResourceTimeoutPolicyDeclaration::FixedTimeout {
            timeout: TemporalDuration::temporal_duration(5).unwrap(),
        })
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput);
    let declaration = if hide_after_timeout {
        declaration.with_output_continuity_policy(
            ResourceOutputContinuityPolicyDeclaration::HideAfterTimeout,
        )
    } else {
        declaration
    };
    let attached = runtime
        .attach_async_capability(declaration)
        .expect("interior gate capability should attach");

    let first = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("first request should admit")
        .resource_admission()
        .expect("first request should expose resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        first.handle(),
        first.attempt(),
        attached.payload_contract_digest().clone(),
        64,
    );

    let pending = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("second request should admit")
        .resource_admission()
        .expect("second request should expose resource admission")
        .admitted_request();
    let timeout_wake = runtime
        .in_flight_resource_request(pending.handle())
        .and_then(|in_flight| in_flight.timeout_wake_id())
        .expect("pending gate request should retain a timeout wake");
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(runtime.clock_basis().current_tick().get().saturating_add(8)),
        ))
        .expect("clock should advance past timeout");
    let ready_timeout = runtime
        .promote_temporal_wake_ready(timeout_wake)
        .expect("timeout wake should promote");
    runtime
        .admit_resource_timeout(pending.handle(), ready_timeout)
        .expect("timeout admission should succeed");

    let state = runtime
        .async_node_gate_state_report(gate)
        .expect("timed-out gate state should materialize");
    let replay = runtime.reconstruct_resource_replay_summary();
    (state, replay, runtime)
}

#[test]
fn async_node_interior_gate_timeout_visibility_reflects_output_continuity_policy() {
    let (preserve_state, preserve_replay, _) = drive_interior_gate_timeout_visibility(false);
    let (hide_state, hide_replay, hide_runtime) = drive_interior_gate_timeout_visibility(true);

    assert_eq!(
        preserve_state.lifecycle_class(),
        ResourceLifecycleClass::TimedOut
    );
    assert_eq!(
        hide_state.lifecycle_class(),
        ResourceLifecycleClass::TimedOut
    );
    assert_eq!(preserve_state.active_request_handle(), None);
    assert_eq!(hide_state.active_request_handle(), None);
    assert_eq!(
        preserve_state.output_continuity(),
        Some(ResourceOutputContinuity::PriorOutputPreserved)
    );
    assert_eq!(
        hide_state.output_continuity(),
        Some(ResourceOutputContinuity::OutputUnavailableByPolicy)
    );
    assert_eq!(
        preserve_state
            .committed_output_identity()
            .map(OutputIdentity::as_str),
        Some("gate-v3")
    );
    assert_eq!(
        hide_state
            .committed_output_identity()
            .map(OutputIdentity::as_str),
        Some("gate-v3")
    );
    assert_eq!(
        preserve_state.downstream_dependence_facts(),
        &[
            AsyncNodeDownstreamDependenceFact::LifecycleClass,
            AsyncNodeDownstreamDependenceFact::CommittedOutput,
            AsyncNodeDownstreamDependenceFact::OutputContinuity,
            AsyncNodeDownstreamDependenceFact::ObservationBoundary,
        ]
    );
    assert_eq!(
        hide_state.downstream_dependence_facts(),
        preserve_state.downstream_dependence_facts()
    );
    assert_eq!(
        preserve_replay.lifecycle_digest(),
        hide_replay.lifecycle_digest()
    );
    assert_ne!(
        preserve_replay.output_continuity_digest(),
        hide_replay.output_continuity_digest(),
        "timeout visibility policy drift must perturb continuity truth without forging a different lifecycle story"
    );
    assert_ne!(
        preserve_state.gate_digest(),
        hide_state.gate_digest(),
        "timeout visibility policy drift must perturb gate truth"
    );
    assert!(
        hide_runtime
            .telemetry()
            .resource
            .resource_previous_output_hidden_count
            >= 1,
        "hide-after-timeout policy should record hidden-output continuity decisions"
    );
}

#[test]
fn async_node_active_gate_legality_drift_revalidates_without_new_lineage_and_replays_after_restore()
{
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
    let gate = graph
        .node()
        .aspect_filter(AspectMask::from_aspect(Aspect::new(0)))
        .build();
    let sink = graph.node().build();
    graph
        .append_dependency(gate, source, Aspect::new(1))
        .expect("source should feed gate with a non-matching aspect");
    graph
        .append_dependency(sink, gate, Aspect::new(0))
        .expect("gate should feed sink");

    let mut source_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 1));
    let mut gate_eval = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("gate-v4"))
    };
    let mut sink_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(2, 0));
    evaluate(&mut graph, source, &mut source_eval).expect("source should evaluate");
    evaluate(&mut graph, gate, &mut gate_eval).expect("gate should evaluate");
    evaluate(&mut graph, sink, &mut sink_eval).expect("sink should evaluate");

    let mut runtime = TestRuntime::build(graph);
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(gate))
        .expect("gate capability should attach");
    let initial = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("initial gate request should admit")
        .resource_admission()
        .expect("initial gate request should expose resource admission")
        .admitted_request();
    let baseline = runtime
        .async_node_gate_state_report(gate)
        .expect("baseline gate state should materialize");
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(source, Aspect::new(1))
        .expect("transaction-local legality drift should stage");
    let rollback = tx
        .rollback()
        .expect("rollback should erase the staged legality drift");
    assert_eq!(rollback.observation.classified_event_count, 0);

    let after_rollback = runtime
        .async_node_gate_state_report(gate)
        .expect("rollback should preserve baseline gate state");
    assert_eq!(after_rollback.gate_digest(), baseline.gate_digest());
    assert_eq!(
        runtime.resource_runtime_summary().in_flight_request_count(),
        1,
        "transaction rollback must not mint fake lifecycle churn"
    );

    mark_dirty(runtime.graph_mut(), source, Aspect::new(1))
        .expect("committed legality drift should propagate to the gate");
    evaluate(runtime.graph_mut(), source, &mut |_id, _graph| {
        Ok(version_ab(0, 2))
    })
    .expect("producer commit should resolve the gate's dependency cause");
    let first_revalidation = runtime
        .revalidate_async_node(attached.revalidation_intent())
        .expect("active gate under legality drift should still return a revalidation report");
    let first_drifted_state = runtime
        .async_node_gate_state_report(gate)
        .expect("drifted gate state should materialize");

    assert_eq!(
        first_revalidation.classification().class(),
        AsyncNodeAdmissionClass::RefreshEligibleNoNewLineage
    );
    assert_eq!(
        first_revalidation.classification().condition_block_class(),
        Some(AsyncNodeConditionBlockClass::AspectFilterMismatch)
    );
    assert_eq!(
        first_revalidation.classification().lifecycle_class(),
        ResourceLifecycleClass::Pending
    );
    assert!(
        first_revalidation.resource_revalidation().is_some(),
        "refresh-eligible gate drift should still drive runtime-owned revalidation truth"
    );
    assert_eq!(
        first_drifted_state.active_request_handle(),
        Some(initial.handle()),
        "legality drift must not mint a fresh active lineage while the pending one remains authoritative"
    );
    assert_eq!(
        first_drifted_state.lifecycle_class(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        runtime.resource_runtime_summary().in_flight_request_count(),
        1,
        "refresh-eligible drift must keep inflight width local to the existing lineage"
    );

    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should rewind to the pre-drift pending gate");
    let restored = runtime
        .async_node_gate_state_report(gate)
        .expect("restored gate state should materialize");
    assert_eq!(restored.gate_digest(), baseline.gate_digest());
    assert_eq!(
        restored
            .active_request_handle()
            .map(|handle| (handle.request_id(), handle.generation())),
        Some((initial.handle().request_id(), initial.handle().generation()))
    );

    mark_dirty(runtime.graph_mut(), source, Aspect::new(1))
        .expect("reapplied legality drift should propagate identically after restore");
    evaluate(runtime.graph_mut(), source, &mut |_id, _graph| {
        Ok(version_ab(0, 2))
    })
    .expect("restored producer should publish the same dependency cause");
    let second_revalidation = runtime
        .revalidate_async_node(attached.revalidation_intent())
        .expect("restored gate should replay the same legality story");
    let second_drifted_state = runtime
        .async_node_gate_state_report(gate)
        .expect("restored drifted gate state should materialize");

    assert_eq!(
        second_revalidation.classification().class(),
        first_revalidation.classification().class()
    );
    assert_eq!(
        second_revalidation.classification().condition_block_class(),
        first_revalidation.classification().condition_block_class()
    );
    assert_eq!(
        second_revalidation.classification().decision_digest(),
        first_revalidation.classification().decision_digest()
    );
    assert_eq!(
        second_drifted_state.gate_digest(),
        first_drifted_state.gate_digest(),
        "restore + identical upstream drift should reconstruct the same gate truth"
    );
    assert_eq!(
        second_drifted_state
            .active_request_handle()
            .map(|handle| (handle.request_id(), handle.generation())),
        first_drifted_state
            .active_request_handle()
            .map(|handle| (handle.request_id(), handle.generation()))
    );
}
