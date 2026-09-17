use std::sync::Arc;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

pub(super) fn rebase(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: Vec<WorthQueryApplicationObservedFact>,
) -> Arc<[WorthQueryApplicationObservedFact]> {
    facts
        .into_iter()
        .map(|fact| match fact {
            WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id,
                native_revision: runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_aspect_version(entity_id, &aspect))
                    .unwrap_or(native_revision),
                aspect,
            },
            WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                endpoints,
            } => WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision: runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| {
                        view.bounded_adjacency_structural_revision(
                            anchor,
                            relation_kind,
                            direction,
                            comparison_work_limit,
                        )
                        .ok()
                    })
                    .map(|comparison| comparison.revision())
                    .unwrap_or(native_revision),
                comparison_work_limit,
                endpoints,
            },
            fact => fact,
        })
        .collect::<Vec<_>>()
        .into()
}
