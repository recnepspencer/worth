use crate::facade::*;
use crate::tests::async_node_support::{
    async_node_capability_declaration, async_node_capability_with_dependents,
    settle_async_dependency_baseline, AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::define_keyed_computation;

struct NoopAsyncNodeObservationListener;

impl ObservationListener<(), (), (), (), ()> for NoopAsyncNodeObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

#[test]
fn async_keyed_node_historical_parity_report_preserves_family_identity_and_runtime_truth() {
    let mut runtime = TestRuntime::build(SignalGraph::new());
    let family = define_keyed_computation(&mut runtime, "async-closeout", ());
    let keyed = family.keyed("left-wing");
    let payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(41))
        .with_max_payload_bytes(2048);
    let binding = keyed
        .declare_async_capability(&mut runtime, payload.clone())
        .expect("keyed node should declare async capability");
    let handle = keyed
        .async_capable_node(&mut runtime)
        .expect("declared keyed node should surface capability handle");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [binding.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let admitted_request = runtime
        .admit_async_node_request(handle.request_intent())
        .expect("keyed request should admit")
        .resource_admission()
        .expect("keyed async request should lower into resource admission")
        .admitted_request();
    let admitted_completion = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            admitted_request.handle().request_id(),
            admitted_request.handle().generation(),
            admitted_request.handle().branch_epoch(),
            admitted_request.attempt(),
            handle.payload_contract_digest().clone(),
            96,
        ))
        .admitted_completion()
        .expect("completion should admit");
    let mut ctx = ();
    runtime
        .transaction(&mut ctx, |tx| {
            let staged = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staged.staged_effect())?;
            Ok(())
        })
        .expect("completion should commit");

    let report = runtime
        .async_keyed_node_historical_parity_report(
            &binding,
            &handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("keyed historical parity report should materialize");

    assert_eq!(report.family(), family.family());
    assert_eq!(report.key(), keyed.key());
    assert_eq!(report.node(), binding.node());
    assert_eq!(
        report
            .historical_parity_report()
            .replay_reconstruction()
            .lifecycle_digest(),
        runtime
            .reconstruct_resource_replay_summary()
            .lifecycle_digest()
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::AsyncKeyedNodeHistoricalParity
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_keyed_node_historical_parity_count,
        1
    );
}

#[test]
fn async_keyed_node_capability_equivalence_report_matches_legacy_runtime_truth() {
    let mut runtime = TestRuntime::build(SignalGraph::new());
    let family = define_keyed_computation(&mut runtime, "async-eq-closeout", ());
    let keyed = family.keyed("left-wing");
    let payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(42))
        .with_max_payload_bytes(1024);
    let declaration = keyed.async_capability_declaration(&mut runtime, payload.clone());
    let binding = keyed
        .declare_async_capability(&mut runtime, payload)
        .expect("keyed node should declare async capability");
    let handle = keyed
        .async_capable_node(&mut runtime)
        .expect("declared keyed node should surface capability handle");

    let report = runtime
        .async_keyed_node_capability_equivalence_report(
            &binding,
            &handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("keyed capability equivalence should materialize");

    assert_eq!(report.family(), family.family());
    assert_eq!(report.key(), keyed.key());
    assert_eq!(
        report.equivalence_report().capability_declaration_digest(),
        canonical_digest(&declaration)
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::AsyncKeyedNodeCapabilityEquivalence
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_keyed_node_capability_equivalence_count,
        1
    );
}

#[test]
fn async_keyed_node_closeout_reports_reject_binding_handle_mismatch() {
    let mut runtime = TestRuntime::build(SignalGraph::new());
    let family = define_keyed_computation(&mut runtime, "async-closeout-mismatch", ());
    let left = family.keyed("left");
    let right = family.keyed("right");
    let payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(43));
    let left_binding = left
        .declare_async_capability(&mut runtime, payload.clone())
        .expect("left should declare async capability");
    right
        .declare_async_capability(&mut runtime, payload)
        .expect("right should declare async capability");
    let right_handle = right
        .async_capable_node(&mut runtime)
        .expect("right handle should exist");

    let denial = runtime
        .async_keyed_node_historical_parity_report(
            &left_binding,
            &right_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("binding/handle mismatch must deny");
    assert_eq!(
        denial.denial_class(),
        AsyncKeyedNodeHistoricalParityDenialClass::BindingHandleNodeMismatch
    );

    let declaration = right.async_capability_declaration(
        &mut runtime,
        AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(44)),
    );
    let eq_denial = runtime
        .async_keyed_node_capability_equivalence_report(
            &left_binding,
            &right_handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("binding/handle mismatch must deny equivalence");
    assert_eq!(
        eq_denial.denial_class(),
        AsyncKeyedNodeCapabilityEquivalenceDenialClass::BindingHandleNodeMismatch
    );
}

