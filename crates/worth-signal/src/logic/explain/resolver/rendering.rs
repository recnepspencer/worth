use crate::data::graph::SignalGraph;
use crate::data::output::PartitionSubscription;

use super::super::types::{
    CausalDisposition, CausalLink, CausalLinkKind, ScopeProvenance, ScopeProvenanceKind,
    UpstreamCause,
};
use super::lineage::ExplanationTraversalCost;

pub(super) fn build_causal_link_with_graph(
    graph: &SignalGraph,
    cause: &UpstreamCause,
    traversal_cost: &mut ExplanationTraversalCost,
) -> CausalLink {
    let scope = scope_provenance_for_cause(graph, cause, traversal_cost);
    let scope_untouched = matches!(scope.kind, ScopeProvenanceKind::Discarded);
    match cause {
        UpstreamCause::Changed {
            source,
            aspect,
            subscription,
            cached_version,
            current_version,
            comparator,
            reason,
        } => {
            let note = scope_note_for_changed(subscription, &scope);
            CausalLink {
                source: Some(*source),
                aspect: Some(*aspect),
                disposition: CausalDisposition::Semantic,
                kind: CausalLinkKind::Changed,
                scope,
                cached_version: Some(*cached_version),
                current_version: Some(*current_version),
                comparator: Some(comparator.clone()),
                reason: Some(reason.clone()),
                note,
            }
        }
        UpstreamCause::SkippedByComparator {
            source,
            aspect,
            subscription: _,
            cached_version,
            current_version,
            comparator,
            reason,
        } => CausalLink {
            source: Some(*source),
            aspect: Some(*aspect),
            disposition: CausalDisposition::Suppressed,
            kind: CausalLinkKind::SkippedByComparator,
            scope,
            cached_version: Some(*cached_version),
            current_version: Some(*current_version),
            comparator: Some(comparator.clone()),
            reason: Some(reason.clone()),
            note: None,
        },
        UpstreamCause::ConditionDeferred {
            source,
            aspect,
            subscription: _,
            cached_version,
            current_version,
            condition,
            decision,
        } => CausalLink {
            source: Some(*source),
            aspect: Some(*aspect),
            disposition: CausalDisposition::Ignored,
            kind: CausalLinkKind::ConditionDeferred {
                condition: condition.clone(),
                decision: *decision,
            },
            scope,
            cached_version: Some(*cached_version),
            current_version: Some(*current_version),
            comparator: None,
            reason: None,
            note: None,
        },
        UpstreamCause::Clean {
            source,
            aspect,
            subscription,
            cached_version,
            current_version,
        } => CausalLink {
            source: Some(*source),
            aspect: Some(*aspect),
            disposition: CausalDisposition::Ignored,
            kind: if scope_untouched
                || (subscription.is_some() && cached_version != current_version)
            {
                CausalLinkKind::ScopeUntouched
            } else {
                CausalLinkKind::Clean
            },
            scope,
            cached_version: Some(*cached_version),
            current_version: Some(*current_version),
            comparator: None,
            reason: None,
            note: if scope_untouched
                || (subscription.is_some() && cached_version != current_version)
            {
                Some("partition-sensitive validation discarded the upstream change for this local scope".to_string())
            } else {
                None
            },
        },
        UpstreamCause::MissingSnapshot {
            source,
            aspect,
            subscription: _,
            current_version,
        } => CausalLink {
            source: Some(*source),
            aspect: Some(*aspect),
            disposition: CausalDisposition::Conservative,
            kind: CausalLinkKind::MissingSnapshot,
            scope,
            cached_version: None,
            current_version: *current_version,
            comparator: None,
            reason: None,
            note: Some("missing dependency snapshot".to_string()),
        },
        UpstreamCause::DependencyRemoved {
            source,
            aspect,
            subscription: _,
            cached_version,
        } => CausalLink {
            source: Some(*source),
            aspect: Some(*aspect),
            disposition: CausalDisposition::Topology,
            kind: CausalLinkKind::DependencyRemoved,
            scope,
            cached_version: Some(*cached_version),
            current_version: None,
            comparator: None,
            reason: None,
            note: Some("dependency rewired away from current topology".to_string()),
        },
    }
}

