use crate::contracts::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::merge::data::VisibleMergeRecord;
use crate::merge::policy::contexts::PolicyReadViewContext;

pub(super) fn entity_basis_for_visible_record<'a>(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
    base_view: &'a PolicyReadViewContext<'_>,
) -> Option<&'a EntityReadRecord> {
    let lineage_hints = [
        record.source_lineage_id,
        record.target_lineage_id,
        record.lineage_id,
        record
            .source_entity
            .as_ref()
            .and_then(|entity| entity.lineage_id),
        record
            .target_entity
            .as_ref()
            .and_then(|entity| entity.lineage_id),
    ];
    for entity_id in entity_basis_candidates(record, classification) {
        if let Some(entity) = base_view.entity_for_record(entity_id, None) {
            return Some(entity);
        }
        for lineage_hint in lineage_hints.into_iter().flatten() {
            if let Some(entity) = base_view.entity_for_record(entity_id, Some(lineage_hint)) {
                return Some(entity);
            }
        }
    }
    None
}

pub(super) fn entity_basis_record_refs(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
) -> Vec<RecordRef> {
    entity_basis_candidates(record, classification)
        .into_iter()
        .map(RecordRef::Entity)
        .collect()
}

fn entity_basis_candidates(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
) -> Vec<crate::contracts::identity::data::record_ids::EntityId> {
    let mut candidates = Vec::new();
    push_entity_record_candidate(classification.target_record.as_ref(), &mut candidates);
    push_entity_record_candidate(Some(&record.record_ref), &mut candidates);
    if let Some(entity) = &record.source_entity {
        push_unique_entity_candidate(entity.entity_id, &mut candidates);
    }
    if let Some(entity) = &record.target_entity {
        push_unique_entity_candidate(entity.entity_id, &mut candidates);
    }
    candidates
}

fn push_entity_record_candidate(
    record_ref: Option<&RecordRef>,
    candidates: &mut Vec<crate::contracts::identity::data::record_ids::EntityId>,
) {
    if let Some(RecordRef::Entity(entity_id)) = record_ref {
        push_unique_entity_candidate(*entity_id, candidates);
    }
}

fn push_unique_entity_candidate(
    entity_id: crate::contracts::identity::data::record_ids::EntityId,
    candidates: &mut Vec<crate::contracts::identity::data::record_ids::EntityId>,
) {
    if !candidates.contains(&entity_id) {
        candidates.push(entity_id);
    }
}

pub(super) fn relation_basis_for_visible_record<'a>(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
    base_view: &'a PolicyReadViewContext<'_>,
) -> Option<&'a RelationReadRecord> {
    for relation_id in relation_basis_candidates(record, classification) {
        if let Some(relation) = base_view.relation_for_record(relation_id) {
            return Some(relation);
        }
    }
    None
}

pub(super) fn relation_basis_record_refs(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
) -> Vec<RecordRef> {
    relation_basis_candidates(record, classification)
        .into_iter()
        .map(RecordRef::Relation)
        .collect()
}

fn relation_basis_candidates(
    record: &VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
) -> Vec<crate::contracts::identity::data::record_ids::RelationId> {
    let mut candidates = Vec::new();
    push_relation_record_candidate(classification.target_record.as_ref(), &mut candidates);
    push_relation_record_candidate(Some(&record.record_ref), &mut candidates);
    if let Some(relation) = &record.source_relation {
        push_unique_relation_candidate(relation.relation_id, &mut candidates);
    }
    if let Some(relation) = &record.target_relation {
        push_unique_relation_candidate(relation.relation_id, &mut candidates);
    }
    candidates
}

fn push_relation_record_candidate(
    record_ref: Option<&RecordRef>,
    candidates: &mut Vec<crate::contracts::identity::data::record_ids::RelationId>,
) {
    if let Some(RecordRef::Relation(relation_id)) = record_ref {
        push_unique_relation_candidate(*relation_id, candidates);
    }
}

fn push_unique_relation_candidate(
    relation_id: crate::contracts::identity::data::record_ids::RelationId,
    candidates: &mut Vec<crate::contracts::identity::data::record_ids::RelationId>,
) {
    if !candidates.contains(&relation_id) {
        candidates.push(relation_id);
    }
}
