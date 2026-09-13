use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntimePolicy};
use crate::logic::explain::NodeExplanation;
use crate::tests::support::{evaluate, version_ab};

pub(crate) fn materialized_explanation() -> NodeExplanation {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().output_identity().build();
    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("retained-fact")
            .with_label("retained-label"))
    })
    .unwrap();
    let explanation = graph
        .observe()
        .materialize()
        .materialize_explanation_artifact(node)
        .unwrap()
        .0
        .unwrap();
    assert!(explanation.historical_artifact_record.is_some());
    explanation
}