fn scope_provenance_for_cause(
    graph: &SignalGraph,
    cause: &UpstreamCause,
    traversal_cost: &mut ExplanationTraversalCost,
) -> ScopeProvenance {
    let (source, subscription, changed, missing_snapshot) = match cause {
        UpstreamCause::Changed {
            source,
            subscription,
            ..
        }
        | UpstreamCause::SkippedByComparator {
            source,
            subscription,
            ..
        }
        | UpstreamCause::ConditionDeferred {
            source,
            subscription,
            ..
        }
        | UpstreamCause::Clean {
            source,
            subscription,
            ..
        }
        | UpstreamCause::MissingSnapshot {
            source,
            subscription,
            ..
        }
        | UpstreamCause::DependencyRemoved {
            source,
            subscription,
            ..
        } => (
            *source,
            subscription.clone(),
            !matches!(
                cause,
                UpstreamCause::Clean { .. } | UpstreamCause::DependencyRemoved { .. }
            ),
            matches!(cause, UpstreamCause::MissingSnapshot { .. }),
        ),
    };

    let Some(validation_scope) = subscription else {
        return ScopeProvenance {
            source_scope: None,
            validation_scope: None,
            kind: ScopeProvenanceKind::None,
            note: None,
        };
    };

    if missing_snapshot {
        return ScopeProvenance {
            source_scope: Some(validation_scope.clone()),
            validation_scope: Some(validation_scope),
            kind: ScopeProvenanceKind::InsufficientEvidence,
            note: Some(
                "recomputed conservatively because dependency snapshot evidence was missing"
                    .to_string(),
            ),
        };
    }

    traversal_cost.note_runtime_artifact_lookup();
    let source_scope = graph
        .node_runtime_artifact_state(source)
        .ok()
        .flatten()
        .and_then(|trace| {
            let changed_scopes = trace.changed_scopes().as_slice();
            translated_source_scope(changed_scopes, &validation_scope).or_else(|| {
                // Exact scoped versions remain equal for disjoint writes.
                // Preserve the latest producer region as discarded diagnostic
                // evidence without treating it as a cause of recomputation.
                matches!(cause, UpstreamCause::Clean { .. })
                    .then(|| changed_scopes.first().cloned())
                    .flatten()
            })
        });

    let (kind, note) = match (source_scope.as_ref(), changed) {
        (Some(source_scope), true) if *source_scope != validation_scope => (
            ScopeProvenanceKind::Translated,
            Some("upstream region evidence was translated into this node's validation scope".to_string()),
        ),
        (Some(_), true) => (ScopeProvenanceKind::Direct, None),
        (Some(_), false) => (
            ScopeProvenanceKind::Discarded,
            Some("scope evidence was considered local but untouched for this node".to_string()),
        ),
        (None, true) => (
            ScopeProvenanceKind::InsufficientEvidence,
            Some("partition-sensitive validation fell back because upstream region evidence was insufficient".to_string()),
        ),
        (None, false) => (ScopeProvenanceKind::Direct, None),
    };

    ScopeProvenance {
        source_scope: source_scope.or_else(|| Some(validation_scope.clone())),
        validation_scope: Some(validation_scope),
        kind,
        note,
    }
}

fn scope_note_for_changed(
    subscription: &Option<PartitionSubscription>,
    scope: &ScopeProvenance,
) -> Option<String> {
    subscription.as_ref().map(|_| match scope.kind {
        ScopeProvenanceKind::Translated => {
            "partition-sensitive scope participated via translated upstream region evidence"
                .to_string()
        }
        _ => "partition-sensitive scope participated directly in this causal frontier".to_string(),
    })
}

fn translated_source_scope(
    changed_scopes: &[PartitionSubscription],
    validation_scope: &PartitionSubscription,
) -> Option<PartitionSubscription> {
    changed_scopes
        .iter()
        .find(|scope| crate::data::output::scopes_overlap(*scope, validation_scope))
        .cloned()
}
