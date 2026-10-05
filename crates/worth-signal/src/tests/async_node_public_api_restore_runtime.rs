use crate::facade::*;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    AsyncNodeTestRuntime as TestRuntime,
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
fn async_capable_node_public_rediscovery_after_restore_preserves_parity_and_explanation_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    let declaration = async_node_capability_declaration(node)
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput)
        .with_output_continuity_policy(ResourceOutputContinuityPolicyDeclaration::HideWhilePending);
    let attached = runtime
        .attach_async_capability(declaration.clone())
        .expect("async capability should attach");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [node],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let admitted_request = runtime
        .admit_async_node_request(attached.request_intent())
        .expect("attached handle should admit request")
        .resource_admission()
        .expect("attached request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        admitted_request.handle(),
        admitted_request.attempt(),
        attached.payload_contract_digest().clone(),
        88,
    );

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should preserve the already-attached capability lineage");

    let rediscovered = runtime
        .async_capable_node(node)
        .expect("restored attachment should still be rediscoverable through the public handle API");
    assert_eq!(rediscovered.node(), attached.node());
    assert_eq!(rediscovered.registry_digest(), attached.registry_digest());
    assert_eq!(rediscovered.bundle_digest(), attached.bundle_digest());
    assert_eq!(
        rediscovered.payload_contract_digest(),
        attached.payload_contract_digest()
    );

    let historical_from_attached = runtime
        .async_node_historical_parity_report(
            &attached,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored world should still certify the original public handle");
    let historical_from_rediscovered = runtime
        .async_node_historical_parity_report(
            &rediscovered,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored world should certify the rediscovered public handle");
    assert_eq!(
        historical_from_attached.parity_digest(),
        historical_from_rediscovered.parity_digest()
    );
    assert!(historical_from_attached.branch_restore_report().is_some());
    assert_eq!(
        historical_from_attached.explanation_availability(),
        historical_from_rediscovered.explanation_availability()
    );
    assert_eq!(
        historical_from_attached.explanation_summary(),
        historical_from_rediscovered.explanation_summary()
    );

    let equivalence_from_attached = runtime
        .async_node_capability_equivalence_report(
            &attached,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored world should certify equivalence from the original public handle");
    let equivalence_from_rediscovered = runtime
        .async_node_capability_equivalence_report(
            &rediscovered,
            &declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored world should certify equivalence from the rediscovered public handle");
    assert_eq!(
        equivalence_from_attached.equivalence_digest(),
        equivalence_from_rediscovered.equivalence_digest()
    );
    assert_eq!(
        equivalence_from_attached.explanation_digest(),
        equivalence_from_rediscovered.explanation_digest()
    );
    assert_eq!(
        equivalence_from_attached.replay_restore_digest(),
        equivalence_from_rediscovered.replay_restore_digest()
    );
}

#[test]
fn keyed_public_handles_fail_closed_after_restore_rebind_and_require_rediscovered_lineage() {
    let mut runtime = TestRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let family = define_keyed_computation(&mut runtime, "async-public-restore", ());
    let keyed = family.keyed("left-wing");
    let _owner = keyed.node(&mut runtime);
    let pre_attachment_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    let payload_a = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(101))
        .with_max_payload_bytes(1024);
    let declaration_a = keyed.async_capability_declaration(&mut runtime, payload_a.clone());
    let attached_a = keyed
        .attach_async_capability(&mut runtime, payload_a)
        .expect("first keyed public attachment should succeed");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [attached_a.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let request_a = runtime
        .admit_async_node_request(attached_a.request_intent())
        .expect("first keyed public handle should admit request")
        .resource_admission()
        .expect("first keyed public request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        request_a.handle(),
        request_a.attempt(),
        attached_a.payload_contract_digest().clone(),
        96,
    );

    let attached_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime
        .restore_snapshot(&attached_snapshot)
        .expect("restoring attached keyed lineage should preserve public rediscovery");
    let rediscovered_a = keyed
        .async_capable_node(&mut runtime)
        .expect("attached keyed lineage should rediscover through the public API");
    assert_eq!(rediscovered_a.node(), attached_a.node());
    assert_eq!(rediscovered_a.bundle_digest(), attached_a.bundle_digest());
    assert_eq!(
        rediscovered_a.payload_contract_digest(),
        attached_a.payload_contract_digest()
    );
    let baseline_equivalence_a = runtime
        .async_node_capability_equivalence_report(
            &attached_a,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored original keyed public handle should still certify");
    let rediscovered_equivalence_a = runtime
        .async_node_capability_equivalence_report(
            &rediscovered_a,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("rediscovered keyed public handle should certify identically");
    assert_eq!(
        baseline_equivalence_a.equivalence_digest(),
        rediscovered_equivalence_a.equivalence_digest()
    );

    runtime
        .restore_snapshot(&pre_attachment_snapshot)
        .expect("restore should erase the original keyed attachment while keeping the owner alive");

    let payload_b = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(102))
        .with_max_payload_bytes(4096);
    let declaration_b = keyed.async_capability_declaration(&mut runtime, payload_b.clone());
    let attached_b = keyed
        .attach_async_capability(&mut runtime, payload_b)
        .expect("rebound keyed public attachment should succeed");
    let rediscovered_b = keyed
        .async_capable_node(&mut runtime)
        .expect("rebound keyed public attachment should rediscover");
    assert_eq!(rediscovered_b.node(), attached_b.node());
    assert_eq!(
        rediscovered_b.payload_contract_digest(),
        attached_b.payload_contract_digest()
    );

    let stale_history = runtime
        .async_node_historical_parity_report(
            &attached_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("old keyed public handle must fail closed after restore and rebind");
    assert_eq!(
        stale_history.denial_class(),
        AsyncNodeHistoricalParityDenialClass::PayloadContractDigestDrift
    );
    let stale_equivalence = runtime
        .async_node_capability_equivalence_report(
            &attached_a,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("old keyed public handle must not certify equivalence after rebind");
    assert_eq!(
        stale_equivalence.denial_class(),
        AsyncNodeCapabilityEquivalenceDenialClass::HistoricalParityDenied
    );
    assert_eq!(
        stale_equivalence
            .historical_parity_denial()
            .expect("equivalence denial should preserve the public historical cause")
            .denial_class(),
        AsyncNodeHistoricalParityDenialClass::PayloadContractDigestDrift
    );

    let rebound_equivalence = runtime
        .async_node_capability_equivalence_report(
            &rediscovered_b,
            &declaration_b,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("rebound keyed public handle should certify with the rebound declaration");
    assert_eq!(
        rebound_equivalence.capability_declaration_digest(),
        canonical_digest(&declaration_b)
    );
    assert_ne!(
        baseline_equivalence_a.capability_declaration_digest(),
        rebound_equivalence.capability_declaration_digest()
    );
    assert_eq!(
        rebound_equivalence.explanation_availability(),
        baseline_equivalence_a.explanation_availability()
    );
}
