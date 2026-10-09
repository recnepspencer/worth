use crate::facade::*;
use crate::tests::async_node_support::{
    async_node_capability_declaration, AsyncNodeTestRuntime as TestRuntime,
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
fn async_node_historical_parity_report_matches_legacy_resource_truth() {
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
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("async capability should attach");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [node],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let admitted_request = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("request should admit through attached capability handle")
        .resource_admission()
        .expect("request admission should lower into resource admission")
        .admitted_request();
    let raw_completion = RawCompletionEnvelope::new(
        admitted_request.handle().request_id(),
        admitted_request.handle().generation(),
        admitted_request.handle().branch_epoch(),
        admitted_request.attempt(),
        attached.payload_contract_digest().clone(),
        64,
    );
    let admitted_completion = runtime
        .admit_resource_completion(raw_completion)
        .admitted_completion()
        .expect("completion should be admitted");
    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staged = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staged.staged_effect())?;
            Ok(())
        })
        .expect("completion should commit and materialize observation lineage");
    let budget = ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX);
    let parity = runtime
        .async_node_historical_parity_report(&attached, budget)
        .expect("historical parity report should materialize for current handle");
    let direct_replay = runtime.reconstruct_resource_replay_summary();
    let direct_diagnostics = runtime
        .try_resource_diagnostics_summary(budget)
        .expect("direct diagnostics summary should admit under unbounded budget");
    let (direct_explanation_artifact, direct_explanation_availability) = runtime
        .graph()
        .materialize_explanation_artifact(node)
        .expect("direct explanation materialization should succeed");
    let direct_observation = runtime
        .latest_resource_observation_batch_report()
        .expect("historical parity should preserve observation lineage when present");
    let direct_explanation_summary = direct_explanation_artifact
        .as_ref()
        .map(|artifact| artifact.diagnostics_summary(runtime.runtime_policy().tier));

    assert_eq!(parity.node(), node);
    assert_eq!(parity.registry_digest(), attached.registry_digest());
    assert_eq!(parity.bundle_digest(), attached.bundle_digest());
    assert_eq!(
        parity.payload_contract_digest(),
        attached.payload_contract_digest()
    );
    assert_eq!(
        parity.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeHistoricalParity
    );
    assert_eq!(
        parity.replay_reconstruction().replay_digest(),
        direct_replay.replay_digest()
    );
    assert_eq!(
        parity.replay_reconstruction().lifecycle_digest(),
        direct_replay.lifecycle_digest()
    );
    assert_eq!(
        parity
            .observation_batch_report()
            .expect("historical parity should carry observation lineage")
            .events()
            .len(),
        direct_observation.events().len()
    );
    assert_eq!(
        parity
            .observation_batch_report()
            .expect("historical parity should carry observation lineage")
            .performance(),
        direct_observation.performance()
    );
    assert_eq!(
        parity.explanation_availability(),
        direct_explanation_availability
    );
    assert_eq!(
        parity.explanation_summary(),
        direct_explanation_summary.as_ref()
    );
    assert_eq!(
        parity
            .diagnostics_summary()
            .expect("historical parity should carry diagnostics summary")
            .provenance_digest(),
        direct_diagnostics.provenance_digest()
    );
    assert!(parity.diagnostics_denial().is_none());
    assert!(parity.branch_restore_report().is_none());
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_node_historical_parity_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_node_capability_broad_scan_denial_count,
        0
    );
}

