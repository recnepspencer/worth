use super::*;

#[test]
fn resource_pending_visibility_can_preserve_prior_output_without_mutating_lifecycle() {
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
        .declare_resource_node(retain_all_transitions_resource_declaration(node))
        .expect("resource declaration should lower");

    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request();
    let admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            admitted_request.handle(),
            admitted_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("matching completion should admit");
    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staging = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staging.staged_effect())?;
            Ok(())
        })
        .expect("completion transaction should commit");

    let report = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("follow-up request should admit");

    assert_eq!(
        report.lifecycle().lifecycle(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        report.lifecycle().output_continuity(),
        ResourceOutputContinuity::PriorOutputPreserved
    );
    assert_eq!(
        report.transition().output_continuity(),
        ResourceOutputContinuity::PriorOutputPreserved
    );
    assert_eq!(
        report
            .performance()
            .output_continuity_classification_width(),
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_previous_output_preserved_count,
        1
    );
}

#[test]
fn resource_pending_visibility_hide_and_preserve_share_lifecycle_but_not_visibility_digest() {
    fn drive_pending_visibility(
        hide_while_pending: bool,
    ) -> (
        ResourceRequestAdmissionReport,
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
        runtime
            .declare_resource_node(if hide_while_pending {
                hide_pending_output_resource_declaration(node)
            } else {
                resource_declaration(node)
            })
            .expect("resource declaration should lower");

        let admitted_request = runtime
            .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
            .expect("initial request should admit")
            .admitted_request();
        let admitted_completion = runtime
            .admit_resource_completion(raw_completion(
                &runtime,
                node,
                admitted_request.handle(),
                admitted_request.attempt(),
                64,
            ))
            .admitted_completion()
            .expect("matching completion should admit");
        let mut ctx = ();
        runtime
            .transaction(request_execution, &mut ctx, |tx| {
                let staging = tx.stage_admitted_resource_completion(admitted_completion)?;
                tx.commit_staged_resource_completion(staging.staged_effect())?;
                Ok(())
            })
            .expect("completion transaction should commit");

        let report = runtime
            .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
            .expect("follow-up request should admit");
        let replay = runtime.reconstruct_resource_replay_summary();
        (report, replay, runtime)
    }

    let (preserve_report, preserve_replay, _) = drive_pending_visibility(false);
    let (hide_report, hide_replay, hide_runtime) = drive_pending_visibility(true);

    assert_eq!(
        preserve_report.lifecycle().lifecycle(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        hide_report.lifecycle().lifecycle(),
        ResourceLifecycleClass::Pending
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
        hide_report.lifecycle().output_continuity(),
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
