use crate::facade::*;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::{define_keyed_computation, evaluate, version_ab};

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
fn public_async_gate_rediscovery_is_branch_local_and_visibility_honest_under_restore_churn() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let gate = graph.node().build();
    let sink = graph.node().build();
    graph
        .append_dependency(gate, source, Aspect::new(0))
        .expect("source should feed gate");
    graph
        .append_dependency(sink, gate, Aspect::new(0))
        .expect("gate should feed sink");
    let mut source_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut gate_eval = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0)).with_output_identity("gate-v1"))
    };
    evaluate(&mut graph, source, &mut source_eval).expect("source should evaluate");
    evaluate(&mut graph, gate, &mut gate_eval).expect("gate should evaluate");

    let mut runtime = TestRuntime::build(graph);
    let declaration = async_node_capability_declaration(gate)
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput)
        .with_output_continuity_policy(ResourceOutputContinuityPolicyDeclaration::HideWhilePending);
    let attached = runtime
        .attach_async_capability(declaration.clone())
        .expect("gate capability should attach");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [gate],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let initial_request = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("initial gate request should admit")
        .resource_admission()
        .expect("initial gate request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        initial_request.handle(),
        initial_request.attempt(),
        attached.payload_contract_digest().clone(),
        88,
    );

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("async-public-gate-feature")
        .expect("feature branch should create");
    let sibling = runtime
        .create_branch("async-public-gate-sibling")
        .expect("sibling branch should create");

    runtime
        .switch_branch(feature.clone())
        .expect("feature branch should activate");
    let feature_handle = runtime
        .async_capable_node(gate)
        .expect("feature branch should rediscover attached gate handle");
    let feature_baseline_state = runtime
        .async_node_gate_state_report(gate)
        .expect("feature baseline gate state should materialize");
    let feature_baseline_history = runtime
        .async_node_historical_parity_report(
            &feature_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature baseline historical parity should materialize");
    let feature_baseline_equivalence = runtime
        .async_node_capability_equivalence_report(
            &feature_handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature baseline equivalence should materialize");
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime
        .admit_async_node_request(feature_handle.request_intent())
        .expect("feature branch should admit a pending gate lineage");
    let feature_drifted_state = runtime
        .async_node_gate_state_report(gate)
        .expect("feature drifted gate state should materialize");
    assert_eq!(
        feature_drifted_state.lifecycle_class(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        feature_drifted_state.output_continuity(),
        Some(ResourceOutputContinuity::OutputUnavailableByPolicy)
    );

    runtime
        .switch_branch(sibling.clone())
        .expect("sibling branch should activate");
    let sibling_handle = runtime
        .async_capable_node(gate)
        .expect("sibling branch should rediscover attached gate handle");
    let sibling_state = runtime
        .async_node_gate_state_report(gate)
        .expect("sibling gate state should materialize");
    let sibling_equivalence = runtime
        .async_node_capability_equivalence_report(
            &sibling_handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("sibling equivalence should materialize");
    assert_eq!(
        sibling_state.gate_digest(),
        feature_baseline_state.gate_digest(),
        "sibling branch must remain at the pre-drift gate truth"
    );
    assert_eq!(
        sibling_equivalence.equivalence_digest(),
        feature_baseline_equivalence.equivalence_digest(),
        "equivalent sibling branch should certify the same gate capability truth"
    );

    runtime
        .switch_branch(main)
        .expect("main branch should reactivate before restoring feature");
    runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .expect("feature branch restore should succeed without mutating sibling");

    runtime
        .switch_branch(feature)
        .expect("feature branch should reactivate after restore");
    let feature_rediscovered = runtime
        .async_capable_node(gate)
        .expect("feature restore should still rediscover the public gate handle");
    let feature_restored_state = runtime
        .async_node_gate_state_report(gate)
        .expect("feature restored gate state should materialize");
    let feature_restored_history = runtime
        .async_node_historical_parity_report(
            &feature_rediscovered,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature restored historical parity should materialize");
    let feature_restored_equivalence = runtime
        .async_node_capability_equivalence_report(
            &feature_rediscovered,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature restored equivalence should materialize");

    assert_eq!(
        feature_restored_state.gate_digest(),
        feature_baseline_state.gate_digest()
    );
    assert_ne!(
        feature_restored_state.gate_digest(),
        feature_drifted_state.gate_digest()
    );
    assert_eq!(
        feature_restored_history
            .replay_reconstruction()
            .replay_digest(),
        feature_baseline_history
            .replay_reconstruction()
            .replay_digest()
    );
    assert_eq!(
        feature_restored_history
            .observation_batch_report()
            .map(|report| report.performance()),
        feature_baseline_history
            .observation_batch_report()
            .map(|report| report.performance())
    );
    assert_eq!(
        feature_restored_history.explanation_availability(),
        feature_baseline_history.explanation_availability()
    );
    assert_eq!(
        feature_restored_history.explanation_summary().is_some(),
        feature_baseline_history.explanation_summary().is_some()
    );
    assert_eq!(
        feature_restored_equivalence.capability_declaration_digest(),
        feature_baseline_equivalence.capability_declaration_digest()
    );
    assert_eq!(
        feature_restored_equivalence.lifecycle_digest(),
        feature_baseline_equivalence.lifecycle_digest()
    );
    assert_eq!(
        feature_restored_equivalence.output_continuity_digest(),
        feature_baseline_equivalence.output_continuity_digest()
    );
    assert_eq!(
        feature_restored_equivalence.observation_digest(),
        feature_baseline_equivalence.observation_digest()
    );
    assert_eq!(
        feature_restored_equivalence.explanation_availability(),
        feature_baseline_equivalence.explanation_availability()
    );
}

#[test]
fn keyed_public_rediscovery_and_rebind_churn_stay_branch_local() {
    let mut runtime = TestRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "async-public-branch-churn", ());
    let keyed = family.keyed("left-wing");
    let _owner = keyed.node(&mut runtime);
    let pre_attachment_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("async-public-keyed-feature")
        .expect("feature branch should create");
    let sibling = runtime
        .create_branch("async-public-keyed-sibling")
        .expect("sibling branch should create");

    runtime
        .switch_branch(feature.clone())
        .expect("feature branch should activate");
    let payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(111))
        .with_max_payload_bytes(1024);
    let declaration = keyed.async_capability_declaration(&mut runtime, payload.clone());
    let attached = keyed
        .attach_async_capability(&mut runtime, payload)
        .expect("feature branch should attach the original keyed public lineage");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [attached.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let request = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("feature keyed public request should admit")
        .resource_admission()
        .expect("feature keyed public request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        request.handle(),
        request.attempt(),
        attached.payload_contract_digest().clone(),
        96,
    );
    let feature_handle = keyed
        .async_capable_node(&mut runtime)
        .expect("feature branch should rediscover the keyed public handle");
    let feature_baseline_equivalence = runtime
        .async_node_capability_equivalence_report(
            &feature_handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature baseline equivalence should materialize");
    let feature_attached_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime
        .restore_snapshot(&pre_attachment_snapshot)
        .expect("feature branch should rewind to the pre-attachment ancestor before rebind churn");
    let payload_b = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(112))
        .with_max_payload_bytes(4096);
    let declaration_b = keyed.async_capability_declaration(&mut runtime, payload_b.clone());
    let _rebound = keyed
        .attach_async_capability(&mut runtime, payload_b)
        .expect("feature branch should rebind keyed public attachment");
    let rebound_rediscovered = keyed
        .async_capable_node(&mut runtime)
        .expect("feature rebound keyed public handle should rediscover");
    let feature_rebound_equivalence = runtime
        .async_node_capability_equivalence_report(
            &rebound_rediscovered,
            &declaration_b,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature rebound equivalence should materialize");
    let stale_feature_equivalence = runtime
        .async_node_capability_equivalence_report(
            &feature_handle,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("feature old public handle must fail closed after branch-local rebind");

    runtime
        .switch_branch(sibling.clone())
        .expect("sibling branch should activate");
    let sibling_payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(111))
        .with_max_payload_bytes(1024);
    let sibling_declaration =
        keyed.async_capability_declaration(&mut runtime, sibling_payload.clone());
    let _sibling_attached = keyed
        .attach_async_capability(&mut runtime, sibling_payload)
        .expect("sibling branch should attach the original keyed lineage independently");
    let sibling_handle = keyed
        .async_capable_node(&mut runtime)
        .expect("sibling branch should rediscover the original keyed public handle");
    let sibling_equivalence = runtime
        .async_node_capability_equivalence_report(
            &sibling_handle,
            &sibling_declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("sibling branch should still certify the original keyed declaration");

    assert_eq!(
        sibling_equivalence.capability_declaration_digest(),
        feature_baseline_equivalence.capability_declaration_digest(),
        "branch-local rebind churn must not perturb sibling keyed declaration truth"
    );
    assert_eq!(
        sibling_equivalence.registry_digest(),
        feature_baseline_equivalence.registry_digest()
    );
    assert_eq!(
        sibling_equivalence.bundle_digest(),
        feature_baseline_equivalence.bundle_digest()
    );
    assert_eq!(
        sibling_equivalence.payload_contract_digest(),
        feature_baseline_equivalence.payload_contract_digest()
    );
    assert_eq!(
        sibling_equivalence.explanation_availability(),
        feature_baseline_equivalence.explanation_availability()
    );
    assert_ne!(
        sibling_equivalence.capability_declaration_digest(),
        feature_rebound_equivalence.capability_declaration_digest(),
        "feature rebound declaration must stay branch-local"
    );
    assert_eq!(
        stale_feature_equivalence.denial_class(),
        AsyncNodeCapabilityEquivalenceDenialClass::HistoricalParityDenied
    );
    assert_eq!(
        stale_feature_equivalence
            .historical_parity_denial()
            .expect("feature denial should preserve historical cause")
            .denial_class(),
        AsyncNodeHistoricalParityDenialClass::PayloadContractDigestDrift
    );

    runtime
        .switch_branch(main)
        .expect("main branch should reactivate before restoring feature");
    runtime
        .restore_branch_snapshot(feature.clone(), &feature_attached_snapshot)
        .expect("inactive feature restore should succeed");
    runtime
        .switch_branch(feature)
        .expect("feature branch should reactivate after restore");
    let feature_restored = keyed
        .async_capable_node(&mut runtime)
        .expect("feature restore should rediscover a public handle");
    let feature_restored_equivalence = runtime
        .async_node_capability_equivalence_report(
            &feature_restored,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature restore should reinstate the original keyed public lineage");
    assert_eq!(
        feature_restored_equivalence.equivalence_digest(),
        feature_baseline_equivalence.equivalence_digest()
    );
    assert_eq!(
        feature_restored_equivalence.explanation_digest(),
        feature_baseline_equivalence.explanation_digest()
    );
}
