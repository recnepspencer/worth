use std::collections::BTreeMap;

use crate::storage::data::{EntityReadRecord, RelationReadRecord, RelationalReadView};
pub(super) struct PolicyReadViewContext<'a> {
    view: &'a RelationalReadView,
    index: &'a PolicyReadViewIndex,
}

pub(super) struct PolicyReadViewIndex {
    entities_by_lineage:
        BTreeMap<crate::identity::data::LineageId, crate::identity::data::EntityId>,
    entities_by_slot:
        BTreeMap<(crate::identity::data::PartitionId, u64), crate::identity::data::EntityId>,
    relations_by_slot:
        BTreeMap<(crate::identity::data::PartitionId, u64), crate::identity::data::RelationId>,
}

impl PolicyReadViewIndex {
    pub(super) fn new(view: &RelationalReadView) -> Self {
        let entities_by_lineage = view
            .entities()
            .iter()
            .filter_map(|entity| {
                entity
                    .lineage_id
                    .map(|lineage_id| (lineage_id, entity.entity_id))
            })
            .collect();
        let entities_by_slot = view
            .entities()
            .iter()
            .map(|entity| {
                (
                    (entity.entity_id.partition_id, entity.entity_id.local_slot.0),
                    entity.entity_id,
                )
            })
            .collect();
        let relations_by_slot = view
            .relations()
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
            .collect();
        Self {
            entities_by_lineage,
            entities_by_slot,
            relations_by_slot,
        }
    }
}

impl<'a> PolicyReadViewContext<'a> {
    pub(super) fn new(view: &'a RelationalReadView, index: &'a PolicyReadViewIndex) -> Self {
        Self { view, index }
    }

    pub(super) fn entity_for_record(
        &self,
        entity_id: crate::identity::data::EntityId,
        lineage_hint: Option<crate::identity::data::LineageId>,
    ) -> Option<&EntityReadRecord> {
        self.view
            .get_entity(entity_id)
            .or_else(|| {
                lineage_hint
                    .and_then(|lineage_id| self.index.entities_by_lineage.get(&lineage_id).copied())
                    .and_then(|resolved_entity_id| self.view.get_entity(resolved_entity_id))
            })
            .or_else(|| {
                self.index
                    .entities_by_slot
                    .get(&(entity_id.partition_id, entity_id.local_slot.0))
                    .copied()
                    .and_then(|resolved_entity_id| self.view.get_entity(resolved_entity_id))
            })
    }

    pub(super) fn relation_for_record(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<&RelationReadRecord> {
        self.view.get_relation(relation_id).or_else(|| {
            self.index
                .relations_by_slot
                .get(&(relation_id.partition_id, relation_id.local_slot.0))
                .copied()
                .and_then(|resolved_relation_id| self.view.get_relation(resolved_relation_id))
        })
    }
}

/// A policy base lends its admitted canonical patch as well as visible records.
/// Source and target record views need no envelope demand for this purpose.
pub(super) struct PolicyBaseReadViewContext<'a> {
    records: PolicyReadViewContext<'a>,
    envelope: &'a crate::history::data::CanonicalCommitEnvelope,
}

impl<'a> PolicyBaseReadViewContext<'a> {
    pub(super) fn new(
        view: &'a RelationalReadView,
        index: &'a PolicyReadViewIndex,
        envelope: &'a crate::history::data::CanonicalCommitEnvelope,
    ) -> Self {
        Self {
            records: PolicyReadViewContext::new(view, index),
            envelope,
        }
    }
    pub(super) fn records(&self) -> &PolicyReadViewContext<'a> {
        &self.records
    }
    pub(super) fn envelope(&self) -> &crate::history::data::CanonicalCommitEnvelope {
        self.envelope
    }
}
