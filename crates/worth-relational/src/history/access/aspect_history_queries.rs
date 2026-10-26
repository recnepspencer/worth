use std::collections::BTreeSet;

use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{
    AspectHistoryCommitSpan, AspectHistoryEntry, AspectHistoryLineageEventSpan,
    AspectHistoryOrigin, AspectHistoryQueryResult, AspectHistoryResolutionTrace,
    AspectResolutionContext, BranchId, HistoryAspectQueryTarget, LineageAspectHistory,
    LineageAspectHistoryQueryResult,
};
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::HistoricalResolutionRequest;
use crate::publication::patch::data::ordered_aspect_keys;
use crate::transactions::data::RecordRef;
use crate::visibility::materialization::read_records::ProjectionAspectFilter;

use super::HistoryAccess;

impl<'runtime> HistoryAccess<'runtime> {
    pub fn entity_aspect_history(
        &self,
        branch_id: &BranchId,
        entity_id: EntityId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Vec<AspectHistoryEntry> {
        self.entity_aspect_history_with_trace(branch_id, entity_id, filter)
            .entries
    }

    pub fn relation_aspect_history(
        &self,
        branch_id: &BranchId,
        relation_id: RelationId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Vec<AspectHistoryEntry> {
        self.relation_aspect_history_with_trace(branch_id, relation_id, filter)
            .entries
    }

    pub fn entity_aspect_history_with_trace(
        &self,
        branch_id: &BranchId,
        entity_id: EntityId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> AspectHistoryQueryResult {
        self.record_aspect_history_with_trace(
            branch_id,
            RecordRef::Entity(entity_id),
            HistoryAspectQueryTarget::Entity(entity_id),
            filter,
        )
    }

    pub fn relation_aspect_history_with_trace(
        &self,
        branch_id: &BranchId,
        relation_id: RelationId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> AspectHistoryQueryResult {
        self.record_aspect_history_with_trace(
            branch_id,
            RecordRef::Relation(relation_id),
            HistoryAspectQueryTarget::Relation(relation_id),
            filter,
        )
    }

    pub fn lineage_entity_aspect_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        lineage_id: LineageId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<Option<LineageAspectHistory>, crate::execution::RelationalExecutionDenialCause>
    {
        Ok(self
            .lineage_entity_aspect_history_with_trace(execution, branch_id, lineage_id, filter)?
            .history)
    }

    pub fn lineage_entity_aspect_history_with_trace(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
        lineage_id: LineageId,
        filter: Option<&ProjectionAspectFilter>,
    ) -> Result<LineageAspectHistoryQueryResult, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        let resolution = self
            .runtime
            .lineage_access()
            .resolve_historical_lineage(HistoricalResolutionRequest {
                branch_id: branch_id.clone(),
                lineage_id,
                boundedness_basis:
                    crate::facade::lineage::HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            });
        let lineage_scope = self.lineage_scope(&resolution.start, &resolution.traversed_event_ids);
        let envelopes = self.branch_commit_envelopes(branch_id);
        let mut entity_scope = BTreeSet::new();
        for id in &lineage_scope {
            if let Some(node) = self.runtime.lineage.demand_node(execution, *id)? {
                entity_scope.insert(node.node().entity_id());
            }
        }
        let entries = envelopes
            .iter()
            .flat_map(|envelope| {
                envelope.committed_record_changes().filter_map(|change| {
                    if filter.is_some_and(|aspect_filter| {
                        !aspect_filter.matches_published_patch(&change.record.authoritative_patch)
                    }) {
                        return None;
                    }
                    match change.record.target {
                        RecordRef::Entity(entity_id) if entity_scope.contains(&entity_id) => {
                            Some(AspectHistoryEntry {
                                origin: aspect_history_origin(change),
                                resolution: AspectResolutionContext::ResolvedViaLineage {
                                    start_lineage_id: resolution.start,
                                    traversed_event_ids: resolution.traversed_event_ids.clone(),
                                },
                            })
                        }
                        _ => None,
                    }
                })
            })
            .collect::<Vec<_>>();
        let trace = AspectHistoryResolutionTrace {
            requested_target: HistoryAspectQueryTarget::Lineage(lineage_id),
            branch_id: branch_id.clone(),
            filter: filter.cloned(),
            resolved_aspects: Self::resolved_aspects(entries.iter()),
            searched_commit_span: Self::commit_span(&envelopes),
            searched_lineage_event_span: Self::lineage_event_span(&resolution.traversed_event_ids),
            returned_entries: entries.len() as u64,
            traversed_commits: envelopes.len() as u64,
            traversed_lineage_events: resolution.traversed_event_ids.len() as u64,
        };
        let history = if entries.is_empty() && lineage_scope.is_empty() {
            None
        } else {
            Some(LineageAspectHistory {
                requested_branch: branch_id.clone(),
                start_lineage_id: resolution.start,
                resolved_lineage_chain: resolution.resolved,
                entries,
                traversed_event_ids: resolution.traversed_event_ids,
            })
        };
        Ok(LineageAspectHistoryQueryResult { history, trace })
    }

    fn record_aspect_history_with_trace(
        &self,
        branch_id: &BranchId,
        target: RecordRef,
        requested_target: HistoryAspectQueryTarget,
        filter: Option<&ProjectionAspectFilter>,
    ) -> AspectHistoryQueryResult {
        let envelopes = self.branch_commit_envelopes(branch_id);
        let entries = envelopes
            .iter()
            .flat_map(|envelope| {
                envelope
                    .committed_record_changes_for_target(&target)
                    .filter(move |change| {
                        filter.is_none_or(|aspect_filter| {
                            aspect_filter
                                .matches_published_patch(&change.record.authoritative_patch)
                        })
                    })
                    .map(|change| AspectHistoryEntry {
                        origin: aspect_history_origin(change),
                        resolution: AspectResolutionContext::DirectRecordHistory,
                    })
            })
            .collect::<Vec<_>>();
        AspectHistoryQueryResult {
            trace: AspectHistoryResolutionTrace {
                requested_target,
                branch_id: branch_id.clone(),
                filter: filter.cloned(),
                resolved_aspects: Self::resolved_aspects(entries.iter()),
                searched_commit_span: Self::commit_span(&envelopes),
                searched_lineage_event_span: None,
                returned_entries: entries.len() as u64,
                traversed_commits: envelopes.len() as u64,
                traversed_lineage_events: 0,
            },
            entries,
        }
    }

    fn lineage_scope(
        &self,
        start_lineage_id: &LineageId,
        traversed_event_ids: &[u64],
    ) -> BTreeSet<LineageId> {
        let mut scope = BTreeSet::from([*start_lineage_id]);
        for event in self
            .runtime
            .lineage
            .shared_events()
            .iter()
            .filter(|event| traversed_event_ids.contains(&event.event_id))
        {
            scope.extend(event.sources.iter().copied());
            scope.extend(event.targets.iter().copied());
        }
        scope
    }

    fn resolved_aspects<'a>(
        entries: impl IntoIterator<Item = &'a AspectHistoryEntry>,
    ) -> Vec<worth_foundational::facade::AspectKey> {
        ordered_aspect_keys(
            entries
                .into_iter()
                .flat_map(|entry| entry.origin.changed_aspects.iter().cloned()),
        )
    }

    fn commit_span(envelopes: &[CanonicalCommitEnvelope]) -> Option<AspectHistoryCommitSpan> {
        Some(AspectHistoryCommitSpan {
            first_commit_id: envelopes.first()?.commit.commit_id,
            last_commit_id: envelopes.last()?.commit.commit_id,
        })
    }

    fn lineage_event_span(event_ids: &[u64]) -> Option<AspectHistoryLineageEventSpan> {
        Some(AspectHistoryLineageEventSpan {
            first_event_id: *event_ids.first()?,
            last_event_id: *event_ids.last()?,
        })
    }
}

fn aspect_history_origin(
    change: crate::replay::data::CommittedRecordChange<'_>,
) -> AspectHistoryOrigin {
    AspectHistoryOrigin {
        commit_id: change.commit.commit_id,
        version_id: change.commit.version_id,
        branch_id: change.commit.branch_id.clone(),
        target: change.record.target.clone(),
        structural_change: change.record.structural_change,
        changed_aspects: change.record.authoritative_changed_aspects(),
        contains_opaque_aspect: change.record.contains_opaque_aspect,
    }
}
