use crate::data::comparator::{ComparatorPolicyResolver, VersionComparatorPolicy};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::EvaluationCondition;

use super::super::analysis::partition_scope_untouched;
use super::super::types::{reason_for_policy, ConditionDecision, UpstreamCause};
use super::lineage::ExplanationLineage;

pub(super) fn resolve_upstream_causes(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
    comparator_override: Option<&VersionComparatorPolicy>,
    explicit_comparator: bool,
    condition: &EvaluationCondition,
    condition_decision: Option<ConditionDecision>,
    lineage: &mut ExplanationLineage,
) -> Result<Vec<UpstreamCause>, SignalError> {
    let mut upstream = Vec::new();
    let current_dependencies = lineage.current_dependencies().to_vec();
    for dependency in &current_dependencies {
        let subscription = dependency.scope_ref().cloned();
        let current_version = if graph.is_alive(dependency.source()) {
            lineage.traversal_cost_mut().note_source_version_lookup();
            Some(graph.node_version_for_scope(
                dependency.source(),
                dependency.aspect(),
                dependency.scope_ref(),
            )?)
        } else {
            None
        };

        let Some(cached_version) = lineage.cached_version(
            dependency.source(),
            dependency.aspect().index(),
            dependency.scope_ref(),
        ) else {
            upstream.push(UpstreamCause::MissingSnapshot {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription: subscription.clone(),
                current_version,
            });
            continue;
        };

        let Some(current_version) = current_version else {
            upstream.push(UpstreamCause::DependencyRemoved {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription: subscription.clone(),
                cached_version,
            });
            continue;
        };

        if current_version == cached_version {
            upstream.push(UpstreamCause::Clean {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription,
                cached_version,
                current_version,
            });
            continue;
        }

        if let Some(scope) = subscription.as_ref() {
            lineage.traversal_cost_mut().note_scope_validation();
            let source_trace = graph.node_runtime_artifact_state(dependency.source())?;
            if partition_scope_untouched(source_trace, scope) {
                upstream.push(UpstreamCause::Clean {
                    source: dependency.source(),
                    aspect: dependency.aspect(),
                    subscription,
                    cached_version,
                    current_version,
                });
                continue;
            }
        }

        if let Some(decision) = condition_decision {
            upstream.push(UpstreamCause::ConditionDeferred {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription,
                cached_version,
                current_version,
                condition: condition.clone(),
                decision,
            });
            continue;
        }

        let policy = comparator_resolver.policy_for_node(node, comparator_override);
        match &policy {
            VersionComparatorPolicy::Exact => upstream.push(UpstreamCause::Changed {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription: subscription.clone(),
                cached_version,
                current_version,
                comparator: policy.clone(),
                reason: reason_for_policy(&policy, explicit_comparator),
            }),
            VersionComparatorPolicy::Tolerance { epsilon } => {
                if current_version.abs_diff(cached_version) > *epsilon {
                    upstream.push(UpstreamCause::Changed {
                        source: dependency.source(),
                        aspect: dependency.aspect(),
                        subscription: subscription.clone(),
                        cached_version,
                        current_version,
                        comparator: policy.clone(),
                        reason: reason_for_policy(&policy, explicit_comparator),
                    });
                } else {
                    upstream.push(UpstreamCause::SkippedByComparator {
                        source: dependency.source(),
                        aspect: dependency.aspect(),
                        subscription: subscription.clone(),
                        cached_version,
                        current_version,
                        comparator: policy.clone(),
                        reason: reason_for_policy(&policy, explicit_comparator),
                    });
                }
            }
            VersionComparatorPolicy::OutputIdentity
            | VersionComparatorPolicy::Custom { .. }
            | VersionComparatorPolicy::Installed { .. } => upstream.push(UpstreamCause::Changed {
                source: dependency.source(),
                aspect: dependency.aspect(),
                subscription: subscription.clone(),
                cached_version,
                current_version,
                comparator: policy.clone(),
                reason: reason_for_policy(&policy, explicit_comparator),
            }),
        }
    }

    let snapshot_entries = lineage.snapshot_entries().to_vec();
    for snapshot_entry in &snapshot_entries {
        if !lineage.contains_current_dependency(
            snapshot_entry.source,
            snapshot_entry.aspect.index(),
            snapshot_entry.scope.as_ref(),
        ) {
            lineage.traversal_cost_mut().note_removed_dependency();
            upstream.push(UpstreamCause::DependencyRemoved {
                source: snapshot_entry.source,
                aspect: snapshot_entry.aspect,
                subscription: snapshot_entry.scope.clone(),
                cached_version: snapshot_entry.cached_version,
            });
        }
    }

    canonicalize_upstream_causes(&mut upstream);
    Ok(upstream)
}

fn canonicalize_upstream_causes(upstream: &mut [UpstreamCause]) {
    upstream.sort_by_key(|cause| match cause {
        UpstreamCause::Changed { source, aspect, .. }
        | UpstreamCause::SkippedByComparator { source, aspect, .. }
        | UpstreamCause::ConditionDeferred { source, aspect, .. }
        | UpstreamCause::Clean { source, aspect, .. }
        | UpstreamCause::MissingSnapshot { source, aspect, .. }
        | UpstreamCause::DependencyRemoved { source, aspect, .. } => {
            let scope_key = match cause {
                UpstreamCause::Changed { subscription, .. }
                | UpstreamCause::SkippedByComparator { subscription, .. }
                | UpstreamCause::ConditionDeferred { subscription, .. }
                | UpstreamCause::Clean { subscription, .. }
                | UpstreamCause::MissingSnapshot { subscription, .. }
                | UpstreamCause::DependencyRemoved { subscription, .. } => subscription
                    .as_ref()
                    .map(|scope| {
                        (
                            scope.partition.0.clone(),
                            scope.detail.clone().unwrap_or_default(),
                            scope.match_mode as u8,
                        )
                    })
                    .unwrap_or_default(),
            };
            (
                source.index(),
                source.generation(),
                aspect.index(),
                scope_key,
            )
        }
    });
}