#[test]
fn async_node_historical_parity_report_carries_diagnostics_denial_without_losing_replay_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("async capability should attach");

    runtime
        .admit_async_node_request(attached.request_intent())
        .expect("request should admit through attached capability handle");

    let budget = ResourceDiagnosticsExpansionBudget::retained_summary_only();
    let parity = runtime
        .async_node_historical_parity_report(&attached, budget)
        .expect("historical parity should still materialize when diagnostics richness denies");
    let direct_replay = runtime.reconstruct_resource_replay_summary();
    let (direct_explanation_artifact, direct_explanation_availability) = runtime
        .graph()
        .materialize_explanation_artifact(node)
        .expect("direct explanation materialization should still reflect active runtime policy");
    let direct_explanation_summary = direct_explanation_artifact
        .as_ref()
        .map(|artifact| artifact.diagnostics_summary(runtime.runtime_policy().tier));

    assert_eq!(
        parity.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeHistoricalParity
    );
    assert_eq!(
        parity.explanation_availability(),
        direct_explanation_availability
    );
    assert_eq!(
        parity.explanation_summary(),
        direct_explanation_summary.as_ref()
    );
    assert!(parity.diagnostics_summary().is_none());
    assert_eq!(
        parity
            .diagnostics_denial()
            .expect("historical parity should carry diagnostics denial")
            .class(),
        ResourceDiagnosticsExpansionDenialClass::ColdReconstructionDisabled
    );
    assert_eq!(
        parity
            .diagnostics_denial()
            .expect("historical parity should carry diagnostics denial")
            .budget(),
        budget
    );
    assert_eq!(
        parity.replay_reconstruction().replay_digest(),
        direct_replay.replay_digest()
    );
    assert_eq!(
        parity.replay_reconstruction().lifecycle_digest(),
        direct_replay.lifecycle_digest()
    );
}

#[test]
fn async_node_historical_parity_report_carries_explanation_denial_without_losing_replay_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("async capability should attach");

    runtime
        .admit_async_node_request(attached.request_intent())
        .expect("request should admit through attached capability handle");
    runtime.adjust_runtime_policy(|mut policy| {
        policy
            .reconstruction_budget
            .allow_explanation_reconstruction = false;
        policy
    });

    let parity = runtime
        .async_node_historical_parity_report(
            &attached,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("historical parity should still materialize when explanation richness denies");
    let direct_replay = runtime.reconstruct_resource_replay_summary();

    assert_eq!(
        parity.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeHistoricalParity
    );
    assert_eq!(
        parity.explanation_availability(),
        DiagnosticsAvailability::DeniedByBudget
    );
    assert!(parity.explanation_summary().is_none());
    assert_eq!(
        parity.replay_reconstruction().replay_digest(),
        direct_replay.replay_digest()
    );
    assert_eq!(
        parity.replay_reconstruction().lifecycle_digest(),
        direct_replay.lifecycle_digest()
    );
}

#[test]
fn keyed_attached_async_node_historical_parity_still_uses_runtime_owned_truth() {
    let mut runtime = TestRuntime::build(SignalGraph::new());
    let family = define_keyed_computation(&mut runtime, "async-history", ());
    let keyed = family.keyed("left-wing");
    let attached = keyed
        .attach_async_capability(
            &mut runtime,
            AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(11))
                .with_max_payload_bytes(2048),
        )
        .expect("keyed computation should attach async capability");

    runtime
        .admit_async_node_request(attached.request_intent())
        .expect("keyed attached capability should admit request");

    let parity = runtime
        .async_node_historical_parity_report(
            &attached,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("keyed historical parity should materialize");

    assert_eq!(parity.node(), attached.node());
    assert_eq!(parity.bundle_digest(), attached.bundle_digest());
    assert!(parity.diagnostics_summary().is_some());
}

#[test]
fn async_node_historical_parity_report_rejects_stale_handle_after_restore() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let attached = runtime
        .attach_async_capability(async_node_capability_declaration(node))
        .expect("async capability should attach after snapshot");

    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should erase post-snapshot capability attachment");

    let denial = runtime
        .async_node_historical_parity_report(
            &attached,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("restored-away capability handle should deny historical parity");

    assert_eq!(denial.node(), node);
    assert_eq!(
        denial.denial_class(),
        AsyncNodeHistoricalParityDenialClass::UndeclaredCapability
    );
    assert_eq!(
        denial.performance().boundary(),
        ResourceBoundaryKind::AsyncNodeHistoricalParity
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .async_node_capability_broad_scan_denial_count,
        1
    );
}
