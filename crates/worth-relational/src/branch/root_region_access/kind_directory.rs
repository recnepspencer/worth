//! Kind dispatch is carried by the selected immutable root.
use super::*;
impl RelationalBranchRoot {
    pub(crate) fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        let regions: Box<dyn Iterator<Item = PartitionId> + '_> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.regions.kinds.entity_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.regions
                .get(id)
                .into_iter()
                .flat_map(move |region| region.directory.entity_ids(kind))
        })
    }
    pub(crate) fn relation_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        let regions: Box<dyn Iterator<Item = PartitionId> + '_> = match partition {
            Some(id) => Box::new(std::iter::once(id)),
            None => Box::new(self.regions.kinds.relation_regions(kind)),
        };
        regions.flat_map(move |id| {
            self.regions
                .get(id)
                .into_iter()
                .flat_map(move |region| region.directory.relation_ids(kind))
        })
    }
    pub(crate) fn entity_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.kinds.entity_regions(kind)
    }
    pub(crate) fn relation_regions_of_kind(
        &self,
        kind: crate::identity::data::KindId,
    ) -> impl Iterator<Item = PartitionId> + '_ {
        self.regions.kinds.relation_regions(kind)
    }
}
