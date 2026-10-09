use super::*;

#[test]
fn resource_cancellation_visibility_can_hide_previous_output() {
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
        .declare_resource_node(hide_after_cancellation_resource_declaration(node))
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
    let cancellation_report = runtime
        .cancel_resource_request(pending.handle(), ResourceCancellationReason::HostRequested)
        .expect("cancellation should succeed");

    assert_eq!(
        cancellation_report
            .lifecycle()
            .expect("cancellation should report lifecycle")
            .output_continuity(),
        ResourceOutputContinuity::OutputUnavailableByPolicy
    );
    assert_eq!(
        cancellation_report
            .performance()
            .output_continuity_classification_width(),
        1
    );
}

#[test]
fn resource_rejection_visibility_can_hide_previous_output() {
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
        .declare_resource_node(hide_after_rejection_resource_declaration(node))
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
    let rejection_report = runtime
        .reject_resource_request(pending.handle(), ResourceRejectionReason::SemanticFailure)
        .expect("rejection should succeed");

    assert_eq!(
        rejection_report
            .lifecycle()
            .expect("rejection should report lifecycle")
            .lifecycle(),
        ResourceLifecycleClass::Rejected
    );
    assert_eq!(
        rejection_report
            .lifecycle()
            .expect("rejection should report lifecycle")
            .output_continuity(),
        ResourceOutputContinuity::OutputUnavailableByPolicy
    );
    assert_eq!(
        rejection_report
            .rejected_request()
            .expect("rejection should retain rejected request")
            .reason(),
        ResourceRejectionReason::SemanticFailure
    );
    assert_eq!(
        rejection_report
            .performance()
            .output_continuity_classification_width(),
        1
    );
}

#[test]
fn resource_rejection_without_prior_output_does_not_charge_terminal_visibility_classification() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(resource_declaration(node))
        .expect("resource declaration should lower");

    let pending = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("first request should admit")
        .admitted_request();
    let decisions_before_rejection = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;

    let rejection_report = runtime
        .reject_resource_request(pending.handle(), ResourceRejectionReason::HostFailure)
        .expect("rejection should succeed");

    assert_eq!(
        rejection_report
            .lifecycle()
            .expect("rejection should report lifecycle")
            .output_continuity(),
        ResourceOutputContinuity::NoPriorOutput
    );
    assert_eq!(
        rejection_report
            .performance()
            .output_continuity_classification_width(),
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_rejection
    );
}

#[test]
fn resource_supersession_visibility_policy_can_hide_previous_output() {
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
        .declare_resource_node(hide_after_supersession_resource_declaration(node))
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

    runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("pending follow-up should admit");
    let third_report = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("third request should supersede active pending request");

    assert_eq!(
        third_report.lifecycle().output_continuity(),
        ResourceOutputContinuity::PriorOutputPreserved
    );
    assert_eq!(
        third_report
            .supersession_record()
            .expect("fresh request should retain supersession record")
            .lifecycle_transition()
            .output_continuity(),
        ResourceOutputContinuity::OutputUnavailableByPolicy
    );
    assert_eq!(
        third_report
            .performance()
            .output_continuity_classification_width(),
        2
    );
}

#[test]
fn resource_supersession_without_prior_output_counts_only_pending_visibility_classification() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(hide_after_supersession_resource_declaration(node))
        .expect("resource declaration should lower");

    let first = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("first request should admit");
    let decisions_before_supersession = runtime
        .telemetry()
        .resource
        .resource_output_continuity_decision_count;

    let second = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("second request should supersede the active pending request");

    assert_eq!(
        second.lifecycle().output_continuity(),
        ResourceOutputContinuity::NoPriorOutput
    );
    assert_eq!(
        second
            .supersession_record()
            .expect("supersession should be retained")
            .lifecycle_transition()
            .output_continuity(),
        ResourceOutputContinuity::NoPriorOutput
    );
    assert_eq!(
        second
            .performance()
            .output_continuity_classification_width(),
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_output_continuity_decision_count,
        decisions_before_supersession + 1
    );
    assert_eq!(
        first.admitted_request().handle(),
        second
            .superseded_request()
            .expect("second request should supersede the first")
    );
}
