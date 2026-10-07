use crate::facade::*;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    async_node_capability_with_dependents, settle_async_dependency_baseline,
    AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::{define_keyed_computation, evaluate, version_ab};

pub(crate) struct NoopAsyncNodeObservationListener;

impl ObservationListener<(), (), (), (), ()> for NoopAsyncNodeObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

pub(crate) struct MilestoneDCombinedWorkload {
    pub(crate) attachment_equivalence_before_restore: AsyncNodeCapabilityEquivalenceReport,
    pub(crate) attachment_equivalence_after_restore: AsyncNodeCapabilityEquivalenceReport,
    pub(crate) gate_state_before_restore: AsyncNodeGateStateReport,
    pub(crate) gate_state_after_restore: AsyncNodeGateStateReport,
    pub(crate) gate_historical_before_restore: AsyncNodeHistoricalParityReport,
    pub(crate) gate_historical_after_restore: AsyncNodeHistoricalParityReport,
    pub(crate) aspect_keyed_historical: AsyncKeyedNodeHistoricalParityReport,
    pub(crate) aspect_keyed_equivalence: AsyncKeyedNodeCapabilityEquivalenceReport,
    pub(crate) hierarchy_replay_before_restore: AsyncNodeHierarchyReplaySummary,
    pub(crate) hierarchy_historical_before_restore: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) hierarchy_replay_after_cancellation: AsyncNodeHierarchyReplaySummary,
    pub(crate) hierarchy_replay_after_restore: AsyncNodeHierarchyReplaySummary,
    pub(crate) hierarchy_cancellation: AsyncNodeHierarchyCancellationReport,
    pub(crate) hierarchy_historical_after_cancellation: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) hierarchy_historical_after_restore: AsyncNodeHierarchyHistoricalParityReport,
}

