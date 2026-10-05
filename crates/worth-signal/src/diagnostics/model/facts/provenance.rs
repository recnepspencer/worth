use std::collections::BTreeMap;

use crate::data::aspect::AspectMask;
use crate::data::handle::NodeId;
use crate::data::node::{ContextRequirement, EvaluationCondition, NodeState};
use crate::data::trace::{
    CausalityMetadata, ColdArtifactRecord, ExecutionTraceStamp, RuntimeArtifactState,
};
use crate::logic::explain::{NodeExplanation, RewiringSummary, UpstreamCause};

use super::projection::compact_retained_explanation;
use super::vocabulary::{
    ProvenanceEdge, ProvenanceEdgeKind, ProvenanceFact, ProvenanceVertex, ProvenanceVertexRole,
};

impl ProvenanceFact {
    pub fn from_explanation(explanation: &NodeExplanation) -> Self {
        let mut vertices = BTreeMap::new();
        vertices.insert(
            explanation.node,
            ProvenanceVertex {
                node: explanation.node,
                role: ProvenanceVertexRole::Target,
                state: Some(format!("{:?}", explanation.state)),
            },
        );
        let mut edges = explanation
            .upstream
            .iter()
            .map(|cause| {
                let edge = ProvenanceEdge::from_upstream_cause(cause);
                vertices
                    .entry(edge.source)
                    .or_insert_with(|| ProvenanceVertex {
                        node: edge.source,
                        role: ProvenanceVertexRole::Upstream,
                        state: None,
                    });
                edge
            })
            .collect::<Vec<_>>();
        edges.sort_by_key(|edge| {
            (
                edge.source.index(),
                edge.source.generation(),
                edge.aspect.index(),
                edge.kind.clone(),
                edge.subscription
                    .as_ref()
                    .map(|scope| (scope.path().segments().to_vec(), scope.coverage() as u8))
                    .unwrap_or_default(),
            )
        });
        Self {
            node: explanation.node,
            materialization_mode: explanation.materialization_mode,
            execution_record_id: explanation.execution_record_id,
            semantic_segment_id: explanation.semantic_segment_id,
            vertices: vertices.into_values().collect(),
            edges,
            causal_links: explanation.causal_links.clone(),
            rewiring: explanation.rewiring.clone(),
            propagation_suppressed: explanation.propagation_suppressed,
            causality_kind: explanation.causality.as_ref().map(|c| c.kind.clone()),
        }
    }

    pub fn from_runtime_projection(
        node: NodeId,
        state: NodeState,
        contract_reads: AspectMask,
        contract_produces: AspectMask,
        contract_partition_scope: Option<Vec<crate::data::output::PartitionSubscription>>,
        required_context: ContextRequirement,
        condition: EvaluationCondition,
        runtime: &RuntimeArtifactState,
        retained: Option<&ColdArtifactRecord>,
        execution: Option<ExecutionTraceStamp>,
        causality: Option<&CausalityMetadata>,
        rewiring: Option<RewiringSummary>,
    ) -> Self {
        Self::from_explanation(&compact_retained_explanation(
            node,
            state,
            contract_reads,
            contract_produces,
            contract_partition_scope,
            required_context,
            condition,
            runtime,
            retained,
            execution,
            causality,
            rewiring,
        ))
    }
}

impl ProvenanceEdge {
    fn from_upstream_cause(cause: &UpstreamCause) -> Self {
        match cause {
            UpstreamCause::Changed {
                source,
                aspect,
                subscription,
                cached_version,
                current_version,
                comparator,
                reason,
            } => Self {
                kind: ProvenanceEdgeKind::Changed,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: Some(*cached_version),
                current_version: Some(*current_version),
                comparator: Some(format!("{comparator:?}")),
                reason: Some(format!("{reason:?}")),
            },
            UpstreamCause::SkippedByComparator {
                source,
                aspect,
                subscription,
                cached_version,
                current_version,
                comparator,
                reason,
            } => Self {
                kind: ProvenanceEdgeKind::SkippedByComparator,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: Some(*cached_version),
                current_version: Some(*current_version),
                comparator: Some(format!("{comparator:?}")),
                reason: Some(format!("{reason:?}")),
            },
            UpstreamCause::ConditionDeferred {
                source,
                aspect,
                subscription,
                cached_version,
                current_version,
                condition: _,
                decision: _,
            } => Self {
                kind: ProvenanceEdgeKind::ConditionDeferred,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: Some(*cached_version),
                current_version: Some(*current_version),
                comparator: None,
                reason: None,
            },
            UpstreamCause::Clean {
                source,
                aspect,
                subscription,
                cached_version,
                current_version,
            } => Self {
                kind: ProvenanceEdgeKind::Clean,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: Some(*cached_version),
                current_version: Some(*current_version),
                comparator: None,
                reason: None,
            },
            UpstreamCause::MissingSnapshot {
                source,
                aspect,
                subscription,
                current_version,
            } => Self {
                kind: ProvenanceEdgeKind::MissingSnapshot,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: None,
                current_version: *current_version,
                comparator: None,
                reason: None,
            },
            UpstreamCause::DependencyRemoved {
                source,
                aspect,
                subscription,
                cached_version,
            } => Self {
                kind: ProvenanceEdgeKind::DependencyRemoved,
                source: *source,
                aspect: *aspect,
                subscription: subscription.clone(),
                cached_version: Some(*cached_version),
                current_version: None,
                comparator: None,
                reason: None,
            },
        }
    }
}
