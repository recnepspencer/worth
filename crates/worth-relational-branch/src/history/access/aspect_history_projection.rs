//! Filter canonical changes and assemble immutable aspect history traces.
use crate::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CommittedRecordChange,
};
use crate::history::data::{
    AspectHistoryCommitSpan, AspectHistoryEntry, AspectHistoryLineageEventSpan,
    AspectHistoryOrigin, AspectHistoryQueryResult, AspectHistoryResolutionTrace,
    AspectResolutionContext, BranchId, HistoryAspectQueryTarget, LineageAspectHistory,
    LineageAspectHistoryQueryResult,
};
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::HistoricalLineageResolution;
use crate::publication::patch::data::ordered_aspect_keys;
use crate::transactions::data::primitives::RecordRef;
use crate::visibility::materialization::read_records::projection::contracts::ProjectionAspectFilter;
use std::collections::BTreeSet;
use std::sync::Arc;

pub fn lineage_history(
    envelopes: &[Arc<CanonicalCommitEnvelope>],
    branch_id: &BranchId,
    lineage_id: LineageId,
    lineage_scope: &BTreeSet<LineageId>,
    entity_scope: &BTreeSet<EntityId>,
    resolution: HistoricalLineageResolution,
    filter: Option<&ProjectionAspectFilter>,
) -> LineageAspectHistoryQueryResult {
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
        resolved_aspects: resolved_aspects(entries.iter()),
        searched_commit_span: commit_span(envelopes),
        searched_lineage_event_span: lineage_event_span(&resolution.traversed_event_ids),
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
    LineageAspectHistoryQueryResult { history, trace }
}

pub fn record_history(
    envelopes: &[Arc<CanonicalCommitEnvelope>],
    branch_id: &BranchId,
    target: RecordRef,
    requested_target: HistoryAspectQueryTarget,
    filter: Option<&ProjectionAspectFilter>,
) -> AspectHistoryQueryResult {
    let entries = envelopes
        .iter()
        .flat_map(|envelope| {
            envelope
                .committed_record_changes_for_target(&target)
                .filter(move |change| {
                    filter.is_none_or(|aspect_filter| {
                        aspect_filter.matches_published_patch(&change.record.authoritative_patch)
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
            resolved_aspects: resolved_aspects(entries.iter()),
            searched_commit_span: commit_span(envelopes),
            searched_lineage_event_span: None,
            returned_entries: entries.len() as u64,
            traversed_commits: envelopes.len() as u64,
            traversed_lineage_events: 0,
        },
        entries,
    }
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

fn commit_span(
    envelopes: &[std::sync::Arc<CanonicalCommitEnvelope>],
) -> Option<AspectHistoryCommitSpan> {
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
fn aspect_history_origin(change: CommittedRecordChange<'_>) -> AspectHistoryOrigin {
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
