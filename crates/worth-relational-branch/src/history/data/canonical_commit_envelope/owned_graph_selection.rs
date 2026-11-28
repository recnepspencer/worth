//! Descriptive provenance of a Native-prepared graph selection; never mutation authority.
use super::*;
use crate::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use crate::identity::data::EntityId;
use crate::identity::data::KindId;
use crate::identity::data::RelationId;
use crate::validation::data::structural_read::StructuralRelationRecord;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedEntity {
    pub id: EntityId,
    pub kind: KindId,
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
    /// Companion records outside the owned closure that follow the selection
    /// (tree-based undo's reversal facts, public #56 ruling (u)): retired
    /// live records of the current basis. Relation-free; absent in envelopes
    /// written before them, which therefore decode as "no companions".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    companion_retired: Vec<EntityId>,
    /// Companion records rematerialized from the selected basis in their
    /// original full-generation slots. Relation-free; absent before them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    companion_selected: Vec<SelectedEntity>,
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
    #[cfg(test)]
    pub(crate) fn from_observed(
        selected_basis: RelationalBranchBasisDescriptor,
        model: EntityId,
        current_entities: Vec<EntityId>,
        transfer_participants: Vec<EntityId>,
        selected_entities: Vec<SelectedEntity>,
        current_relations: Vec<StructuralRelationRecord>,
        selected_relations: Vec<StructuralRelationRecord>,
        companion_retired: Vec<EntityId>,
        companion_selected: Vec<SelectedEntity>,
    ) -> Self {
        Self {
            _decoded_memory: None,
            companion_retired,
            companion_selected,
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
    /// Companion records the selection retires (outside the owned closure).
    pub(crate) fn companion_retired(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.companion_retired.iter().copied()
    }
    /// Companion records the selection rematerializes, with their kinds.
    pub(crate) fn companion_selected(&self) -> impl Iterator<Item = (EntityId, KindId)> + '_ {
        self.companion_selected
            .iter()
            .map(|entity| (entity.id, entity.kind))
    }
    pub fn model(&self) -> EntityId {
        self.model
    }
    /// Whether this selection rematerialized `id` as a companion record, in
    /// its original slot with its retained payload (ruling (u)).
    pub fn restores_companion(&self, id: EntityId) -> bool {
        self.companion_selected.iter().any(|entity| entity.id == id)
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
            .chain(
                self.companion_retired
                    .iter()
                    .copied()
                    .map(RecordRef::Entity),
            )
            .chain(
                self.companion_selected
                    .iter()
                    .map(|e| RecordRef::Entity(e.id)),
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
            && self.companion_retired == other.companion_retired
            && self.companion_selected == other.companion_selected
    }
}
impl Eq for CanonicalOwnedGraphSelection {}

#[doc(hidden)]
pub trait CanonicalOwnedGraphSelectionOwnerPort: Sized {
    fn selected_entity_decode_storage(capacity: usize) -> Option<u64>;
    fn transfer_participants(&self) -> impl Iterator<Item = EntityId> + '_;
    fn companion_retired(&self) -> impl Iterator<Item = EntityId> + '_;
    fn companion_selected(&self) -> impl Iterator<Item = (EntityId, KindId)> + '_;
    fn selected_entities(&self) -> impl ExactSizeIterator<Item = (EntityId, KindId)> + '_;
    fn selected_entity_kind(&self, id: EntityId) -> Option<KindId>;
    fn selected_relations(&self) -> &[StructuralRelationRecord];
    fn current_relations(&self) -> &[StructuralRelationRecord];
    fn current_entities(&self) -> &[EntityId];
    fn retired_entities(&self) -> impl Iterator<Item = EntityId> + '_;
    fn retired_relations(&self) -> impl Iterator<Item = RelationId> + '_;
    fn partitions(&self) -> impl Iterator<Item = crate::identity::data::PartitionId> + Clone + '_;
    fn targets(&self) -> impl Iterator<Item = RecordRef> + Clone + '_;
}

impl CanonicalOwnedGraphSelectionOwnerPort for CanonicalOwnedGraphSelection {
    fn selected_entity_decode_storage(capacity: usize) -> Option<u64> {
        CanonicalOwnedGraphSelection::selected_entity_decode_storage(capacity)
    }

    fn transfer_participants(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.transfer_participants()
    }

    fn companion_retired(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.companion_retired()
    }

    fn companion_selected(&self) -> impl Iterator<Item = (EntityId, KindId)> + '_ {
        self.companion_selected()
    }

    fn selected_entities(&self) -> impl ExactSizeIterator<Item = (EntityId, KindId)> + '_ {
        self.selected_entities()
    }

    fn selected_entity_kind(&self, id: EntityId) -> Option<KindId> {
        self.selected_entity_kind(id)
    }

    fn selected_relations(&self) -> &[StructuralRelationRecord] {
        self.selected_relations()
    }

    fn current_relations(&self) -> &[StructuralRelationRecord] {
        self.current_relations()
    }

    fn current_entities(&self) -> &[EntityId] {
        self.current_entities()
    }

    fn retired_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.retired_entities()
    }

    fn retired_relations(&self) -> impl Iterator<Item = RelationId> + '_ {
        self.retired_relations()
    }

    fn partitions(&self) -> impl Iterator<Item = crate::identity::data::PartitionId> + Clone + '_ {
        self.partitions()
    }

    fn targets(&self) -> impl Iterator<Item = RecordRef> + Clone + '_ {
        self.targets()
    }
}

#[cfg(test)]
#[path = "owned_graph_selection_format_tests.rs"]
mod format_tests;
