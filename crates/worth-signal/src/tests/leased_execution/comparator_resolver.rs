use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{authority, request};
use crate::data::comparator::{
    ComparatorPolicyResolver, VersionComparatorPolicy, VersionComparatorResolver,
};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract, NodeId,
    SignalError, SignalGraph,
};
use crate::logic::planner::precompute::callback::CheckedPrecompute;
use crate::logic::planner::{
    execute_prepared_plan_in_scope, run_signal_request_scope, TemporalLoweringContext,
};

struct ChangingResolver {
    calls: AtomicUsize,
}

impl VersionComparatorResolver for ChangingResolver {
    fn resolve(
        &mut self,
        key: &str,
        _aspect: Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, SignalError> {
        assert_eq!(key, "first");
        Ok(true)
    }
}

impl ComparatorPolicyResolver for ChangingResolver {
    fn policy_for_node(
        &self,
        _node: NodeId,
        _override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        VersionComparatorPolicy::Custom {
            key: if call == 0 { "first" } else { "changed" }.to_owned(),
        }
    }
}

#[test]
fn checked_apply_consumes_the_policy_resolved_before_callbacks() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();
    let mut resolver = ChangingResolver {
        calls: AtomicUsize::new(0),
    };
    let lease = authority().request_lease(request(2, 10_000_000)).unwrap();
    let evaluator =
        |_ctx: &mut crate::logic::checked_context::CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
            Ok(AspectVersion::zero().with(Aspect::new(0), 41))
        };
    let report = run_signal_request_scope(&lease, |work, progress, preparation| {
        execute_prepared_plan_in_scope(
            &mut graph,
            &plan,
            &CheckedPrecompute::new(&(), &evaluator),
            &mut resolver,
            TemporalLoweringContext::graph_only(),
            &lease,
            work,
            progress,
            preparation,
        )
    })
    .unwrap();
    assert_eq!(report.tasks_executed, 1);
    assert_eq!(resolver.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        graph.node_aspect_version(node).unwrap().get(Aspect::new(0)),
        41
    );
}
