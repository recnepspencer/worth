use crate::facade::*;
use crate::tests::async_node_support::{
    admit_and_commit_async_node_completion, async_node_capability_declaration,
    async_node_capability_with_dependents, settle_async_dependency_baseline,
    AsyncNodeTestRuntime as TestRuntime,
};
use crate::tests::support::{evaluate, version_ab};

pub(crate) struct PublicHierarchyBranchWorkload {
    pub(crate) feature_gate_baseline: AsyncNodeGateStateReport,
    pub(crate) feature_gate_history_baseline: AsyncNodeHistoricalParityReport,
    pub(crate) feature_gate_drifted: AsyncNodeGateStateReport,
    pub(crate) feature_gate_restored_state: AsyncNodeGateStateReport,
    pub(crate) feature_gate_restored_history: AsyncNodeHistoricalParityReport,
    pub(crate) feature_hierarchy_baseline: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) feature_hierarchy_after_cancellation: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) feature_hierarchy_restored: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) sibling_gate_baseline: AsyncNodeGateStateReport,
    pub(crate) sibling_gate_history: AsyncNodeHistoricalParityReport,
    pub(crate) sibling_hierarchy: AsyncNodeHierarchyHistoricalParityReport,
    pub(crate) sibling_still_gate_baseline: AsyncNodeGateStateReport,
    pub(crate) sibling_still_hierarchy: AsyncNodeHierarchyHistoricalParityReport,
}

pub(crate) struct NoopAsyncNodeObservationListener;

impl ObservationListener<(), (), (), (), ()> for NoopAsyncNodeObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

pub(crate) fn explanation_signature(summary: Option<&ExplanationSummary>) -> Option<String> {
    summary.map(|summary| {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            summary.node,
            summary.materialization_mode,
            summary.state,
            summary.output_change,
            summary.memoized_origin,
            summary.reuse_origin,
            summary.direct_cause_kinds
        )
    })
}

