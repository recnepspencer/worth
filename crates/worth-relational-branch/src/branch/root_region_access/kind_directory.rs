//! Kind dispatch is carried by the selected immutable root.
use super::*;
impl RelationalBranchRoot {
    pub fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::EntityId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
        self.regions.entity_ids_of_kind(kind, partition)
    }
    pub fn relation_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::RelationId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
        self.regions.relation_ids_of_kind(kind, partition)
    }
    pub fn entity_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.entity_regions_of_kind(kind)
    }
    pub fn relation_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.relation_regions_of_kind(kind)
    }
}
