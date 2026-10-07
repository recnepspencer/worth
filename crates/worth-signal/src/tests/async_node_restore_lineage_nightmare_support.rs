use crate::facade::*;
use crate::tests::async_node_nightmare_support::NoopAsyncNodeObservationListener;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    async_node_capability_with_dependents, settle_async_dependency_baseline,
    AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::define_keyed_computation;

pub(crate) struct MilestoneDRestoreLineageNightmareWorkload {
    pub(crate) baseline_hierarchy_historical: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) drifted_hierarchy_historical: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) restored_hierarchy_historical: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) baseline_keyed_historical: AsyncKeyedNodeHistoricalParityReport,
    pub(crate) baseline_keyed_equivalence: AsyncKeyedNodeCapabilityEquivalenceReport,
    pub(crate) stale_keyed_historical_denial: DeniedAsyncKeyedNodeHistoricalParity,
    pub(crate) stale_keyed_equivalence_denial: DeniedAsyncKeyedNodeCapabilityEquivalence,
    pub(crate) rebound_keyed_historical: AsyncKeyedNodeHistoricalParityReport,
    pub(crate) rebound_keyed_equivalence: AsyncKeyedNodeCapabilityEquivalenceReport,
    pub(crate) old_binding_new_handle_denial: DeniedAsyncKeyedNodeHistoricalParity,
    pub(crate) old_declaration_new_lineage_denial: DeniedAsyncKeyedNodeCapabilityEquivalence,
}

pub(crate) fn milestone_d_restore_lineage_nightmare_workload(
) -> MilestoneDRestoreLineageNightmareWorkload {
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
    admit_and_commit_async_node_completion(
        &mut runtime,
        parent_request.handle(),
        parent_request.attempt(),
        parent_handle.payload_contract_digest().clone(),
        128,
    );

    let family = define_keyed_computation(&mut runtime, "async-restore-nightmare", ());
    let keyed = family.keyed("left");
    let _keyed_owner = keyed.node(&mut runtime);
    let baseline_hierarchy_historical = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("baseline hierarchy historical parity should materialize");
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    let payload_a = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(91))
        .with_max_payload_bytes(1024);
    let declaration_a = keyed.async_capability_declaration(&mut runtime, payload_a.clone());
    let binding_a = keyed
        .declare_async_capability(&mut runtime, payload_a)
        .expect("first keyed capability should attach");
    let handle_a = keyed
        .async_capable_node(&mut runtime)
        .expect("first keyed handle should exist");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [binding_a.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let keyed_request_a = runtime
        .admit_async_node_request(handle_a.request_intent())
        .expect("first keyed request should admit")
        .resource_admission()
        .expect("first keyed request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        keyed_request_a.handle(),
        keyed_request_a.attempt(),
        handle_a.payload_contract_digest().clone(),
        96,
    );

    let baseline_keyed_historical = runtime
        .async_keyed_node_historical_parity_report(
            &binding_a,
            &handle_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("baseline keyed historical parity should materialize");
    let baseline_keyed_equivalence = runtime
        .async_keyed_node_capability_equivalence_report(
            &binding_a,
            &handle_a,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("baseline keyed equivalence should materialize");

    runtime
        .cancel_async_node_request(
            child_request.handle(),
            ResourceCancellationReason::HostRequested,
        )
        .expect("child cancellation should create hierarchy drift");
    let drifted_hierarchy_historical = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("drifted hierarchy historical parity should materialize");

    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should erase post-snapshot keyed attachment and hierarchy drift");
    let restored_hierarchy_historical = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored hierarchy historical parity should materialize");

    let stale_keyed_historical_denial = runtime
        .async_keyed_node_historical_parity_report(
            &binding_a,
            &handle_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("restored-away keyed capability should fail closed");
    let stale_keyed_equivalence_denial = runtime
        .async_keyed_node_capability_equivalence_report(
            &binding_a,
            &handle_a,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("equivalence should fail closed for restored-away keyed capability");

    let payload_b = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(92))
        .with_max_payload_bytes(4096);
    let declaration_b = keyed.async_capability_declaration(&mut runtime, payload_b.clone());
    let binding_b = keyed
        .declare_async_capability(&mut runtime, payload_b)
        .expect("rebound keyed capability should attach");
    let handle_b = keyed
        .async_capable_node(&mut runtime)
        .expect("rebound keyed handle should exist");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [binding_b.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let keyed_request_b = runtime
        .admit_async_node_request(handle_b.request_intent())
        .expect("rebound keyed request should admit")
        .resource_admission()
        .expect("rebound keyed request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        keyed_request_b.handle(),
        keyed_request_b.attempt(),
        handle_b.payload_contract_digest().clone(),
        144,
    );

    let rebound_keyed_historical = runtime
        .async_keyed_node_historical_parity_report(
            &binding_b,
            &handle_b,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("rebound keyed historical parity should materialize");
    let rebound_keyed_equivalence = runtime
        .async_keyed_node_capability_equivalence_report(
            &binding_b,
            &handle_b,
            &declaration_b,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("rebound keyed equivalence should materialize");

    let old_binding_new_handle_denial = runtime
        .async_keyed_node_historical_parity_report(
            &binding_a,
            &handle_b,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("old binding must not certify rebound keyed lineage");
    let old_declaration_new_lineage_denial = runtime
        .async_keyed_node_capability_equivalence_report(
            &binding_b,
            &handle_b,
            &declaration_a,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect_err("old declaration must not certify rebound keyed lineage");

    MilestoneDRestoreLineageNightmareWorkload {
        baseline_hierarchy_historical,
        drifted_hierarchy_historical,
        restored_hierarchy_historical,
        baseline_keyed_historical,
        baseline_keyed_equivalence,
        stale_keyed_historical_denial,
        stale_keyed_equivalence_denial,
        rebound_keyed_historical,
        rebound_keyed_equivalence,
        old_binding_new_handle_denial,
        old_declaration_new_lineage_denial,
    }
}
