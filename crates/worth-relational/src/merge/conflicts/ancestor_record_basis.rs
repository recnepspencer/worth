use std::collections::BTreeMap;

use worth_foundational::facade::AuthoritativeRecordAspectState;

use crate::identity::data::{EntityId, LineageId, PartitionId, RelationId};
use crate::merge::data::VisibleMergeRecord;
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};
use crate::transactions::data::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;

#[cfg(test)]
mod tests;

pub(super) struct AncestorRecordBasisContext<'a> {
    view: &'a RelationalReadView,
    entities_by_lineage: BTreeMap<LineageId, EntityId>,
    entities_by_slot: BTreeMap<(PartitionId, u64), EntityId>,
    relations_by_slot: BTreeMap<(PartitionId, u64), RelationId>,
}

pub(super) struct AncestorEntityRecordBasis<'a> {
    record: &'a EntityReadRecord,
}

pub(super) struct AncestorRelationRecordBasis<'a> {
    record: &'a RelationReadRecord,
}

impl<'a> AncestorRecordBasisContext<'a> {
    pub(super) fn new(view: &'a RelationalReadView) -> Self {
        Self {
            view,
            entities_by_lineage: index_entities_by_lineage(view),
            entities_by_slot: index_entities_by_slot(view),
            relations_by_slot: index_relations_by_slot(view),
        }
    }

    pub(super) fn entity_basis(
        &self,
        record: &VisibleMergeRecord,
        target_record: Option<&RecordRef>,
    ) -> Option<AncestorEntityRecordBasis<'a>> {
        let lineage_hints = entity_lineage_hints(record);
        for entity_id in entity_basis_candidates(record, target_record) {
            if let Some(entity) = self.entity_by_id_or_slot(entity_id) {
                return Some(AncestorEntityRecordBasis { record: entity });
            }
            for lineage_hint in lineage_hints.iter().flatten().copied() {
                if let Some(entity) = self.entity_by_lineage(lineage_hint) {
                    return Some(AncestorEntityRecordBasis { record: entity });
                }
            }
        }
        None
    }

    pub(super) fn relation_basis(
        &self,
        record: &VisibleMergeRecord,
        target_record: Option<&RecordRef>,
    ) -> Option<AncestorRelationRecordBasis<'a>> {
        for relation_id in relation_basis_candidates(record, target_record) {
            if let Some(relation) = self.relation_by_id_or_slot(relation_id) {
                return Some(AncestorRelationRecordBasis { record: relation });
            }
        }
        None
    }

    fn entity_by_id_or_slot(&self, entity_id: EntityId) -> Option<&'a EntityReadRecord> {
        self.view.get_entity(entity_id).or_else(|| {
            self.entities_by_slot
                .get(&(entity_id.partition_id, entity_id.local_slot.0))
                .copied()
                .and_then(|resolved_entity_id| self.view.get_entity(resolved_entity_id))
        })
    }

    fn entity_by_lineage(&self, lineage_id: LineageId) -> Option<&'a EntityReadRecord> {
        self.entities_by_lineage
            .get(&lineage_id)
            .copied()
            .and_then(|entity_id| self.view.get_entity(entity_id))
    }

    fn relation_by_id_or_slot(&self, relation_id: RelationId) -> Option<&'a RelationReadRecord> {
        self.view.get_relation(relation_id).or_else(|| {
            self.relations_by_slot
                .get(&(relation_id.partition_id, relation_id.local_slot.0))
                .copied()
                .and_then(|resolved_relation_id| self.view.get_relation(resolved_relation_id))
        })
    }
}

impl AncestorEntityRecordBasis<'_> {
    #[cfg(test)]
    pub(super) fn record_ref(&self) -> RecordRef {
        RecordRef::Entity(self.record.entity_id)
    }

    pub(super) fn lifecycle(&self) -> RecordLifecycleState {
        self.record.lifecycle
    }

    pub(super) fn authoritative_state(&self) -> Option<&AuthoritativeRecordAspectState> {
        self.record.authoritative_aspect_state.as_ref()
    }
}

impl AncestorRelationRecordBasis<'_> {
    #[cfg(test)]
    pub(super) fn record_ref(&self) -> RecordRef {
        RecordRef::Relation(self.record.relation_id)
    }

    pub(super) fn lifecycle(&self) -> RecordLifecycleState {
        self.record.lifecycle
    }

    pub(super) fn source_endpoint(&self) -> EntityId {
        self.record.source
    }

    pub(super) fn target_endpoint(&self) -> EntityId {
        self.record.target
    }

    pub(super) fn authoritative_state(&self) -> Option<&AuthoritativeRecordAspectState> {
        self.record.authoritative_aspect_state.as_ref()
    }
}

fn index_entities_by_lineage(view: &RelationalReadView) -> BTreeMap<LineageId, EntityId> {
    view.entities()
        .iter()
        .filter_map(|entity| {
            entity
                .lineage_id
                .map(|lineage_id| (lineage_id, entity.entity_id))
        })
        .collect()
}

fn index_entities_by_slot(view: &RelationalReadView) -> BTreeMap<(PartitionId, u64), EntityId> {
    view.entities()
        .iter()
        .map(|entity| {
            (
                (entity.entity_id.partition_id, entity.entity_id.local_slot.0),
                entity.entity_id,
            )
        })
        .collect()
}

fn index_relations_by_slot(view: &RelationalReadView) -> BTreeMap<(PartitionId, u64), RelationId> {
    view.relations()
        .iter()
        .map(|relation| {
            (
                (
                    relation.relation_id.partition_id,
                    relation.relation_id.local_slot.0,
                ),
                relation.relation_id,
            )
        })
        .collect()
}

fn entity_basis_candidates(
    record: &VisibleMergeRecord,
    target_record: Option<&RecordRef>,
) -> Vec<EntityId> {
    let mut candidates = Vec::new();
    push_entity_record_candidate(target_record, &mut candidates);
    push_entity_record_candidate(Some(&record.record_ref), &mut candidates);
    if let Some(entity) = &record.source_entity {
        push_unique_entity_candidate(entity.entity_id, &mut candidates);
    }
    if let Some(entity) = &record.target_entity {
        push_unique_entity_candidate(entity.entity_id, &mut candidates);
    }
    candidates
}

fn entity_lineage_hints(record: &VisibleMergeRecord) -> [Option<LineageId>; 5] {
    [
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
    ]
}

fn push_entity_record_candidate(record_ref: Option<&RecordRef>, candidates: &mut Vec<EntityId>) {
    if let Some(RecordRef::Entity(entity_id)) = record_ref {
        push_unique_entity_candidate(*entity_id, candidates);
    }
}

fn push_unique_entity_candidate(entity_id: EntityId, candidates: &mut Vec<EntityId>) {
    if !candidates.contains(&entity_id) {
        candidates.push(entity_id);
    }
}

fn relation_basis_candidates(
    record: &VisibleMergeRecord,
    target_record: Option<&RecordRef>,
) -> Vec<RelationId> {
    let mut candidates = Vec::new();
    push_relation_record_candidate(target_record, &mut candidates);
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
    candidates: &mut Vec<RelationId>,
) {
    if let Some(RecordRef::Relation(relation_id)) = record_ref {
        push_unique_relation_candidate(*relation_id, candidates);
    }
}

fn push_unique_relation_candidate(relation_id: RelationId, candidates: &mut Vec<RelationId>) {
    if !candidates.contains(&relation_id) {
        candidates.push(relation_id);
    }
}
