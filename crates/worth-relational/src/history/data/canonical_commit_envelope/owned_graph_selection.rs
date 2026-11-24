//! Descriptive provenance of a Native-prepared graph selection; never mutation authority.
use super::*;
use crate::branch::RelationalBranchBasisDescriptor;
use crate::identity::data::EntityId;
use crate::identity::data::KindId;
use crate::identity::data::RelationId;
use crate::validation::data::structural_read::StructuralRelationRecord;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SelectedEntity {
    pub(crate) id: EntityId,
    pub(crate) kind: KindId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOwnedGraphSelection {
    selected_basis: RelationalBranchBasisDescriptor,
    model: EntityId,
    current_entities: Vec<EntityId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    transfer_participants: Vec<EntityId>,
    selected_entities: Vec<SelectedEntity>,
    current_relations: Vec<StructuralRelationRecord>,
    selected_relations: Vec<StructuralRelationRecord>,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl CanonicalOwnedGraphSelection {
    pub(crate) fn selected_entity_decode_storage(capacity: usize) -> Option<u64> {
        u64::try_from(
            std::alloc::Layout::array::<SelectedEntity>(capacity)
                .ok()?
                .size(),
        )
        .ok()
    }
    pub(crate) fn from_observed(
        selected_basis: RelationalBranchBasisDescriptor,
        model: EntityId,
        current_entities: Vec<EntityId>,
        transfer_participants: Vec<EntityId>,
        selected_entities: Vec<SelectedEntity>,
        current_relations: Vec<StructuralRelationRecord>,
        selected_relations: Vec<StructuralRelationRecord>,
    ) -> Self {
        Self {
            _decoded_memory: None,
            selected_basis,
            model,
            current_entities,
            transfer_participants,
            selected_entities,
            current_relations,
            selected_relations,
        }
    }
    pub(crate) fn transfer_participants(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.transfer_participants.iter().copied()
    }
    pub fn model(&self) -> EntityId {
        self.model
    }
    pub fn selected_basis(&self) -> &RelationalBranchBasisDescriptor {
        &self.selected_basis
    }
    pub(crate) fn selected_entities(
        &self,
    ) -> impl ExactSizeIterator<Item = (EntityId, KindId)> + '_ {
        self.selected_entities
            .iter()
            .map(|entity| (entity.id, entity.kind))
    }
    pub(crate) fn selected_entity_kind(&self, id: EntityId) -> Option<KindId> {
        self.selected_entities
            .binary_search_by_key(&id, |entity| entity.id)
            .ok()
            .map(|index| self.selected_entities[index].kind)
    }
    pub(crate) fn selected_relations(&self) -> &[StructuralRelationRecord] {
        &self.selected_relations
    }
    pub(crate) fn current_relations(&self) -> &[StructuralRelationRecord] {
        &self.current_relations
    }
    pub(crate) fn current_entities(&self) -> &[EntityId] {
        &self.current_entities
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

impl PartialEq for CanonicalOwnedGraphSelection {
    fn eq(&self, other: &Self) -> bool {
        self.selected_basis == other.selected_basis
            && self.model == other.model
            && self.current_entities == other.current_entities
            && self.transfer_participants == other.transfer_participants
            && self.selected_entities == other.selected_entities
            && self.current_relations == other.current_relations
            && self.selected_relations == other.selected_relations
    }
}
impl Eq for CanonicalOwnedGraphSelection {}
