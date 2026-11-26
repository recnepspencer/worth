use super::*;
use crate::identity::data::VersionId;
use crate::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionViewOwnerPort as _;

impl super::ProjectionRecordSource for VisibilityProjectionView {
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        self.demand_all_full(execution)
    }
    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        self.demand_regions_head(execution, partitions)
    }
    fn admitted_head_region(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.admitted_head_region(id)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        self.demand_regions_full(execution, partitions)
    }

    fn selected_schema_registry(&self) -> Option<&crate::schema::data::RelationalSchemaRegistry> {
        self.selected_schema().map(|schema| schema.registry)
    }

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(crate::identity::data::PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        match self.basis.root() {
            Some(root) => root.try_for_each_partition_id(visit),
            None => Ok(()),
        }
    }

    fn selected_entity_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::EntityId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
        self.basis
            .root()
            .into_iter()
            .flat_map(move |root| root.entity_ids_of_kind(kind, partition))
    }

    fn selected_relation_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<
        Item = Result<
            crate::identity::data::RelationId,
            crate::storage::data::RelationalStorageUnavailable,
        >,
    > + '_ {
        self.basis
            .root()
            .into_iter()
            .flat_map(move |root| root.relation_ids_of_kind(kind, partition))
    }

    fn retained_head_region(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.basis.root()?.accepted_head_region_view(partition)
    }
    fn selected_storage(&self) -> Option<&dyn crate::storage::overlay::PartitionAccess> {
        if let Some(resident) = &self.resident {
            Some(resident)
        } else {
            self.basis
                .root()
                .is_some()
                .then_some(&self.basis as &dyn crate::storage::overlay::PartitionAccess)
        }
    }
    fn is_exact_basis(&self) -> bool {
        self.is_exact_basis()
    }
    fn version_id(&self) -> VersionId {
        self.version_id()
    }
}