pub(crate) fn milestone_d_combined_workload() -> MilestoneDCombinedWorkload {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let gate = graph.node().build();
    let sink = graph.node().build();
    let parent = graph.node().build();
    let child = graph.node().build();
    let grandchild = graph.node().build();
    graph
        .append_dependency(gate, source, Aspect::new(0))
        .expect("source should feed gate");
    graph
        .append_dependency(sink, gate, Aspect::new(0))
        .expect("gate should feed sink");
    graph
        .append_dependency(child, parent, Aspect::new(0))
        .expect("child should depend on parent");
    graph
        .append_dependency(grandchild, child, Aspect::new(0))
        .expect("grandchild should depend on child");
    settle_async_dependency_baseline(&mut graph, [parent, child, grandchild]);
    let mut source_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut gate_eval = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0)).with_output_identity("gate-v1"))
    };
    let mut sink_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(3, 0));
    evaluate(&mut graph, source, &mut source_eval).expect("source should evaluate");
    evaluate(&mut graph, gate, &mut gate_eval).expect("gate should evaluate");
    evaluate(&mut graph, sink, &mut sink_eval).expect("sink should evaluate");

    let mut runtime = TestRuntime::build(graph);
    let gate_declaration = async_node_capability_with_dependents(gate, [sink])
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput)
        .with_output_continuity_policy(ResourceOutputContinuityPolicyDeclaration::HideWhilePending);
    let gate_handle = runtime
        .attach_async_capability(gate_declaration.clone())
        .expect("gate capability should attach");
    let parent_handle = runtime
        .attach_async_capability(async_node_capability_with_dependents(parent, [child]))
        .expect("parent capability should attach");
    let child_handle = runtime
        .attach_async_capability(async_node_capability_with_dependents(child, [grandchild]))
        .expect("child capability should attach");
    runtime
        .attach_async_capability(async_node_capability_declaration(grandchild))
        .expect("grandchild capability should attach");

    let family = define_keyed_computation(&mut runtime, "async-nightmare", ());
    let keyed = family.keyed("left");
    let keyed_payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(77));
    let keyed_declaration = keyed.async_capability_declaration(&mut runtime, keyed_payload.clone());
    let keyed_binding = keyed
        .declare_async_capability(&mut runtime, keyed_payload)
        .expect("keyed capability should attach");
    let keyed_handle = keyed
        .async_capable_node(&mut runtime)
        .expect("keyed handle should exist");

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [gate, parent, keyed_binding.node()],
        Box::new(NoopAsyncNodeObservationListener),
    );

    let gate_request = runtime
        .admit_async_node_request(gate_handle.request_intent())
        .expect("gate request should admit")
        .resource_admission()
        .expect("gate request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        gate_request.handle(),
        gate_request.attempt(),
        gate_handle.payload_contract_digest().clone(),
        64,
    );
    runtime
        .admit_async_node_request(keyed_handle.request_intent())
        .expect("keyed request should admit");
    let parent_request = runtime
        .admit_async_node_request(parent_handle.request_intent())
        .expect("parent request should admit")
        .resource_admission()
        .expect("parent request should lower into resource admission")
        .admitted_request();
    runtime
        .admit_async_node_request(child_handle.request_intent())
        .expect("child request should admit");
    runtime
        .admit_async_node_request(AsyncNodeRequestIntent::new(grandchild))
        .expect("grandchild request should admit");

    let attachment_equivalence_before_restore = runtime
        .async_node_capability_equivalence_report(
            &gate_handle,
            &gate_declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("gate equivalence should materialize");
    let gate_state_before_restore = runtime
        .async_node_gate_state_report(gate)
        .expect("gate state should materialize");
    let gate_historical_before_restore = runtime
        .async_node_historical_parity_report(
            &gate_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("gate historical parity should materialize");
    let aspect_keyed_historical = runtime
        .async_keyed_node_historical_parity_report(
            &keyed_binding,
            &keyed_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("keyed historical parity should materialize");
    let aspect_keyed_equivalence = runtime
        .async_keyed_node_capability_equivalence_report(
            &keyed_binding,
            &keyed_handle,
            &keyed_declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("keyed equivalence should materialize");
    let hierarchy_replay_before_restore = runtime
        .async_node_hierarchy_replay_summary(parent)
        .expect("hierarchy replay should materialize");
    let hierarchy_historical_before_restore = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("pre-cancellation hierarchy historical parity should materialize");

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let hierarchy_cancellation = runtime
        .cancel_async_node_request(
            parent_request.handle(),
            ResourceCancellationReason::HostRequested,
        )
        .expect("hierarchy cancellation should materialize");
    let hierarchy_replay_after_cancellation = runtime
        .async_node_hierarchy_replay_summary(parent)
        .expect("post-cancellation hierarchy replay should materialize");
    let hierarchy_historical_after_cancellation = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("post-cancellation hierarchy historical parity should materialize");

    runtime
        .restore_snapshot(&snapshot)
        .expect("restore should reinstate combined nightmare truth");

    let attachment_equivalence_after_restore = runtime
        .async_node_capability_equivalence_report(
            &gate_handle,
            &gate_declaration,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("gate equivalence should rematerialize after restore");
    let gate_state_after_restore = runtime
        .async_node_gate_state_report(gate)
        .expect("restored gate state should materialize");
    let gate_historical_after_restore = runtime
        .async_node_historical_parity_report(
            &gate_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored gate historical parity should materialize");
    let hierarchy_replay_after_restore = runtime
        .async_node_hierarchy_replay_summary(parent)
        .expect("restored hierarchy replay should materialize");
    let hierarchy_historical_after_restore = runtime
        .async_node_hierarchy_historical_parity_report(
            &parent_handle,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("restored hierarchy historical parity should materialize");

    MilestoneDCombinedWorkload {
        attachment_equivalence_before_restore,
        attachment_equivalence_after_restore,
        gate_state_before_restore,
        gate_state_after_restore,
        gate_historical_before_restore,
        gate_historical_after_restore,
        aspect_keyed_historical,
        aspect_keyed_equivalence,
        hierarchy_replay_before_restore,
        hierarchy_historical_before_restore,
        hierarchy_replay_after_cancellation,
        hierarchy_replay_after_restore,
        hierarchy_cancellation,
        hierarchy_historical_after_cancellation,
        hierarchy_historical_after_restore,
    }
}