#[test]
fn async_node_hierarchy_historical_parity_report_preserves_restore_honesty() {
    let mut graph = SignalGraph::new();
    let parent = graph.node().build();
    let child = graph.node().build();
    let grandchild = graph.node().build();
    graph
        .append_dependency(child, parent, Aspect::new(0))
        .expect("child should depend on parent");
    graph
        .append_dependency(grandchild, child, Aspect::new(0))
        .expect("grandchild should depend on child");
    settle_async_dependency_baseline(&mut graph, [parent, child, grandchild]);

    let mut runtime = TestRuntime::build(graph);
    let parent_handle = runtime
        .attach_async_capability(async_node_capability_with_dependents(parent, [child]))
        .expect("parent capability should attach");
    let child_handle = runtime
        .attach_async_capability(async_node_capability_with_dependents(child, [grandchild]))
        .expect("child capability should attach");
    let grandchild_handle = runtime
        .attach_async_capability(async_node_capability_declaration(grandchild))
        .expect("grandchild capability should attach");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [parent],
        Box::new(NoopAsyncNodeObservationListener),
    );
    runtime
        .admit_async_node_request(grandchild_handle.request_intent())
        .expect("grandchild request should admit");
    let child_request = runtime
        .admit_async_node_request(child_handle.request_intent())
        .expect("child request should admit")
        .resource_admission()
        .expect("child request should lower into resource admission")
        .admitted_request();
    let parent_request = runtime
        .admit_async_node_request(parent_handle.request_intent())
        .expect("parent request should admit")
        .resource_admission()
        .expect("parent request should lower into resource admission")
        .admitted_request();
    let admitted_completion = runtime
        .admit_resource_completion(RawCompletionEnvelope::new(
            parent_request.handle().request_id(),
            parent_request.handle().generation(),
            parent_request.handle().branch_epoch(),
            parent_request.attempt(),
            parent_handle.payload_contract_digest().clone(),
            128,
        ))
        .admitted_completion()
        .expect("parent completion should admit");
    let mut ctx = ();
    runtime
        .transaction(&mut ctx, |tx| {
            let staged = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staged.staged_effect())?;
            Ok(())
        })
        .expect("parent completion should commit");

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let baseline = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("hierarchy historical parity should materialize");
    runtime
        .cancel_async_node_request(
            child_request.handle(),
            ResourceCancellationReason::HostRequested,
        )
        .expect("child request should cancel to create post-snapshot drift");
    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should reinstate hierarchy truth");
    let restored = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored hierarchy historical parity should materialize");

    assert_eq!(
        restored.hierarchy_replay_summary().replay_digest(),
        baseline.hierarchy_replay_summary().replay_digest()
    );
    assert_eq!(
        restored
            .historical_parity_report()
            .replay_reconstruction()
            .replay_digest(),
        baseline
            .historical_parity_report()
            .replay_reconstruction()
            .replay_digest()
    );
    assert_eq!(
        restored
            .historical_parity_report()
            .explanation_availability(),
        baseline
            .historical_parity_report()
            .explanation_availability()
    );
    assert_eq!(
        restored
            .historical_parity_report()
            .observation_batch_report()
            .map(|report| report.performance()),
        baseline
            .historical_parity_report()
            .observation_batch_report()
            .map(|report| report.performance())
    );
    assert!(restored
        .historical_parity_report()
        .branch_restore_report()
        .is_some());
    assert_eq!(
        restored.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeHierarchyHistoricalParity
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_node_hierarchy_historical_parity_count,
        1
    );
}

#[test]
fn async_node_hierarchy_historical_parity_report_rejects_non_hierarchical_root() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let handle = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("leaf capability should attach");

    let denial = runtime
        .async_node_hierarchy_historical_parity_report(
            &handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("leaf node should not certify hierarchy historical parity");

    assert_eq!(
        denial.denial_class(),
        AsyncNodeHierarchyHistoricalParityDenialClass::NotHierarchicalRoot
    );
}
