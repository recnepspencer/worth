use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::EvaluationCondition;
use crate::data::output::{scope_touched_by_artifact_state, PartitionSubscription};

use super::types::ConditionDecision;

pub(super) fn classify_condition_decision(
    graph: &SignalGraph,
    node: NodeId,
    condition: &EvaluationCondition,
    dirty_aspects: crate::data::aspect::AspectMask,
) -> Option<ConditionDecision> {
    let max_delta = max_dependency_delta(graph, node).ok()?;

    match condition {
        EvaluationCondition::AspectFilter(mask)
            if !dirty_aspects.is_empty() && !dirty_aspects.intersects(*mask) =>
        {
            Some(ConditionDecision::Deferred)
        }
        EvaluationCondition::OnDemand => Some(ConditionDecision::Deferred),
        EvaluationCondition::DeltaThreshold(threshold)
            if !dirty_aspects.is_empty() && (max_delta as f64) <= *threshold =>
        {
            Some(ConditionDecision::RevertedClean)
        }
        EvaluationCondition::Temporal(_) => Some(ConditionDecision::Deferred),
        EvaluationCondition::Custom(_) => Some(ConditionDecision::Deferred),
        _ => None,
    }
}

fn max_dependency_delta(graph: &SignalGraph, node: NodeId) -> Result<u64, SignalError> {
    let mut max_delta = 0;
    for snapshot_entry in graph.get_dep_snapshot(node)?.entries() {
        if !graph.is_alive(snapshot_entry.source) {
            continue;
        }
        let current_version = graph.node_version_for_scope(
            snapshot_entry.source,
            snapshot_entry.aspect,
            snapshot_entry.scope.as_ref(),
        )?;
        max_delta = max_delta.max(current_version.abs_diff(snapshot_entry.cached_version));
    }
    Ok(max_delta)
}

pub(super) fn partition_scope_untouched(
    artifact_state: Option<&crate::data::trace::RuntimeArtifactState>,
    scope: &PartitionSubscription,
) -> bool {
    !scope_touched_by_artifact_state(artifact_state, scope)
}
