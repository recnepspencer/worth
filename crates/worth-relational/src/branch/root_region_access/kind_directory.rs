//! Kind dispatch is carried by the selected immutable root.
use super::*;
impl RelationalBranchRoot {
    pub(crate) fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        self.regions.entity_ids_of_kind(kind, partition)
    }
    pub(crate) fn relation_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.regions.relation_ids_of_kind(kind, partition)
    }
    pub(crate) fn entity_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.entity_regions_of_kind(kind)
    }
    pub(crate) fn relation_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.relation_regions_of_kind(kind)
    }
}
