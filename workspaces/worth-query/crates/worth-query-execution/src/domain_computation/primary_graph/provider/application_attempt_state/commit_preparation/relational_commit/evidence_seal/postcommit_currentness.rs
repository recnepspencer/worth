use std::sync::Arc;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

mod adjacency;
use adjacency::rebase_decision_adjacency;
mod retirement;

pub(super) fn rebase_output(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: Vec<WorthQueryApplicationObservedFact>,
    correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    changed_records: &[worth_relational::facade::transactions::RecordRef],
    producer_output: bool,
    maximum_pair_rebase_work: usize,
) -> Arc<[WorthQueryApplicationObservedFact]> {
    let changed_entities = changed_records
        .iter()
        .filter_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let retired = correspondence
        .retired_entity_ids()
        .filter(|entity| changed_entities.contains(entity))
        .collect();
    let Some(facts) = retirement::rebase(runtime, snapshot, facts, &retired) else {
        return Arc::from([]);
    };
    rebase(
        runtime,
        snapshot,
        facts,
        producer_output,
        maximum_pair_rebase_work,
    )
}

pub(super) fn rebase(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: Vec<WorthQueryApplicationObservedFact>,
    producer_output: bool,
    maximum_pair_rebase_work: usize,
) -> Arc<[WorthQueryApplicationObservedFact]> {
    let mut failed_native_rebase = false;
    let rebased = facts
        .into_iter()
        .map(|fact| match fact {
            WorthQueryApplicationObservedFact::Field {
                entity_id, locator, ..
            }
            | WorthQueryApplicationObservedFact::AbsentField {
                entity_id, locator, ..
            } => {
                let locator = worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    locator.aspect().aspect_key().clone(),
                    locator.field_path().clone(),
                );
                WorthQueryApplicationObservedFact::SourceFieldRevision {
                    entity_id,
                    native_revision: runtime
                        .read_truth()
                        .project_snapshot(snapshot)
                        .and_then(|view| view.entity_field_revision(entity_id, &locator)),
                    locator,
                }
            }
            WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => {
                let committed_revision = runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .and_then(|view| view.entity_aspect_version(entity_id, &aspect));
                if committed_revision.is_none() {
                    failed_native_rebase = true;
                }
                WorthQueryApplicationObservedFact::SourceAspectRevision {
                    entity_id,
                    native_revision: committed_revision.unwrap_or(native_revision),
                    aspect,
                }
            }
            WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                endpoints,
            } => {
                let committed_comparison = runtime
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
                    });
                if committed_comparison.is_none() {
                    failed_native_rebase = true;
                }
                WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                    relation_kind,
                    anchor,
                    direction,
                    native_revision: committed_comparison
                        .map(|comparison| comparison.revision())
                        .unwrap_or(native_revision),
                    comparison_work_limit,
                    endpoints,
                }
            }
            fact @ WorthQueryApplicationObservedFact::Relation { .. }
            | fact @ WorthQueryApplicationObservedFact::Adjacency { .. } => {
                rebase_decision_adjacency(runtime, snapshot, fact, maximum_pair_rebase_work)
            }
            fact => fact,
        })
        .collect::<Vec<_>>();
    if producer_output
        && (failed_native_rebase || !rebased.iter().all(native_output_currentness_fact))
    {
        // Failed structural acquisition or an unsupported tracked read cannot
        // be silently replaced by only the query-footprint subset.
        Arc::from([])
    } else {
        rebased.into()
    }
}

fn native_output_currentness_fact(fact: &WorthQueryApplicationObservedFact) -> bool {
    matches!(
        fact,
        WorthQueryApplicationObservedFact::RetiredOutputEntity { .. }
            | WorthQueryApplicationObservedFact::SourceEntity { .. }
            | WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
            | WorthQueryApplicationObservedFact::SourceFieldRevision { .. }
            | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            | WorthQueryApplicationObservedFact::Entity { .. }
    )
}

#[cfg(test)]
mod tests;