pub(crate) fn public_hierarchy_branch_workload() -> PublicHierarchyBranchWorkload {
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

    let parent = graph.node().build();
    let child = graph.node().build();
    let grandchild = graph.node().build();
    graph
        .append_dependency(child, parent, Aspect::new(0))
        .expect("child should depend on parent");
    graph
        .append_dependency(grandchild, child, Aspect::new(0))
        .expect("grandchild should depend on child");

    let mut source_eval = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut gate_eval = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0)).with_output_identity("gate-v1"))
    };
    evaluate(&mut graph, source, &mut source_eval).expect("source should evaluate");
    evaluate(&mut graph, gate, &mut gate_eval).expect("gate should evaluate");
    settle_async_dependency_baseline(&mut graph, [sink, parent, child, grandchild]);

    let mut runtime = TestRuntime::build(graph);
    let gate_declaration = async_node_capability_declaration(gate)
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleAndOutput)
        .with_output_continuity_policy(ResourceOutputContinuityPolicyDeclaration::HideWhilePending);
    runtime
        .attach_async_capability(gate_declaration)
        .expect("gate capability should attach");
    runtime
        .attach_async_capability(async_node_capability_with_dependents(parent, [child]))
        .expect("parent capability should attach");
    runtime
        .attach_async_capability(async_node_capability_with_dependents(child, [grandchild]))
        .expect("child capability should attach");
    runtime
        .attach_async_capability(async_node_capability_declaration(grandchild))
        .expect("grandchild capability should attach");

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("async-public-hierarchy-feature")
        .expect("feature branch should create");
    let sibling = runtime
        .create_branch("async-public-hierarchy-sibling")
        .expect("sibling branch should create");

    runtime
        .switch_branch(feature.clone())
        .expect("feature branch should activate");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [gate, parent],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let feature_gate = runtime
        .async_capable_node(gate)
        .expect("feature branch should rediscover gate handle");
    let feature_parent = runtime
        .async_capable_node(parent)
        .expect("feature branch should rediscover hierarchy root handle");
    let feature_child = runtime
        .async_capable_node(child)
        .expect("feature branch should rediscover child handle");
    let feature_grandchild = runtime
        .async_capable_node(grandchild)
        .expect("feature branch should rediscover grandchild handle");

    let feature_gate_request = runtime
        .admit_async_node_request(feature_gate.request_intent())
        .expect("feature gate request should admit")
        .resource_admission()
        .expect("feature gate request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        feature_gate_request.handle(),
        feature_gate_request.attempt(),
        feature_gate.payload_contract_digest().clone(),
        88,
    );
    runtime
        .admit_async_node_request(feature_grandchild.request_intent())
        .expect("feature grandchild request should admit");
    let feature_child_request = runtime
        .admit_async_node_request(feature_child.request_intent())
        .expect("feature child request should admit")
        .resource_admission()
        .expect("feature child request should lower into resource admission")
        .admitted_request();
    let feature_parent_request = runtime
        .admit_async_node_request(feature_parent.request_intent())
        .expect("feature parent request should admit")
        .resource_admission()
        .expect("feature parent request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        feature_parent_request.handle(),
        feature_parent_request.attempt(),
        feature_parent.payload_contract_digest().clone(),
        144,
    );

    let feature_gate_baseline = runtime
        .async_node_gate_state_report(gate)
        .expect("feature gate baseline should materialize");
    let feature_gate_history_baseline = runtime
        .async_node_historical_parity_report(
            &feature_gate,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature gate historical parity should materialize");
    let feature_hierarchy_baseline = runtime
        .async_node_hierarchy_historical_parity_report(
            &feature_parent,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature hierarchy historical parity should materialize");
    let feature_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    runtime
        .admit_async_node_request(feature_gate.request_intent())
        .expect("feature gate should admit a pending drift lineage");
    let feature_gate_drifted = runtime
        .async_node_gate_state_report(gate)
        .expect("feature gate drift should materialize");
    let _feature_hierarchy_cancellation = runtime
        .cancel_async_node_request(
            feature_child_request.handle(),
            ResourceCancellationReason::HostRequested,
        )
        .expect("feature child cancellation should create hierarchy drift");
    let _feature_hierarchy_replay_after_cancellation = runtime
        .async_node_hierarchy_replay_summary(parent)
        .expect("feature hierarchy replay after cancellation should materialize");
    let feature_hierarchy_after_cancellation = runtime
        .async_node_hierarchy_historical_parity_report(
            &feature_parent,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature drifted hierarchy historical parity should materialize");

    runtime
        .switch_branch(sibling.clone())
        .expect("sibling branch should activate");
    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [gate, parent],
        Box::new(NoopAsyncNodeObservationListener),
    );
    let sibling_gate = runtime
        .async_capable_node(gate)
        .expect("sibling branch should rediscover gate handle");
    let sibling_parent = runtime
        .async_capable_node(parent)
        .expect("sibling branch should rediscover hierarchy root handle");
    let sibling_child = runtime
        .async_capable_node(child)
        .expect("sibling branch should rediscover child handle");
    let sibling_grandchild = runtime
        .async_capable_node(grandchild)
        .expect("sibling branch should rediscover grandchild handle");

    let sibling_gate_request = runtime
        .admit_async_node_request(sibling_gate.request_intent())
        .expect("sibling gate request should admit")
        .resource_admission()
        .expect("sibling gate request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        sibling_gate_request.handle(),
        sibling_gate_request.attempt(),
        sibling_gate.payload_contract_digest().clone(),
        88,
    );
    runtime
        .admit_async_node_request(sibling_grandchild.request_intent())
        .expect("sibling grandchild request should admit");
    runtime
        .admit_async_node_request(sibling_child.request_intent())
        .expect("sibling child request should admit");
    let sibling_parent_request = runtime
        .admit_async_node_request(sibling_parent.request_intent())
        .expect("sibling parent request should admit")
        .resource_admission()
        .expect("sibling parent request should lower into resource admission")
        .admitted_request();
    admit_and_commit_async_node_completion(
        &mut runtime,
        sibling_parent_request.handle(),
        sibling_parent_request.attempt(),
        sibling_parent.payload_contract_digest().clone(),
        144,
    );

    let sibling_gate_baseline = runtime
        .async_node_gate_state_report(gate)
        .expect("sibling gate baseline should materialize");
    let sibling_gate_history = runtime
        .async_node_historical_parity_report(
            &sibling_gate,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("sibling gate historical parity should materialize");
    let sibling_hierarchy = runtime
        .async_node_hierarchy_historical_parity_report(
            &sibling_parent,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("sibling hierarchy historical parity should materialize");

    runtime
        .switch_branch(main)
        .expect("main branch should reactivate before feature restore");
    runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .expect("feature restore should succeed");

    runtime
        .switch_branch(sibling)
        .expect("sibling branch should remain independently accessible");
    let sibling_still_gate_baseline = runtime
        .async_node_gate_state_report(gate)
        .expect("sibling gate baseline should remain available");
    let sibling_still_parent = runtime
        .async_capable_node(parent)
        .expect("sibling hierarchy root should still rediscover");
    let sibling_still_hierarchy = runtime
        .async_node_hierarchy_historical_parity_report(
            &sibling_still_parent,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("sibling hierarchy should remain stable");

    runtime
        .switch_branch(feature)
        .expect("feature branch should reactivate after restore");
    let feature_gate_restored = runtime
        .async_capable_node(gate)
        .expect("feature restore should rediscover gate handle");
    let feature_parent_restored = runtime
        .async_capable_node(parent)
        .expect("feature restore should rediscover hierarchy root handle");
    let feature_gate_restored_state = runtime
        .async_node_gate_state_report(gate)
        .expect("feature restored gate state should materialize");
    let feature_gate_restored_history = runtime
        .async_node_historical_parity_report(
            &feature_gate_restored,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature restored gate history should materialize");
    let feature_hierarchy_restored = runtime
        .async_node_hierarchy_historical_parity_report(
            &feature_parent_restored,
            ResourceDiagnosticsExpansionBudget::allow_cold_reconstruction(u32::MAX),
        )
        .expect("feature restored hierarchy historical parity should materialize");

    PublicHierarchyBranchWorkload {
        feature_gate_baseline,
        feature_gate_history_baseline,
        feature_gate_drifted,
        feature_gate_restored_state,
        feature_gate_restored_history,
        feature_hierarchy_baseline,
        feature_hierarchy_after_cancellation,
        feature_hierarchy_restored,
        sibling_gate_baseline,
        sibling_gate_history,
        sibling_hierarchy,
        sibling_still_gate_baseline,
        sibling_still_hierarchy,
    }
}
