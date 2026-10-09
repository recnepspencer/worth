use crate::facade::{
    Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain, ClockTick, NodeEvaluationResult,
    OutputIdentity, PreviousValueRevision, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy, TemporalCondition, TemporalWakeRetirementReason,
};

#[test]
fn ready_temporal_wake_grants_previous_value_access_and_captures_committed_state() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();
    let value_aspect = Aspect::new(0);

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        value_aspect,
                        7,
                    )]))
                    .with_output_identity("baseline-value"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(2).unwrap(), ClockTick::new(2))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();
    let reconstructed_reads_before = runtime
        .observe()
        .metrics()
        .storage
        .reconstructed_artifact_read_count;
    let reference = runtime.previous_temporal_value(&access, source).unwrap();

    assert_eq!(access.wake_id(), ready.id());
    assert_eq!(access.branch_id(), runtime.current_branch().id);
    assert_eq!(reference.revision(), PreviousValueRevision::new(1));
    assert_eq!(reference.branch_id(), runtime.current_branch().id);
    assert_eq!(reference.access_wake_id(), ready.id());
    assert_eq!(reference.node(), source);
    assert_eq!(reference.captured_at_tick(), ClockTick::new(2));
    assert_eq!(reference.aspect_version().get(value_aspect), 7);
    assert_eq!(
        reference.output_identity().map(OutputIdentity::as_str),
        Some("baseline-value")
    );
    assert_eq!(
        runtime.telemetry().temporal.previous_value_reference_count,
        1
    );
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .storage
            .reconstructed_artifact_read_count,
        reconstructed_reads_before
    );
}

#[test]
fn on_demand_previous_value_access_does_not_capture_optional_telemetry() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let source = runtime.graph_mut().node().build();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        Aspect::new(0),
                        7,
                    )]))
                    .with_output_identity("on-demand-previous-value"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(2).unwrap(), ClockTick::new(2))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();

    let reference = runtime.previous_temporal_value(&access, source).unwrap();

    assert_eq!(reference.revision(), PreviousValueRevision::new(1));
    assert_eq!(reference.branch_id(), runtime.current_branch().id);
    assert_eq!(reference.access_wake_id(), ready.id());
    assert_eq!(reference.node(), source);
    assert_eq!(reference.captured_at_tick(), ClockTick::new(2));
    assert_eq!(reference.aspect_version().get(Aspect::new(0)), 7);
    assert_eq!(
        reference.output_identity().map(OutputIdentity::as_str),
        Some("on-demand-previous-value")
    );

    assert_eq!(
        runtime.telemetry().temporal.previous_value_reference_count,
        0,
        "OnDemand previous-value access must not write optional telemetry"
    );
}

#[test]
fn previous_value_access_is_rejected_after_wake_retirement() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();
    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(1).unwrap(), ClockTick::new(1))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();
    runtime
        .retire_temporal_wake(ready.id(), TemporalWakeRetirementReason::Consumed)
        .unwrap();

    let err = runtime
        .previous_temporal_value(&access, source)
        .unwrap_err();
    assert!(
        format!("{err}").contains("inactive temporal access wake"),
        "retired wakes must not keep previous-value access alive"
    );
}

#[test]
fn previous_value_access_is_branch_scoped() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        Aspect::new(0),
                        5,
                    )]))
                    .with_output_identity("main-branch"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(1).unwrap(), ClockTick::new(1))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();

    let feature = runtime.create_branch("feature-previous-value").unwrap();
    runtime.switch_branch(feature).unwrap();

    let err = runtime
        .previous_temporal_value(&access, source)
        .unwrap_err();
    assert!(
        format!("{err}").contains("belongs to branch"),
        "previous-value capabilities must not cross branch boundaries"
    );
}

#[test]
fn previous_value_access_is_rejected_after_restore_epoch_changes() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        Aspect::new(0),
                        5,
                    )]))
                    .with_output_identity("restore-epoch"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(1).unwrap(), ClockTick::new(1))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();

    runtime.restore_snapshot(&snapshot).unwrap();

    let err = runtime
        .previous_temporal_value(&access, source)
        .unwrap_err();
    assert!(
        format!("{err}").contains("stale restore epoch"),
        "previous-value capabilities minted before restore must not revive on the restored branch"
    );
}

#[test]
fn previous_value_reads_committed_branch_truth_after_failed_transaction() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();
    let value_aspect = Aspect::new(1);

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        value_aspect,
                        11,
                    )]))
                    .with_output_identity("committed-baseline"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let _ = runtime.transaction(request_execution, &mut (), |tx| {
        tx.read(source, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                    value_aspect,
                    99,
                )]))
                .with_output_identity("staged-only"),
            ))
        })?;
        Err::<(), SignalError>(SignalError::invalid_input("force rollback"))
    });

    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(4).unwrap(), ClockTick::new(4))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(4),
        ))
        .unwrap();
    let ready = runtime.promote_temporal_wake_ready(wake.id()).unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(ready.id())
        .unwrap();
    let reference = runtime.previous_temporal_value(&access, source).unwrap();

    assert_eq!(reference.aspect_version().get(value_aspect), 11);
    assert_eq!(
        reference.output_identity().map(OutputIdentity::as_str),
        Some("committed-baseline")
    );
}
