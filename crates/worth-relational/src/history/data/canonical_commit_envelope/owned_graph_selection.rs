//! Descriptive provenance of a Native-prepared graph selection; never mutation authority.
use super::*;
use crate::{
    branch::RelationalOwnedModelGraphSelection,
    identity::data::{EntityId, KindId, RelationId},
    storage::overlay::CurrentPartitionAccess,
    validation::data::StructuralRelationRecord,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SelectedEntity {
    id: EntityId,
    kind: KindId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalOwnedGraphSelection {
    selected_basis: crate::branch::RelationalBranchBasisDescriptor,
    model: EntityId,
    current_entities: Vec<EntityId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    transfer_participants: Vec<EntityId>,
    selected_entities: Vec<SelectedEntity>,
    current_relations: Vec<StructuralRelationRecord>,
    selected_relations: Vec<StructuralRelationRecord>,
}
impl CanonicalOwnedGraphSelection {
    pub(crate) fn from_prepared(
        graph: &RelationalOwnedModelGraphSelection,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        let entities = graph.entities();
        let mut selected_entities = Vec::new();
        for &id in graph.restored_entities() {
            let kind = entities.selected_basis().inner.root.get_current_partition(id.partition_id)
                .and_then(|region| region.entity(id)).filter(|slot| slot.is_live()).and_then(|slot| slot.kind_id())
                .ok_or_else(|| crate::transactions::data::TransactionCommitError::conflict(
                    crate::transactions::data::CommitConflict::new(crate::transactions::data::ConflictClass::MutationStateInconsistency {
                        detail: "prepared graph selected entity unavailable".into(),
                        evidence: crate::transactions::data::MutationStateInconsistencyEvidence::MaterializationTransition { record: RecordRef::Entity(id) },
                    })))?;
            selected_entities.push(SelectedEntity { id, kind });
        }
        Ok(Self {
            selected_basis: entities
                .selected_basis()
                .descriptor()
                .in_canonical_frame()
                .map_err(|_| {
                    crate::mvcc::origin_denial("graph selection basis cannot enter canonical frame")
                })?,
            model: entities.model(),
            current_entities: graph.current_entities().iter().copied().collect(),
            transfer_participants: graph.transfer_participants().iter().copied().collect(),
            selected_entities,
            current_relations: graph.current_relations().values().copied().collect(),
            selected_relations: graph.selected_relations().values().copied().collect(),
        })
    }
    pub(crate) fn transfer_participants(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.transfer_participants.iter().copied()
    }
    pub fn model(&self) -> EntityId {
        self.model
    }
    pub fn selected_basis(&self) -> &crate::branch::RelationalBranchBasisDescriptor {
        &self.selected_basis
    }
    pub(crate) fn selected_entities(
        &self,
    ) -> impl ExactSizeIterator<Item = (EntityId, KindId)> + '_ {
        self.selected_entities
            .iter()
            .map(|entity| (entity.id, entity.kind))
    }
    pub(crate) fn selected_relations(&self) -> &[StructuralRelationRecord] {
        &self.selected_relations
    }
    pub(crate) fn current_relations(&self) -> &[StructuralRelationRecord] {
        &self.current_relations
    }
    pub(crate) fn retired_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.current_entities.iter().copied().filter(|id| {
            self.selected_entities
                .binary_search_by_key(id, |entity| entity.id)
                .is_err()
        })
    }
    pub(crate) fn retired_relations(&self) -> impl Iterator<Item = RelationId> + '_ {
        self.current_relations
            .iter()
            .map(|edge| edge.relation_id)
            .filter(|id| {
                self.selected_relations
                    .binary_search_by_key(id, |edge| edge.relation_id)
                    .is_err()
            })
    }
    pub(crate) fn partitions(
        &self,
    ) -> impl Iterator<Item = crate::identity::data::PartitionId> + Clone + '_ {
        self.targets()
            .map(|target| match target {
                RecordRef::Entity(id) => id.partition_id,
                RecordRef::Relation(id) => id.partition_id,
            })
            .chain(
                self.current_relations
                    .iter()
                    .chain(&self.selected_relations)
                    .flat_map(|edge| [edge.source.partition_id, edge.target.partition_id]),
            )
    }
    pub(crate) fn targets(&self) -> impl Iterator<Item = RecordRef> + Clone + '_ {
        self.current_entities
            .iter()
            .copied()
            .map(RecordRef::Entity)
            .chain(
                self.selected_entities
                    .iter()
                    .map(|e| RecordRef::Entity(e.id)),
            )
            .chain(
                self.current_relations
                    .iter()
                    .map(|edge| RecordRef::Relation(edge.relation_id)),
            )
            .chain(
                self.selected_relations
                    .iter()
                    .map(|e| RecordRef::Relation(e.relation_id)),
            )
    }
}
